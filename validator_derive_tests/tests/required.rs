use serde::Serialize;
use validator::Validate;

#[derive(Serialize)]
struct ObjectRef {
    id: i32,
    name: String,
}

#[derive(Serialize, Validate)]
struct CheckedObjectRef {
    #[validate(range(min = 1))]
    id: i32,
    #[validate(length(min = 1))]
    name: String,
}

#[derive(Validate)]
struct Required {
    #[validate(required)]
    val: Option<ObjectRef>,
}

#[derive(Validate)]
struct RequiredNested {
    #[validate(required, nested)]
    val: Option<CheckedObjectRef>,
}

#[test]
fn can_validate_required() {
    let s = Required {
        val: Some(ObjectRef {
            id: 0,
            name: String::new(),
        }),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_required_nested() {
    let s = RequiredNested {
        val: Some(CheckedObjectRef {
            id: 1,
            name: String::from("Reference representation"),
        }),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn none_fails_required() {
    let s = Required { val: None };

    assert!(s.validate().is_err());
}

#[test]
fn none_fails_required_nested() {
    let s = RequiredNested { val: None };

    assert!(s.validate().is_err());
}

#[test]
fn can_specify_code_for_required() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(required(code = "oops"))]
        val: Option<String>,
    }
    let s = TestStruct { val: None };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].code, "oops");
}

#[test]
fn can_specify_message_for_required() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(required(message = "oops"))]
        val: Option<String>,
    }
    let s = TestStruct { val: None };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].clone().message.unwrap(), "oops");
}

#[test]
fn can_validate_custom_impl_for_required() {
    #[derive(Debug, Serialize)]
    enum CustomOption<T> {
        Something(T),
        Nothing,
    }

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(required)]
        val: CustomOption<String>,
    }

    impl<T> validator::ValidateRequired for CustomOption<T> {
        fn is_some(&self) -> bool {
            match self {
                CustomOption::Something(_) => true,
                CustomOption::Nothing => false,
            }
        }
    }

    let something = TestStruct {
        val: CustomOption::Something("this is something".to_string()),
    };
    let nothing = TestStruct {
        val: CustomOption::Nothing,
    };

    assert!(something.validate().is_ok());
    assert!(nothing.validate().is_err());
}

// ---- Required if ---- //

// ---- Condition Function --- //
// This function serves as the 'function' parameter for `required_if`
// It dictates whether the `user_settings` field is required.
fn is_admin_user(s: &TestOptionRequiredIf) -> bool {
    s.is_admin
}

// The `user_settings` field is required ONLY IF `is_admin` is true.
#[derive(Validate)]
struct TestOptionRequiredIf {
    #[validate(required_if(func = "is_admin_user"))]
    user_settings: Option<String>,
    is_admin: bool,
}

#[test]
fn test_required_if_option_field_is_some_and_condition_is_true() {
    // Scenario 1: Field is Some, Condition is true
    // Expected: VALID
    let instance = TestOptionRequiredIf {
        user_settings: Some("dark_mode".to_string()),
        is_admin: true, // Condition is true
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: user_settings=Some, is_admin=true"
    );
}

#[test]
fn test_required_if_option_field_is_some_and_condition_is_false() {
    // Scenario 2: Field is Some, Condition is false
    // Expected: VALID (Rule is skipped)
    let instance = TestOptionRequiredIf {
        user_settings: Some("light_mode".to_string()),
        is_admin: false, // Condition is false
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: user_settings=Some, is_admin=false (rule skipped)"
    );
}

#[test]
fn test_required_if_option_field_is_none_and_condition_is_true() {
    // Scenario 3: Field is None, Condition is true
    // Expected: INVALID (Field is required but missing)
    let instance = TestOptionRequiredIf {
        user_settings: None, // Field is None
        is_admin: true,      // Condition is true -> field is required
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: user_settings=None, is_admin=true"
    );

    // Assert the specific error code
    let errors = result.unwrap_err();
    let field_errors = errors.field_errors();
    assert!(
        field_errors.contains_key("user_settings"),
        "Expected error for 'user_settings' field"
    );
    assert_eq!(
        field_errors["user_settings"][0].code, "required_if",
        "Expected error code 'required_if'"
    );
}

#[test]
fn test_required_if_option_field_is_none_and_condition_is_false() {
    // Scenario 4: Field is None, Condition is false
    // Expected: VALID (Rule is skipped, so missing field is OK)
    let instance = TestOptionRequiredIf {
        user_settings: None, // Field is None
        is_admin: false,     // Condition is false -> field is NOT required
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: user_settings=None, is_admin=false (rule skipped)"
    );
}

// ---- Required with ---- //

#[derive(Validate)]
pub struct TestOptionRequiredWith {
    // `main_field` is required if either `other_field_1` OR `other_field_2` is Some.
    #[validate(required_with(other_fields("other_field_1", "other_field_2")))]
    main_field: Option<String>,
    other_field_1: Option<String>,
    other_field_2: Option<String>,
}

#[test]
fn test_required_with_main_field_is_some_always_valid() {
    // Scenario: The `main_field` is Some.
    // Expected: VALID. The rule passes because the field is already present,
    //           regardless of the state of `other_field_1` or `other_field_2`.
    let instance = TestOptionRequiredWith {
        main_field: Some("ValueA".to_string()),
        other_field_1: None,
        other_field_2: None,
    };

    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is Some, so required_with rule is satisfied."
    );
}

#[test]
fn test_required_with_main_field_is_none_and_no_others_are_some() {
    // Scenario: `main_field` is None, and NEITHER `other_field_1` NOR `other_field_2` are Some.
    // Expected: VALID. The condition for `required_with` is not met, so the rule passes.
    let instance = TestOptionRequiredWith {
        main_field: None,
        other_field_1: None,
        other_field_2: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is None, but no other fields are Some."
    );
}

#[test]
fn test_required_with_main_field_is_none_and_one_other_is_some() {
    // Scenario: `main_field` is None, but `other_field_1` IS Some.
    // Expected: INVALID. `main_field` is required because `other_field_1` is present.
    let instance = TestOptionRequiredWith {
        main_field: None,
        other_field_1: Some("Present".to_string()), // This makes `main_field` required
        other_field_2: None,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: main_field is None, but other_field_1 is Some."
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "required_with",
        "Error code should be 'required_with' for missing main_field."
    );
}

#[test]
fn test_required_with_main_field_is_none_and_multiple_others_are_some() {
    // Scenario: `main_field` is None, and BOTH `other_field_1` AND `other_field_2` are Some.
    // Expected: INVALID. `main_field` is required because multiple other fields are present.
    let instance = TestOptionRequiredWith {
        main_field: None,
        other_field_1: Some("ValueX".to_string()),
        other_field_2: Some("ValueY".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: main_field is None, and multiple other fields are Some."
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "required_with",
        "Error code should be 'required_with' for missing main_field."
    );
}

// ---- Required with all ---- //

#[derive(Validate, Debug)]
pub struct TestOptionRequiredWithAll {
    #[validate(required_with_all(other_fields("field_b", "field_c")))]
    main_field: Option<String>,
    field_b: Option<String>,
    field_c: Option<String>,
}

#[test]
fn test_required_with_all_main_field_is_some_always_valid() {
    // Scenario: `main_field` is Some.
    // Expected: VALID. The rule passes because the field is already present.
    let instance = TestOptionRequiredWithAll {
        main_field: Some("Value".to_string()),
        field_b: None,
        field_c: Some("X".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is Some, satisfying required_with_all."
    );
}

#[test]
fn test_required_with_all_main_field_is_none_and_not_all_others_are_some() {
    // Scenario: `main_field` is None, and NOT ALL `field_b` AND `field_c` are Some (i.e., one is None).
    // Expected: VALID. The condition for `required_with_all` is not met.
    let instance = TestOptionRequiredWithAll {
        main_field: None,
        field_b: Some("Present".to_string()),
        field_c: None, // One of the 'all' fields is None, so condition fails
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is None, but not all other fields are Some."
    );
}

#[test]
fn test_required_with_all_main_field_is_none_and_all_others_are_some() {
    // Scenario: `main_field` is None, but ALL `field_b` AND `field_c` ARE Some.
    // Expected: INVALID. `main_field` is required.
    let instance = TestOptionRequiredWithAll {
        main_field: None,
        field_b: Some("ValueB".to_string()),
        field_c: Some("ValueC".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: main_field is None, but ALL other fields are Some."
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "required_with_all",
        "Error code should be 'required_with_all'."
    );
}

// ---- Required without ---- //

#[derive(Validate, Debug)]
pub struct TestOptionRequiredWithout {
    #[validate(required_without(other_fields("field_d", "field_e")))]
    main_field: Option<String>,
    field_d: Option<String>,
    field_e: Option<String>,
}

#[test]
fn test_required_without_main_field_is_some_always_valid() {
    // Scenario: `main_field` is Some.
    // Expected: VALID. The rule passes because the field is already present.
    let instance = TestOptionRequiredWithout {
        main_field: Some("Value".to_string()),
        field_d: None,
        field_e: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is Some, satisfying required_without."
    );
}

#[test]
fn test_required_without_main_field_is_none_and_no_others_are_none() {
    // Scenario: `main_field` is None, and NEITHER `field_d` NOR `field_e` are None (i.e., both are Some).
    // Expected: VALID. The condition for `required_without` is not met.
    let instance = TestOptionRequiredWithout {
        main_field: None,
        field_d: Some("ValueD".to_string()),
        field_e: Some("ValueE".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is None, but no other fields are None."
    );
}

#[test]
fn test_required_without_main_field_is_none_and_one_other_is_none() {
    // Scenario: `main_field` is None, but `field_d` IS None.
    // Expected: INVALID. `main_field` is required because `field_d` is absent.
    let instance = TestOptionRequiredWithout {
        main_field: None,
        field_d: None, // This makes `main_field` required
        field_e: Some("ValueE".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: main_field is None, but one other field is None."
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "required_without",
        "Error code should be 'required_without'."
    );
}

// ---- Required without all ---- //

#[derive(Validate, Debug)]
pub struct TestOptionRequiredWithoutAll {
    #[validate(required_without_all(other_fields("field_f", "field_g")))]
    main_field: Option<String>,
    field_f: Option<String>,
    field_g: Option<String>,
}

#[test]
fn test_required_without_all_main_field_is_some_always_valid() {
    // Scenario: `main_field` is Some.
    // Expected: VALID. The rule passes because the field is already present.
    let instance = TestOptionRequiredWithoutAll {
        main_field: Some("Value".to_string()),
        field_f: None,
        field_g: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is Some, satisfying required_without_all."
    );
}

#[test]
fn test_required_without_all_main_field_is_none_and_not_all_others_are_none() {
    // Scenario: `main_field` is None, and NOT ALL `field_f` AND `field_g` are None (i.e., one is Some).
    // Expected: VALID. The condition for `required_without_all` is not met.
    let instance = TestOptionRequiredWithoutAll {
        main_field: None,
        field_f: Some("Present".to_string()), // One is Some, so condition fails
        field_g: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: main_field is None, but not all other fields are None."
    );
}

#[test]
fn test_required_without_all_main_field_is_none_and_all_others_are_none() {
    // Scenario: `main_field` is None, but ALL `field_f` AND `field_g` ARE None.
    // Expected: INVALID. `main_field` is required.
    let instance = TestOptionRequiredWithoutAll {
        main_field: None,
        field_f: None,
        field_g: None,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: main_field is None, and ALL other fields are None."
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "required_without_all",
        "Error code should be 'required_without_all'."
    );
}

// ============================================================================
// Option<Option<T>> (PATCH Semantics) Tests
// ============================================================================
//
// For Option<Option<T>> fields:
// - None = field omitted (don't update in PATCH context)
// - Some(None) = explicitly clear/reset the field
// - Some(Some(value)) = has a value
//
// "Has value" = Some(Some(_)) only. Both None and Some(None) mean "no value".

// ---- Required with Option<Option<T>> ---- //

#[derive(Validate, Debug)]
struct TestRequiredWithDoubleOption {
    // country_id is required when area_id has a value (not just present)
    #[validate(required_with(other_fields("area_id")))]
    country_id: Option<Option<i32>>,

    area_id: Option<Option<i32>>,
}

#[test]
fn test_required_with_double_option_both_omitted() {
    // Scenario: Both fields are omitted (None)
    // Expected: VALID - area_id has no value, so country_id is not required
    let instance = TestRequiredWithDoubleOption {
        country_id: None,
        area_id: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: area_id omitted (None), country_id not required"
    );
}

#[test]
fn test_required_with_double_option_other_clearing() {
    // Scenario: area_id is being cleared (Some(None))
    // Expected: VALID - clearing a field is not the same as having a value
    let instance = TestRequiredWithDoubleOption {
        country_id: None,
        area_id: Some(None), // Explicitly clearing the field
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: area_id is Some(None) (clearing), country_id not required"
    );
}

#[test]
fn test_required_with_double_option_other_has_value_main_missing() {
    // Scenario: area_id has a value (Some(Some(_))), but country_id is None
    // Expected: INVALID - country_id is required when area_id has a value
    let instance = TestRequiredWithDoubleOption {
        country_id: None,
        area_id: Some(Some(123)),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: area_id has value Some(Some(123)), but country_id is None"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["country_id"][0].code, "required_with");
}

#[test]
fn test_required_with_double_option_other_has_value_main_clearing() {
    // Scenario: area_id has a value (Some(Some(_))), but country_id is clearing (Some(None))
    // Expected: INVALID - country_id must have a value, not just be present
    let instance = TestRequiredWithDoubleOption {
        country_id: Some(None), // Clearing, not a value
        area_id: Some(Some(123)),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: area_id has value, but country_id is Some(None) (clearing, not a value)"
    );
}

#[test]
fn test_required_with_double_option_both_have_values() {
    // Scenario: Both fields have values
    // Expected: VALID
    let instance = TestRequiredWithDoubleOption {
        country_id: Some(Some(229)), // USA
        area_id: Some(Some(123)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields have values"
    );
}

#[test]
fn test_required_with_double_option_main_has_value_other_omitted() {
    // Scenario: country_id has value, area_id omitted
    // Expected: VALID - area_id has no value so country_id not required (but allowed)
    let instance = TestRequiredWithDoubleOption {
        country_id: Some(Some(229)),
        area_id: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: country_id has value, area_id omitted"
    );
}

#[test]
fn test_required_with_double_option_main_has_value_other_clearing() {
    // Scenario: country_id has value, area_id is clearing
    // Expected: VALID - area_id is clearing (no value) so country_id not required
    let instance = TestRequiredWithDoubleOption {
        country_id: Some(Some(229)),
        area_id: Some(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: country_id has value, area_id is clearing"
    );
}

// ---- Required without with Option<Option<T>> ---- //

#[derive(Validate, Debug)]
struct TestRequiredWithoutDoubleOption {
    // main_field is required when backup_field has NO value
    #[validate(required_without(other_fields("backup_field")))]
    main_field: Option<Option<String>>,

    backup_field: Option<Option<String>>,
}

#[test]
fn test_required_without_double_option_both_have_values() {
    // Scenario: Both fields have values
    // Expected: VALID - backup_field has value, so main_field not required (but allowed)
    let instance = TestRequiredWithoutDoubleOption {
        main_field: Some(Some("main".to_string())),
        backup_field: Some(Some("backup".to_string())),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both have values"
    );
}

#[test]
fn test_required_without_double_option_backup_omitted_main_has_value() {
    // Scenario: backup_field omitted (None), main_field has value
    // Expected: VALID - main_field is required and has value
    let instance = TestRequiredWithoutDoubleOption {
        main_field: Some(Some("main".to_string())),
        backup_field: None,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: backup omitted, main has value"
    );
}

#[test]
fn test_required_without_double_option_backup_omitted_main_missing() {
    // Scenario: backup_field omitted (None), main_field also omitted
    // Expected: INVALID - backup has no value, so main is required
    let instance = TestRequiredWithoutDoubleOption {
        main_field: None,
        backup_field: None,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: backup omitted, main also omitted"
    );
}

#[test]
fn test_required_without_double_option_backup_clearing_main_has_value() {
    // Scenario: backup_field is clearing (Some(None)), main_field has value
    // Expected: VALID - main_field is required (backup has no value) and has value
    let instance = TestRequiredWithoutDoubleOption {
        main_field: Some(Some("main".to_string())),
        backup_field: Some(None), // Clearing = no value
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: backup clearing, main has value"
    );
}

#[test]
fn test_required_without_double_option_backup_clearing_main_missing() {
    // Scenario: backup_field is clearing (Some(None)), main_field omitted
    // Expected: INVALID - backup has no value (clearing), so main is required but missing
    let instance = TestRequiredWithoutDoubleOption {
        main_field: None,
        backup_field: Some(None), // Clearing = no value
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: backup clearing (no value), main omitted"
    );
}

#[test]
fn test_required_without_double_option_backup_has_value_main_missing() {
    // Scenario: backup_field has value, main_field omitted
    // Expected: VALID - backup has value, so main is not required
    let instance = TestRequiredWithoutDoubleOption {
        main_field: None,
        backup_field: Some(Some("backup".to_string())),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: backup has value, main not required"
    );
}

// ---- Basic required with Option<Option<T>> ---- //

#[derive(Validate, Debug)]
struct TestRequiredDoubleOption {
    #[validate(required)]
    value: Option<Option<String>>,
}

#[test]
fn test_required_double_option_has_value() {
    // Scenario: Field has a value (Some(Some(_)))
    // Expected: VALID
    let instance = TestRequiredDoubleOption {
        value: Some(Some("hello".to_string())),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: value is Some(Some(...))"
    );
}

#[test]
fn test_required_double_option_omitted() {
    // Scenario: Field is omitted (None)
    // Expected: INVALID - required field has no value
    let instance = TestRequiredDoubleOption { value: None };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: value is None");
}

#[test]
fn test_required_double_option_clearing() {
    // Scenario: Field is clearing (Some(None))
    // Expected: INVALID - clearing is not the same as having a value
    let instance = TestRequiredDoubleOption { value: Some(None) };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: value is Some(None) (clearing, not a value)"
    );
}
