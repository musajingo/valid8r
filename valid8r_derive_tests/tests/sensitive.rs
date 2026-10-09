//! Tests for `#[validate(sensitive)]`: a sensitive field's submitted value must
//! never appear in validation error params — neither as its own `value` param
//! nor as the `other` param of a `must_match` error on a different field.

use valid8r::Validate;

// ============================================================================
// Basic redaction: the field's own value is stripped from its errors
// ============================================================================

#[derive(Validate, Debug)]
struct BasicSensitive {
    #[validate(sensitive, length(min = 8))]
    password: String,
}

#[test]
fn sensitive_field_value_is_stripped_from_own_errors() {
    let instance = BasicSensitive {
        password: "short".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["password"][0];
    assert_eq!(err.code, "length");
    assert!(
        !err.params.contains_key("value"),
        "sensitive field leaked its own value: {:?}",
        err.params
    );
}

// ============================================================================
// must_match: neither field sensitive — both params present (baseline)
// ============================================================================

#[derive(Validate, Debug)]
struct MatchNeitherSensitive {
    nickname: String,

    #[validate(must_match(other = "nickname"))]
    nickname_confirmation: String,
}

#[test]
fn must_match_without_sensitive_keeps_both_params() {
    let instance = MatchNeitherSensitive {
        nickname: "bob".to_string(),
        nickname_confirmation: "bobby".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["nickname_confirmation"][0];
    assert_eq!(err.code, "must_match");
    assert_eq!(err.params["value"], "bobby");
    assert_eq!(err.params["other"], "bob");
}

// ============================================================================
// must_match: only the referenced (other) field sensitive
// ============================================================================

#[derive(Validate, Debug)]
struct MatchOtherSensitive {
    #[validate(sensitive)]
    password: String,

    #[validate(must_match(other = "password"))]
    confirmation: String,
}

#[test]
fn must_match_omits_other_param_when_referenced_field_is_sensitive() {
    let instance = MatchOtherSensitive {
        password: "secret".to_string(),
        confirmation: "wrong".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["confirmation"][0];
    assert_eq!(err.code, "must_match");
    assert!(
        !err.params.contains_key("other"),
        "sensitive password leaked through 'other' param: {:?}",
        err.params
    );
    // The confirmation itself is not sensitive, so its own value stays.
    assert_eq!(err.params["value"], "wrong");
}

// ============================================================================
// must_match: only the validated field sensitive
// ============================================================================

#[derive(Validate, Debug)]
struct MatchMainSensitive {
    nickname: String,

    #[validate(sensitive, must_match(other = "nickname"))]
    secret_alias: String,
}

#[test]
fn must_match_omits_value_param_when_validated_field_is_sensitive() {
    let instance = MatchMainSensitive {
        nickname: "bob".to_string(),
        secret_alias: "hidden".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["secret_alias"][0];
    assert_eq!(err.code, "must_match");
    assert!(
        !err.params.contains_key("value"),
        "sensitive field leaked its own value: {:?}",
        err.params
    );
    // The nickname is not sensitive, so the comparison target stays.
    assert_eq!(err.params["other"], "bob");
}

// ============================================================================
// must_match: both fields sensitive
// ============================================================================

#[derive(Validate, Debug)]
struct MatchBothSensitive {
    #[validate(sensitive)]
    password: String,

    #[validate(sensitive, must_match(other = "password"))]
    confirmation: String,
}

#[test]
fn must_match_omits_both_params_when_both_fields_are_sensitive() {
    let instance = MatchBothSensitive {
        password: "secret".to_string(),
        confirmation: "wrong".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["confirmation"][0];
    assert_eq!(err.code, "must_match");
    assert!(
        !err.params.contains_key("value"),
        "sensitive confirmation leaked: {:?}",
        err.params
    );
    assert!(
        !err.params.contains_key("other"),
        "sensitive password leaked through 'other' param: {:?}",
        err.params
    );
}

// ============================================================================
// must_match: explicit `sensitive = false` is NOT sensitive
// ============================================================================

#[derive(Validate, Debug)]
struct MatchExplicitlyNotSensitive {
    #[validate(sensitive = false)]
    nickname: String,

    #[validate(must_match(other = "nickname"))]
    nickname_confirmation: String,
}

#[test]
fn must_match_keeps_other_param_when_sensitive_is_explicitly_false() {
    let instance = MatchExplicitlyNotSensitive {
        nickname: "bob".to_string(),
        nickname_confirmation: "bobby".to_string(),
    };
    let errors = instance.validate().unwrap_err();
    let err = &errors.field_errors()["nickname_confirmation"][0];
    assert_eq!(err.code, "must_match");
    assert_eq!(err.params["other"], "bob");
    assert_eq!(err.params["value"], "bobby");
}
