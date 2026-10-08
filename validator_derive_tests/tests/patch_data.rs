//! Tests for `PatchData<T>` validation support.
//!
//! `PatchData<T>` is a wrapper type that provides PATCH semantics:
//! - `PatchData(None)` = field omitted (absent)
//! - `PatchData(Some(None))` = field explicitly set to null
//! - `PatchData(Some(Some(value)))` = field has a value
//!
//! The validation macro treats `PatchData<T>` as equivalent to `Option<Option<T>>`,
//! meaning "has value" = `Some(Some(_))` only.

use patch_data::PatchData;
use validator::Validate;

// ============================================================================
// Basic Required Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestRequiredPatchData {
    #[validate(required)]
    value: PatchData<String>,
}

#[test]
fn test_required_patch_data_absent_fails() {
    // PatchData(None) = absent, should fail required validation
    let instance = TestRequiredPatchData {
        value: PatchData(None),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: PatchData(None) means absent, required validation should fail"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["value"][0].code, "required");
}

#[test]
fn test_required_patch_data_null_fails() {
    // PatchData(Some(None)) = explicitly null, should fail required validation
    let instance = TestRequiredPatchData {
        value: PatchData(Some(None)),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: PatchData(Some(None)) means null, required validation should fail"
    );
    let errors = result.unwrap_err();
    assert_eq!(errors.field_errors()["value"][0].code, "required");
}

#[test]
fn test_required_patch_data_has_value_passes() {
    // PatchData(Some(Some(value))) = has value, should pass required validation
    let instance = TestRequiredPatchData {
        value: PatchData(Some(Some("hello".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: PatchData(Some(Some(_))) has a value"
    );
}

// ============================================================================
// Required With - PatchData<T> as the required field
// ============================================================================

#[derive(Validate, Debug)]
struct TestRequiredWithPatchData {
    #[validate(required_with(other_fields("other")))]
    main_field: PatchData<i32>,

    other: PatchData<i32>,
}

#[test]
fn test_required_with_patch_data_both_absent() {
    // Both fields absent - VALID (other has no value, main not required)
    let instance = TestRequiredWithPatchData {
        main_field: PatchData(None),
        other: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: other is absent, main_field not required"
    );
}

#[test]
fn test_required_with_patch_data_other_null() {
    // Other field is null (clearing) - VALID (null is not "has value")
    let instance = TestRequiredWithPatchData {
        main_field: PatchData(None),
        other: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: other is Some(None) (clearing), main_field not required"
    );
}

#[test]
fn test_required_with_patch_data_other_has_value_main_absent() {
    // Other has value, main absent - INVALID
    let instance = TestRequiredWithPatchData {
        main_field: PatchData(None),
        other: PatchData(Some(Some(123))),
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
fn test_required_with_patch_data_other_has_value_main_null() {
    // Other has value, main is null - INVALID (null is not "has value")
    let instance = TestRequiredWithPatchData {
        main_field: PatchData(Some(None)),
        other: PatchData(Some(Some(123))),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: other has value, but main_field is Some(None) (not a value)"
    );
}

#[test]
fn test_required_with_patch_data_both_have_values() {
    // Both have values - VALID
    let instance = TestRequiredWithPatchData {
        main_field: PatchData(Some(Some(456))),
        other: PatchData(Some(Some(123))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields have values"
    );
}

// ============================================================================
// Length Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestLengthPatchData {
    #[validate(length(min = 3, max = 10))]
    text: PatchData<String>,
}

#[test]
fn test_length_patch_data_absent_skips_validation() {
    // Absent field should skip validation
    let instance = TestLengthPatchData {
        text: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips length validation"
    );
}

#[test]
fn test_length_patch_data_null_skips_validation() {
    // Null field should skip validation
    let instance = TestLengthPatchData {
        text: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips length validation"
    );
}

#[test]
fn test_length_patch_data_value_too_short() {
    // Value too short - INVALID
    let instance = TestLengthPatchData {
        text: PatchData(Some(Some("ab".to_string()))),
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
fn test_length_patch_data_value_valid() {
    // Valid length - VALID
    let instance = TestLengthPatchData {
        text: PatchData(Some(Some("hello".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'hello' has valid length"
    );
}

#[test]
fn test_length_patch_data_value_too_long() {
    // Value too long - INVALID
    let instance = TestLengthPatchData {
        text: PatchData(Some(Some("this is way too long".to_string()))),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: value exceeds max length 10"
    );
}

// ============================================================================
// Email Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestEmailPatchData {
    #[validate(email)]
    email: PatchData<String>,
}

#[test]
fn test_email_patch_data_absent_skips_validation() {
    let instance = TestEmailPatchData {
        email: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips email validation"
    );
}

#[test]
fn test_email_patch_data_null_skips_validation() {
    let instance = TestEmailPatchData {
        email: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips email validation"
    );
}

#[test]
fn test_email_patch_data_invalid_email() {
    let instance = TestEmailPatchData {
        email: PatchData(Some(Some("not-an-email".to_string()))),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-an-email' is not a valid email"
    );
}

#[test]
fn test_email_patch_data_valid_email() {
    let instance = TestEmailPatchData {
        email: PatchData(Some(Some("test@example.com".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'test@example.com' is a valid email"
    );
}

// ============================================================================
// Required If with PatchData<T>
// ============================================================================

fn is_online_update(data: &TestRequiredIfPatchData) -> bool {
    // Check if is_online is being set to true
    matches!(*data.is_online, Some(Some(true)))
}

#[derive(Validate, Debug)]
struct TestRequiredIfPatchData {
    #[validate(required_if(func = "is_online_update"))]
    website: PatchData<String>,

    is_online: PatchData<bool>,
}

#[test]
fn test_required_if_patch_data_condition_false() {
    // is_online not set to true, website not required
    let instance = TestRequiredIfPatchData {
        website: PatchData(None),
        is_online: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: is_online not true, website not required"
    );
}

#[test]
fn test_required_if_patch_data_condition_true_field_missing() {
    // is_online set to true, website required but missing
    let instance = TestRequiredIfPatchData {
        website: PatchData(None),
        is_online: PatchData(Some(Some(true))),
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
fn test_required_if_patch_data_condition_true_field_present() {
    // is_online set to true, website has value
    let instance = TestRequiredIfPatchData {
        website: PatchData(Some(Some("https://example.com".to_string()))),
        is_online: PatchData(Some(Some(true))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: is_online is true and website has value"
    );
}

// ============================================================================
// Range Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestRangePatchData {
    #[validate(range(min = 1, max = 100))]
    quantity: PatchData<i32>,
}

#[test]
fn test_range_patch_data_absent_skips() {
    let instance = TestRangePatchData {
        quantity: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips range validation"
    );
}

#[test]
fn test_range_patch_data_null_skips() {
    let instance = TestRangePatchData {
        quantity: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips range validation"
    );
}

#[test]
fn test_range_patch_data_value_below_min() {
    let instance = TestRangePatchData {
        quantity: PatchData(Some(Some(0))),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 0 is below min 1");
}

#[test]
fn test_range_patch_data_value_in_range() {
    let instance = TestRangePatchData {
        quantity: PatchData(Some(Some(50))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 50 is within range [1, 100]"
    );
}

#[test]
fn test_range_patch_data_value_above_max() {
    let instance = TestRangePatchData {
        quantity: PatchData(Some(Some(101))),
    };
    let result = instance.validate();
    assert!(result.is_err(), "Expected INVALID: 101 is above max 100");
}

// ============================================================================
// Mixed PatchData and Option<Option<T>> in same struct
// ============================================================================

#[derive(Validate, Debug)]
struct TestMixedPatchTypes {
    #[validate(required_with(other_fields("legacy_field")))]
    new_field: PatchData<String>,

    legacy_field: Option<Option<String>>,
}

#[test]
fn test_mixed_types_legacy_has_value_new_required() {
    // legacy_field has value (Option<Option<T>>), new_field required
    let instance = TestMixedPatchTypes {
        new_field: PatchData(None),
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
        new_field: PatchData(Some(Some("new".to_string()))),
        legacy_field: Some(Some("legacy".to_string())),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: both fields have values"
    );
}

// ============================================================================
// URL Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestUrlPatchData {
    #[validate(url)]
    website: PatchData<String>,
}

#[test]
fn test_url_patch_data_absent_skips_validation() {
    let instance = TestUrlPatchData {
        website: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips url validation"
    );
}

#[test]
fn test_url_patch_data_null_skips_validation() {
    let instance = TestUrlPatchData {
        website: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips url validation"
    );
}

#[test]
fn test_url_patch_data_invalid_url() {
    let instance = TestUrlPatchData {
        website: PatchData(Some(Some("not-a-url".to_string()))),
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
fn test_url_patch_data_valid_url() {
    let instance = TestUrlPatchData {
        website: PatchData(Some(Some("https://example.com".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'https://example.com' is a valid URL"
    );
}

// ============================================================================
// URL with nullable flag and PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestUrlNullablePatchData {
    #[validate(url(nullable))]
    website: PatchData<String>,
}

#[test]
fn test_url_nullable_patch_data_empty_string_valid() {
    // With nullable flag, empty strings are allowed (will be converted to NULL)
    let instance = TestUrlNullablePatchData {
        website: PatchData(Some(Some("".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: empty string with nullable url is valid"
    );
}

#[test]
fn test_url_nullable_patch_data_invalid_url() {
    let instance = TestUrlNullablePatchData {
        website: PatchData(Some(Some("not-a-url".to_string()))),
    };
    let result = instance.validate();
    assert!(
        result.is_err(),
        "Expected INVALID: 'not-a-url' is still invalid even with nullable"
    );
}

// ============================================================================
// Phone Number Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestPhonePatchData {
    #[validate(phone_number)]
    phone: PatchData<String>,
}

#[test]
fn test_phone_patch_data_absent_skips_validation() {
    let instance = TestPhonePatchData {
        phone: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips phone validation"
    );
}

#[test]
fn test_phone_patch_data_null_skips_validation() {
    let instance = TestPhonePatchData {
        phone: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips phone validation"
    );
}

#[test]
fn test_phone_patch_data_invalid_phone() {
    let instance = TestPhonePatchData {
        phone: PatchData(Some(Some("not-a-phone".to_string()))),
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
fn test_phone_patch_data_valid_phone() {
    // Use E.164 format: +[country code][area code][subscriber number]
    let instance = TestPhonePatchData {
        phone: PatchData(Some(Some("+14155552671".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: '+14155552671' is a valid phone number"
    );
}

// ============================================================================
// Contains Validation with PatchData<T>
// ============================================================================

#[derive(Validate, Debug)]
struct TestContainsPatchData {
    #[validate(contains(pattern = "@"))]
    contact: PatchData<String>,
}

#[test]
fn test_contains_patch_data_absent_skips_validation() {
    let instance = TestContainsPatchData {
        contact: PatchData(None),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: absent field skips contains validation"
    );
}

#[test]
fn test_contains_patch_data_null_skips_validation() {
    let instance = TestContainsPatchData {
        contact: PatchData(Some(None)),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: null field skips contains validation"
    );
}

#[test]
fn test_contains_patch_data_missing_pattern() {
    let instance = TestContainsPatchData {
        contact: PatchData(Some(Some("no at sign".to_string()))),
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
fn test_contains_patch_data_has_pattern() {
    let instance = TestContainsPatchData {
        contact: PatchData(Some(Some("user@example.com".to_string()))),
    };
    assert!(
        instance.validate().is_ok(),
        "Expected VALID: 'user@example.com' contains '@'"
    );
}
