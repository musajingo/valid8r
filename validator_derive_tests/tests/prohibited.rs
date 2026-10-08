use validator::Validate;

// ---- Prohibited If ---- //

fn should_prohibit_setting(s: &TestOptionProhibitedIf) -> bool {
    s.is_restricted_account
}

#[derive(Validate, Debug)]
struct TestOptionProhibitedIf {
    #[validate(prohibited_if(func = "should_prohibit_setting"))]
    advanced_settings: Option<String>,
    is_restricted_account: bool,
}

#[test]
fn test_prohibited_if_field_is_none_and_condition_is_true() {
    // Scenario: Field is None, Condition is true (prohibition active)
    // Expected: VALID (field is correctly absent)
    let instance = TestOptionProhibitedIf {
        advanced_settings: None,
        is_restricted_account: true,
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_if_field_is_some_and_condition_is_true() {
    // Scenario: Field is Some, Condition is true (prohibition active)
    // Expected: INVALID (field should be absent)
    let instance = TestOptionProhibitedIf {
        advanced_settings: Some("secret_setting".to_string()),
        is_restricted_account: true,
    };
    let result = instance.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().field_errors()["advanced_settings"][0].code,
        "prohibited_if"
    );
}

#[test]
fn test_prohibited_if_field_is_none_and_condition_is_false() {
    // Scenario: Field is None, Condition is false (prohibition inactive)
    // Expected: VALID
    let instance = TestOptionProhibitedIf {
        advanced_settings: None,
        is_restricted_account: false,
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_if_field_is_some_and_condition_is_false() {
    // Scenario: Field is Some, Condition is false (prohibition inactive)
    // Expected: VALID
    let instance = TestOptionProhibitedIf {
        advanced_settings: Some("admin_setting".to_string()),
        is_restricted_account: false,
    };
    assert!(instance.validate().is_ok());
}

// ---- Prohibited With ---- //

#[derive(Validate, Debug)]
struct TestOptionProhibitedWith {
    #[validate(prohibited_with(other_fields("other_field_1", "other_field_2")))]
    main_field: Option<String>,
    other_field_1: Option<String>,
    other_field_2: Option<String>,
}

#[test]
fn test_prohibited_with_main_field_is_none_always_valid() {
    // Scenario: main_field is None.
    // Expected: VALID (field is correctly absent, condition for prohibition doesn't matter for this state)
    let instance = TestOptionProhibitedWith {
        main_field: None,
        other_field_1: Some("value".to_string()),
        other_field_2: None,
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_with_main_field_is_some_and_one_other_is_some() {
    // Scenario: main_field is Some, and other_field_1 is Some (condition for prohibition met)
    // Expected: INVALID
    let instance = TestOptionProhibitedWith {
        main_field: Some("problem".to_string()),
        other_field_1: Some("value".to_string()),
        other_field_2: None,
    };
    let result = instance.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().field_errors()["main_field"][0].code,
        "prohibited_with"
    );
}

#[test]
fn test_prohibited_with_main_field_is_some_and_no_others_are_some() {
    // Scenario: main_field is Some, but no other fields are Some (condition for prohibition NOT met)
    // Expected: VALID
    let instance = TestOptionProhibitedWith {
        main_field: Some("allowed".to_string()),
        other_field_1: None,
        other_field_2: None,
    };
    assert!(instance.validate().is_ok());
}

// ---- Prohibited With All ---- //

#[derive(Validate, Debug)]
struct TestOptionProhibitedWithAll {
    #[validate(prohibited_with_all(other_fields("other_field_a", "other_field_b")))]
    main_field: Option<String>,
    other_field_a: Option<String>,
    other_field_b: Option<String>,
}

#[test]
fn test_prohibited_with_all_main_field_is_none_always_valid() {
    let instance = TestOptionProhibitedWithAll {
        main_field: None,
        other_field_a: Some("value".to_string()),
        other_field_b: Some("value".to_string()),
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_with_all_main_field_is_some_and_all_others_are_some() {
    // Scenario: main_field is Some, and ALL other fields are Some (condition for prohibition met)
    // Expected: INVALID
    let instance = TestOptionProhibitedWithAll {
        main_field: Some("problem".to_string()),
        other_field_a: Some("value_a".to_string()),
        other_field_b: Some("value_b".to_string()),
    };
    let result = instance.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().field_errors()["main_field"][0].code,
        "prohibited_with_all"
    );
}

#[test]
fn test_prohibited_with_all_main_field_is_some_and_not_all_others_are_some() {
    // Scenario: main_field is Some, but NOT ALL other fields are Some (condition for prohibition NOT met)
    // Expected: VALID
    let instance = TestOptionProhibitedWithAll {
        main_field: Some("allowed".to_string()),
        other_field_a: Some("value_a".to_string()),
        other_field_b: None,
    };
    assert!(instance.validate().is_ok());
}

// ---- Prohibited Without ---- //

#[derive(Validate, Debug)]
struct TestOptionProhibitedWithout {
    #[validate(prohibited_without(other_fields("other_field_x", "other_field_y")))]
    main_field: Option<String>,
    other_field_x: Option<String>,
    other_field_y: Option<String>,
}

#[test]
fn test_prohibited_without_main_field_is_none_always_valid() {
    let instance = TestOptionProhibitedWithout {
        main_field: None,
        other_field_x: None,
        other_field_y: Some("value".to_string()),
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_without_main_field_is_some_and_one_other_is_none() {
    // Scenario: main_field is Some, and other_field_x is None (condition for prohibition met)
    // Expected: INVALID
    let instance = TestOptionProhibitedWithout {
        main_field: Some("problem".to_string()),
        other_field_x: None,
        other_field_y: Some("value_y".to_string()),
    };
    let result = instance.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().field_errors()["main_field"][0].code,
        "prohibited_without"
    );
}

#[test]
fn test_prohibited_without_main_field_is_some_and_no_others_are_none() {
    // Scenario: main_field is Some, but all other fields are Some (condition for prohibition NOT met)
    // Expected: VALID
    let instance = TestOptionProhibitedWithout {
        main_field: Some("allowed".to_string()),
        other_field_x: Some("value_x".to_string()),
        other_field_y: Some("value_y".to_string()),
    };
    assert!(instance.validate().is_ok());
}

// ---- Prohibited Without All ---- //

#[derive(Validate, Debug)]
struct TestOptionProhibitedWithoutAll {
    #[validate(prohibited_without_all(other_fields("other_field_c", "other_field_d")))]
    main_field: Option<String>,
    other_field_c: Option<String>,
    other_field_d: Option<String>,
}

#[test]
fn test_prohibited_without_all_main_field_is_none_always_valid() {
    let instance = TestOptionProhibitedWithoutAll {
        main_field: None,
        other_field_c: None,
        other_field_d: None,
    };
    assert!(instance.validate().is_ok());
}

#[test]
fn test_prohibited_without_all_main_field_is_some_and_all_others_are_none() {
    // Scenario: main_field is Some, and ALL other fields are None (condition for prohibition met)
    // Expected: INVALID
    let instance = TestOptionProhibitedWithoutAll {
        main_field: Some("problem".to_string()),
        other_field_c: None,
        other_field_d: None,
    };
    let result = instance.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().field_errors()["main_field"][0].code,
        "prohibited_without_all"
    );
}

#[test]
fn test_prohibited_without_all_main_field_is_some_and_not_all_others_are_none() {
    // Scenario: main_field is Some, but NOT ALL other fields are None (condition for prohibition NOT met)
    // Expected: VALID
    let instance = TestOptionProhibitedWithoutAll {
        main_field: Some("allowed".to_string()),
        other_field_c: None,
        other_field_d: Some("value_d".to_string()),
    };
    assert!(instance.validate().is_ok());
}
