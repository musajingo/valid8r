//! Tests for `Delta<T>` validation support.
//!
//! `Delta<T>` is a three-state enum that provides PATCH semantics:
//! - `Delta::Unchanged` = field omitted from the request (absent)
//! - `Delta::Clear` = field explicitly set to null
//! - `Delta::Set(value)` = field has a value
//!
//! The validation macro treats only `Delta::Set(_)` as "has a value":
//! value validators run on the inner value of `Set` and are skipped for
//! `Unchanged` and `Clear`, and presence validators (`required*`,
//! `prohibited*`) count `Unchanged` and `Clear` both as absent.

use delta::Delta;
use valid8r::Validate;

// ============================================================================
// Basic Required Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestRequiredDelta {
    #[validate(required)]
    value: Delta<String>,
}

#[test]
fn test_required_delta_unchanged_fails() {
    // Unchanged = absent, should fail required validation
    let instance = TestRequiredDelta {
        value: Delta::Unchanged,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: Delta::Unchanged means absent, required validation should fail"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["value"][0].code, "required");
}

#[test]
fn test_required_delta_clear_fails() {
    // Clear = explicitly null, should fail required validation
    let instance = TestRequiredDelta {
        value: Delta::Clear,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: Delta::Clear means null, required validation should fail"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["value"][0].code, "required");
}

#[test]
fn test_required_delta_set_passes() {
    // Set(value) = has value, should pass required validation
    let instance = TestRequiredDelta {
        value: Delta::Set("hello".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: Delta::Set(_) has a value"
    );
}

// ============================================================================
// Required With - Delta<T> as both the required field and the other field
// ============================================================================

#[derive(Validate, Debug)]
struct TestRequiredWithDelta {
    #[validate(required_with(other_fields("other")))]
    main_field: Delta<i32>,

    other: Delta<i32>,
}

#[test]
fn test_required_with_delta_both_unchanged() {
    // Both fields absent - VALID (other has no value, main not required)
    let instance = TestRequiredWithDelta {
        main_field: Delta::Unchanged,
        other: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: other is absent, main_field not required"
    );
}

#[test]
fn test_required_with_delta_other_clear() {
    // Other field is being cleared - VALID (Clear is not "has value")
    let instance = TestRequiredWithDelta {
        main_field: Delta::Unchanged,
        other: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: other is Delta::Clear (clearing), main_field not required"
    );
}

#[test]
fn test_required_with_delta_other_set_main_unchanged() {
    // Other has value, main absent - INVALID
    let instance = TestRequiredWithDelta {
        main_field: Delta::Unchanged,
        other: Delta::Set(123),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: other has value, but main_field is absent"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["main_field"][0].code, "required_with");
}

#[test]
fn test_required_with_delta_other_set_main_clear() {
    // Other has value, main is being cleared - INVALID (Clear is not "has value")
    let instance = TestRequiredWithDelta {
        main_field: Delta::Clear,
        other: Delta::Set(123),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: other has value, but main_field is Delta::Clear (not a value)"
    );
}

#[test]
fn test_required_with_delta_both_set() {
    // Both have values - VALID
    let instance = TestRequiredWithDelta {
        main_field: Delta::Set(456),
        other: Delta::Set(123),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields have values"
    );
}

// ============================================================================
// Prohibited With - Delta<T> as both the prohibited field and the other field
// ============================================================================

#[derive(Validate, Debug)]
struct TestProhibitedWithDelta {
    #[validate(prohibited_with(other_fields("other")))]
    main_field: Delta<String>,

    other: Delta<String>,
}

#[test]
fn test_prohibited_with_delta_other_set_main_set_fails() {
    // Other has value, main also has value - INVALID (mutually exclusive)
    let instance = TestProhibitedWithDelta {
        main_field: Delta::Set("main".to_string()),
        other: Delta::Set("other".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: other has value, main_field must not have one"
    );
    let errors = result.unwrap_err();
    assert_eq!(
        errors.field_errors()["main_field"][0].code,
        "prohibited_with"
    );
}

#[test]
fn test_prohibited_with_delta_other_set_main_clear_passes() {
    // Other has value, main is being cleared - VALID (Clear is not "has value")
    let instance = TestProhibitedWithDelta {
        main_field: Delta::Clear,
        other: Delta::Set("other".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: clearing main_field does not violate the prohibition"
    );
}

#[test]
fn test_prohibited_with_delta_other_unchanged_main_set_passes() {
    // Other absent, main has value - VALID (prohibition not active)
    let instance = TestProhibitedWithDelta {
        main_field: Delta::Set("main".to_string()),
        other: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: other is absent, prohibition not active"
    );
}

// ============================================================================
// Length Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestLengthDelta {
    #[validate(length(min = 3, max = 10))]
    text: Delta<String>,
}

#[test]
fn test_length_delta_unchanged_skips_validation() {
    let instance = TestLengthDelta {
        text: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips length validation"
    );
}

#[test]
fn test_length_delta_clear_skips_validation() {
    let instance = TestLengthDelta { text: Delta::Clear };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips length validation"
    );
}

#[test]
fn test_length_delta_value_too_short() {
    let instance = TestLengthDelta {
        text: Delta::Set("ab".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: value 'ab' is shorter than min length 3"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["text"][0].code, "length");
}

#[test]
fn test_length_delta_value_valid() {
    let instance = TestLengthDelta {
        text: Delta::Set("hello".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'hello' has valid length"
    );
}

#[test]
fn test_length_delta_value_too_long() {
    let instance = TestLengthDelta {
        text: Delta::Set("this is way too long".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: value exceeds max length 10"
    );
}

// ============================================================================
// Email Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestEmailDelta {
    #[validate(email)]
    email: Delta<String>,
}

#[test]
fn test_email_delta_unchanged_skips_validation() {
    let instance = TestEmailDelta {
        email: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips email validation"
    );
}

#[test]
fn test_email_delta_clear_skips_validation() {
    let instance = TestEmailDelta {
        email: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips email validation"
    );
}

#[test]
fn test_email_delta_invalid_email() {
    let instance = TestEmailDelta {
        email: Delta::Set("not-an-email".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-an-email' is not a valid email"
    );
}

#[test]
fn test_email_delta_valid_email() {
    let instance = TestEmailDelta {
        email: Delta::Set("test@example.com".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'test@example.com' is a valid email"
    );
}

// ============================================================================
// Required If with Delta<T>
// ============================================================================

fn is_online_update(data: &TestRequiredIfDelta) -> bool {
    // Check if is_online is being set to true
    matches!(data.is_online, Delta::Set(true))
}

#[derive(Validate, Debug)]
struct TestRequiredIfDelta {
    #[validate(required_if(func = "is_online_update"))]
    website: Delta<String>,

    is_online: Delta<bool>,
}

#[test]
fn test_required_if_delta_condition_false() {
    // is_online not set to true, website not required
    let instance = TestRequiredIfDelta {
        website: Delta::Unchanged,
        is_online: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: is_online not true, website not required"
    );
}

#[test]
fn test_required_if_delta_condition_true_field_missing() {
    // is_online set to true, website required but missing
    let instance = TestRequiredIfDelta {
        website: Delta::Unchanged,
        is_online: Delta::Set(true),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: is_online is true, but website is absent"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["website"][0].code, "required_if");
}

#[test]
fn test_required_if_delta_condition_true_field_present() {
    // is_online set to true, website has value
    let instance = TestRequiredIfDelta {
        website: Delta::Set("https://example.com".to_string()),
        is_online: Delta::Set(true),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: is_online is true and website has value"
    );
}

// ============================================================================
// Range Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestRangeDelta {
    #[validate(range(min = 1, max = 100))]
    quantity: Delta<i32>,
}

#[test]
fn test_range_delta_unchanged_skips() {
    let instance = TestRangeDelta {
        quantity: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips range validation"
    );
}

#[test]
fn test_range_delta_clear_skips() {
    let instance = TestRangeDelta {
        quantity: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips range validation"
    );
}

#[test]
fn test_range_delta_value_below_min() {
    let instance = TestRangeDelta {
        quantity: Delta::Set(0),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 0 is below min 1");
}

#[test]
fn test_range_delta_value_in_range() {
    let instance = TestRangeDelta {
        quantity: Delta::Set(50),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 50 is within range [1, 100]"
    );
}

#[test]
fn test_range_delta_value_above_max() {
    let instance = TestRangeDelta {
        quantity: Delta::Set(101),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 101 is above max 100");
}

// ============================================================================
// Mixed Delta and Option<Option<T>> in same struct
// ============================================================================

#[derive(Validate, Debug)]
struct TestMixedPatchTypes {
    #[validate(required_with(other_fields("legacy_field")))]
    new_field: Delta<String>,

    legacy_field: Option<Option<String>>,
}

#[test]
fn test_mixed_types_legacy_has_value_new_required() {
    // legacy_field has value (Option<Option<T>>), new_field required
    let instance = TestMixedPatchTypes {
        new_field: Delta::Unchanged,
        legacy_field: Some(Some("legacy".to_string())),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: legacy_field has value, new_field required but absent"
    );
}

#[test]
fn test_mixed_types_both_have_values() {
    let instance = TestMixedPatchTypes {
        new_field: Delta::Set("new".to_string()),
        legacy_field: Some(Some("legacy".to_string())),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields have values"
    );
}

// ============================================================================
// URL Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestUrlDelta {
    #[validate(url)]
    website: Delta<String>,
}

#[test]
fn test_url_delta_unchanged_skips_validation() {
    let instance = TestUrlDelta {
        website: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips url validation"
    );
}

#[test]
fn test_url_delta_clear_skips_validation() {
    let instance = TestUrlDelta {
        website: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips url validation"
    );
}

#[test]
fn test_url_delta_invalid_url() {
    let instance = TestUrlDelta {
        website: Delta::Set("not-a-url".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-a-url' is not a valid URL"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["website"][0].code, "url");
}

#[test]
fn test_url_delta_valid_url() {
    let instance = TestUrlDelta {
        website: Delta::Set("https://example.com".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'https://example.com' is a valid URL"
    );
}

// ============================================================================
// URL with nullable flag and Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestUrlNullableDelta {
    #[validate(url(nullable))]
    website: Delta<String>,
}

#[test]
fn test_url_nullable_delta_empty_string_valid() {
    // With nullable flag, empty strings are allowed (will be converted to NULL)
    let instance = TestUrlNullableDelta {
        website: Delta::Set("".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: empty string with nullable url is valid"
    );
}

#[test]
fn test_url_nullable_delta_invalid_url() {
    let instance = TestUrlNullableDelta {
        website: Delta::Set("not-a-url".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-a-url' is still invalid even with nullable"
    );
}

// ============================================================================
// Phone Number Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestPhoneDelta {
    #[validate(phone_number)]
    phone: Delta<String>,
}

#[test]
fn test_phone_delta_unchanged_skips_validation() {
    let instance = TestPhoneDelta {
        phone: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips phone validation"
    );
}

#[test]
fn test_phone_delta_clear_skips_validation() {
    let instance = TestPhoneDelta {
        phone: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips phone validation"
    );
}

#[test]
fn test_phone_delta_invalid_phone() {
    let instance = TestPhoneDelta {
        phone: Delta::Set("not-a-phone".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-a-phone' is not a valid phone number"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["phone"][0].code, "phone_number");
}

#[test]
fn test_phone_delta_valid_phone() {
    // Use E.164 format: +[country code][area code][subscriber number]
    let instance = TestPhoneDelta {
        phone: Delta::Set("+14155552671".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: '+14155552671' is a valid phone number"
    );
}

// ============================================================================
// Contains Validation with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestContainsDelta {
    #[validate(contains(pattern = "@"))]
    contact: Delta<String>,
}

#[test]
fn test_contains_delta_unchanged_skips_validation() {
    let instance = TestContainsDelta {
        contact: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips contains validation"
    );
}

#[test]
fn test_contains_delta_clear_skips_validation() {
    let instance = TestContainsDelta {
        contact: Delta::Clear,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips contains validation"
    );
}

#[test]
fn test_contains_delta_missing_pattern() {
    let instance = TestContainsDelta {
        contact: Delta::Set("no at sign".to_string()),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'no at sign' doesn't contain '@'"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["contact"][0].code, "contains");
}

#[test]
fn test_contains_delta_has_pattern() {
    let instance = TestContainsDelta {
        contact: Delta::Set("user@example.com".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'user@example.com' contains '@'"
    );
}

// ============================================================================
// Custom Validation with Delta<T>
// ============================================================================

fn not_reserved(name: &str) -> Result<(), valid8r::ValidationError> {
    if name == "admin" {
        return Err(valid8r::ValidationError::new("reserved_name"));
    }
    Ok(())
}

#[derive(Validate, Debug)]
struct TestCustomDelta {
    #[validate(custom(function = not_reserved))]
    name: Delta<String>,
}

#[test]
fn test_custom_delta_unchanged_skips_validation() {
    let instance = TestCustomDelta {
        name: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips custom validation"
    );
}

#[test]
fn test_custom_delta_invalid_value() {
    let instance = TestCustomDelta {
        name: Delta::Set("admin".to_string()),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 'admin' is reserved");
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["name"][0].code, "reserved_name");
}

#[test]
fn test_custom_delta_valid_value() {
    let instance = TestCustomDelta {
        name: Delta::Set("musa".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'musa' is not reserved"
    );
}

// ============================================================================
// Custom Validation with numeric Delta<T>
// ============================================================================

// Numeric fields are passed to custom validators by value, same as for
// `i32`, `Option<i32>` and `Option<Option<i32>>` fields.
fn not_zero(quantity: i32) -> Result<(), valid8r::ValidationError> {
    if quantity == 0 {
        return Err(valid8r::ValidationError::new("zero_quantity"));
    }
    Ok(())
}

#[derive(Validate, Debug)]
struct TestCustomNumericDelta {
    #[validate(custom(function = not_zero))]
    quantity: Delta<i32>,
}

#[test]
fn test_custom_numeric_delta_unchanged_skips_validation() {
    let instance = TestCustomNumericDelta {
        quantity: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips custom validation"
    );
}

#[test]
fn test_custom_numeric_delta_invalid_value() {
    let instance = TestCustomNumericDelta {
        quantity: Delta::Set(0),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 0 quantity is rejected");
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["quantity"][0].code, "zero_quantity");
}

#[test]
fn test_custom_numeric_delta_valid_value() {
    let instance = TestCustomNumericDelta {
        quantity: Delta::Set(5),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 5 is a valid quantity"
    );
}

// ============================================================================
// Must Match with Delta<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestMustMatchDelta {
    #[validate(must_match(other = "password_confirmation"))]
    password: Delta<String>,

    password_confirmation: Delta<String>,
}

#[test]
fn test_must_match_delta_both_set_equal_passes() {
    let instance = TestMustMatchDelta {
        password: Delta::Set("secret".to_string()),
        password_confirmation: Delta::Set("secret".to_string()),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields set to the same value"
    );
}

#[test]
fn test_must_match_delta_both_set_different_fails() {
    let instance = TestMustMatchDelta {
        password: Delta::Set("secret".to_string()),
        password_confirmation: Delta::Set("other".to_string()),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: values differ");
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["password"][0].code, "must_match");
}

#[test]
fn test_must_match_delta_set_vs_unchanged_fails() {
    // Setting the field without confirming it is a mismatch
    let instance = TestMustMatchDelta {
        password: Delta::Set("secret".to_string()),
        password_confirmation: Delta::Unchanged,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: password set but confirmation absent"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["password"][0].code, "must_match");
}

#[test]
fn test_must_match_delta_set_vs_clear_fails() {
    let instance = TestMustMatchDelta {
        password: Delta::Set("secret".to_string()),
        password_confirmation: Delta::Clear,
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: password set but confirmation cleared"
    );
}

#[test]
fn test_must_match_delta_unchanged_skips_validation() {
    // Not touching the field means nothing to confirm
    let instance = TestMustMatchDelta {
        password: Delta::Unchanged,
        password_confirmation: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips must_match validation"
    );
}

#[test]
fn test_must_match_delta_clear_skips_validation() {
    // Clearing the field means there is no new value to confirm
    let instance = TestMustMatchDelta {
        password: Delta::Clear,
        password_confirmation: Delta::Unchanged,
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: cleared field skips must_match validation"
    );
}
