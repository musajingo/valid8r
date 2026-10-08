use validator::Validate;

#[derive(Validate)]
struct TestAcceptedBool {
    #[validate(accepted)]
    terms_agreed: bool,
}

#[test]
fn test_consent_bool_true_is_valid() {
    let s = TestAcceptedBool { terms_agreed: true };
    assert!(
        s.validate().is_ok(),
        "Validation should pass when terms_agreed is true for 'accepted'"
    );
}

#[test]
fn test_consent_bool_false_is_invalid() {
    let s = TestAcceptedBool {
        terms_agreed: false,
    };
    let res = s.validate();
    assert!(
        res.is_err(),
        "Validation should fail when terms_agreed is false for 'accepted'"
    );
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(
        errs.contains_key("terms_agreed"),
        "Error details should contain 'terms_agreed' field"
    );
    assert_eq!(
        errs["terms_agreed"].len(),
        1,
        "There should be one error for 'terms_agreed'"
    );
    assert_eq!(
        errs["terms_agreed"][0].code, "accepted",
        "The error code for 'terms_agreed' should be 'accepted'"
    );
}

#[derive(Validate)]
struct TestAcceptedString {
    #[validate(accepted)]
    permission_given: String,
}

#[test]
fn test_consent_string_accepted_values_are_valid() {
    let accepted_values = ["true", "TRUE", "1", "on", "ON", "yes", "YES"];
    for val_str in accepted_values {
        let s = TestAcceptedString {
            permission_given: val_str.to_string(),
        };
        assert!(
            s.validate().is_ok(),
            "Expected string '{}' to be valid for 'accepted'",
            val_str
        );
    }
}

#[test]
fn test_consent_string_declined_values_are_invalid() {
    let declined_values = [
        "false",
        "FALSE",
        "0",
        "off",
        "OFF",
        "no",
        "NO",
        "anything_else",
        "",
    ];
    for val_str in declined_values {
        let s = TestAcceptedString {
            permission_given: val_str.to_string(),
        };
        let res = s.validate();
        assert!(
            res.is_err(),
            "Expected string '{}' to be invalid for 'accepted'",
            val_str
        );
        let err = res.unwrap_err();
        let errs = err.field_errors();
        assert!(
            errs.contains_key("permission_given"),
            "Field 'permission_given' not in errors for value '{}'",
            val_str
        );
        assert_eq!(
            errs["permission_given"].len(),
            1,
            "Incorrect number of errors for value '{}'",
            val_str
        );
        assert_eq!(
            errs["permission_given"][0].code, "accepted",
            "Incorrect error code for value '{}'",
            val_str
        );
    }
}

#[derive(Validate)]
struct TestAcceptedStr<'a> {
    #[validate(accepted)]
    approval: &'a str,
}

#[test]
fn test_consent_str_accepted_values_are_valid() {
    let accepted_values = ["true", "TRUE", "1", "on", "ON", "yes", "YES"];
    for val_str in accepted_values {
        let s = TestAcceptedStr { approval: val_str };
        assert!(
            s.validate().is_ok(),
            "Expected &str '{}' to be valid for 'accepted'",
            val_str
        );
    }
}

#[test]
fn test_consent_str_declined_values_are_invalid() {
    let declined_values = [
        "false",
        "FALSE",
        "0",
        "off",
        "OFF",
        "no",
        "NO",
        "random_text",
        "",
    ];
    for val_str in declined_values {
        let s = TestAcceptedStr { approval: val_str };
        let res = s.validate();
        assert!(
            res.is_err(),
            "Expected &str '{}' to be invalid for 'accepted'",
            val_str
        );
        let err = res.unwrap_err();
        let errs = err.field_errors();
        assert!(
            errs.contains_key("approval"),
            "Field 'approval' not in errors for value '{}'",
            val_str
        );
        assert_eq!(
            errs["approval"].len(),
            1,
            "Incorrect number of errors for value '{}'",
            val_str
        );
        assert_eq!(
            errs["approval"][0].code, "accepted",
            "Incorrect error code for value '{}'",
            val_str
        );
    }
}

// --- Tests for #[validate(accepted_if(path = ...))] ---

fn accepted_if_bool_condition(s: &TestAcceptedIfBoolConditional) -> bool {
    s.should_condition_pass
}

#[derive(Validate)]
struct TestAcceptedIfBoolConditional {
    #[validate(accepted_if(function = "accepted_if_bool_condition"))]
    consent_given: Option<bool>,
    should_condition_pass: bool,
}

#[test]
fn test_accepted_if_bool_conditional_logic() {
    // consent=true, condition=true -> VALID
    let s1 = TestAcceptedIfBoolConditional {
        consent_given: Some(true),
        should_condition_pass: true,
    };
    assert!(
        s1.validate().is_ok(),
        "expected accepted_if (Option<bool>): consent=true, condition=true -> to pass"
    );

    // consent=true, condition=false -> INVALID
    let s2 = TestAcceptedIfBoolConditional {
        consent_given: Some(true),
        should_condition_pass: false,
    };

    assert!(
        s2.validate().is_ok(),
        "expected accepted_if (Option<bool>): consent=true, condition=false -> to pass"
    );

    // consent=false, condition=true -> INVALID
    let s3 = TestAcceptedIfBoolConditional {
        consent_given: Some(false),
        should_condition_pass: true,
    };
    assert!(
        s3.validate().is_err(),
        "accepted_if (Option<bool>): consent=false, condition=true -> INVALID"
    );
    assert_eq!(
        s3.validate().unwrap_err().field_errors()["consent_given"][0].code,
        "accepted_if",
        "Error code should be 'accepted' for failed accepted_if (Option<bool>)"
    );

    // consent=false, condition=false -> INVALID
    let s4 = TestAcceptedIfBoolConditional {
        consent_given: Some(false),
        should_condition_pass: false,
    };
    assert!(
        s4.validate().is_ok(),
        "expected accepted_if (Option<bool>): consent=false, condition=false -> to pass"
    );
}

fn accepted_if_string_condition(s: &TestAcceptedIfStringConditional) -> bool {
    s.feature_is_active
}

#[derive(Validate)]
struct TestAcceptedIfStringConditional {
    #[validate(accepted_if(function = "accepted_if_string_condition"))]
    agreement: String,
    feature_is_active: bool,
}

#[test]
fn test_accepted_if_string_conditional_logic() {
    // consent="yes", condition=true -> VALID
    let s1 = TestAcceptedIfStringConditional {
        agreement: "yes".to_string(),
        feature_is_active: true,
    };
    assert!(
        s1.validate().is_ok(),
        "accepted_if (String): consent='yes', condition=true -> VALID"
    );

    // consent="true", condition=false -> INVALID
    let s2 = TestAcceptedIfStringConditional {
        agreement: "true".to_string(),
        feature_is_active: false,
    };
    assert!(
        s2.validate().is_ok(),
        "expected accepted_if (String): consent='true', condition=false -> to pass"
    );

    // consent="declined", condition=true -> INVALID
    let s3 = TestAcceptedIfStringConditional {
        agreement: "declined".to_string(),
        feature_is_active: true,
    };
    assert!(
        s3.validate().is_err(),
        "accepted_if (String): consent='declined', condition=true -> INVALID"
    );
    assert_eq!(
        s3.validate().unwrap_err().field_errors()["agreement"][0].code,
        "accepted_if"
    );
}

// --- Tests for #[validate(declined_if(function = ...))] ---

fn declined_if_bool_condition(s: &TestDeclinedIfBoolConditional) -> bool {
    s.must_opt_out_condition
}

#[derive(Validate)]
struct TestDeclinedIfBoolConditional {
    #[validate(declined_if(function = "declined_if_bool_condition"))]
    opt_out_confirmed: Option<bool>, // true means "accepted newsletter", false means "declined newsletter"
    must_opt_out_condition: bool,
}

#[test]
fn test_declined_if_bool_conditional_logic() {
    // declined (opt_out=false), condition=true -> VALID
    let s1 = TestDeclinedIfBoolConditional {
        opt_out_confirmed: Some(false),
        must_opt_out_condition: true,
    };
    assert!(
        s1.validate().is_ok(),
        "declined_if (Option<bool>): declined=true, condition=true -> VALID"
    );

    // declined (opt_out=false), condition=false -> INVALID
    let s2 = TestDeclinedIfBoolConditional {
        opt_out_confirmed: Some(false),
        must_opt_out_condition: false,
    };
    assert!(
        s2.validate().is_ok(),
        "expected declined_if (Option<bool>): declined=true, condition=false -> to pass"
    );

    // accepted (opt_out=true), condition=true -> INVALID
    let s3 = TestDeclinedIfBoolConditional {
        opt_out_confirmed: Some(true),
        must_opt_out_condition: true,
    };
    assert!(
        s3.validate().is_err(),
        "declined_if (Option<bool>): declined=false, condition=true -> INVALID"
    );
    assert_eq!(
        s3.validate().unwrap_err().field_errors()["opt_out_confirmed"][0].code,
        "declined_if"
    );

    // accepted (opt_out=true), condition=false -> INVALID
    let s4 = TestDeclinedIfBoolConditional {
        opt_out_confirmed: Some(true),
        must_opt_out_condition: false,
    };
    assert!(
        s4.validate().is_ok(),
        "expected declined_if (Option<bool>): declined=false, condition=false -> to pass"
    );
}

fn declined_if_string_condition(s: &TestDeclinedIfStringConditional) -> bool {
    s.action_is_restricted
}

#[derive(Validate)]
struct TestDeclinedIfStringConditional {
    #[validate(declined_if(function = "declined_if_string_condition"))]
    action_consent: String, // "no", "0" etc. means declined
    action_is_restricted: bool,
}

#[test]
fn test_declined_if_string_conditional_logic() {
    // consent="no" (declined), condition=true -> VALID
    let s1 = TestDeclinedIfStringConditional {
        action_consent: "no".to_string(),
        action_is_restricted: true,
    };
    assert!(
        s1.validate().is_ok(),
        "declined_if (String): consent='no', condition=true -> VALID"
    );

    // consent="false" (declined), condition=false -> INVALID
    let s2 = TestDeclinedIfStringConditional {
        action_consent: "false".to_string(),
        action_is_restricted: false,
    };
    assert!(
        s2.validate().is_ok(),
        "expected declined_if (String): consent='false', condition=false -> to pass"
    );

    // consent="yes" (accepted), condition=true -> INVALID
    let s3 = TestDeclinedIfStringConditional {
        action_consent: "yes".to_string(),
        action_is_restricted: true,
    };
    assert!(
        s3.validate().is_err(),
        "declined_if (String): consent='yes', condition=true -> INVALID"
    );
    assert_eq!(
        s3.validate().unwrap_err().field_errors()["action_consent"][0].code,
        "declined_if"
    );

    // consent="anything_else" (declined), condition=true -> VALID
    let s4 = TestDeclinedIfStringConditional {
        action_consent: "anything_else".to_string(),
        action_is_restricted: true,
    };
    assert!(
        s4.validate().is_ok(),
        "declined_if (String): consent='anything_else', condition=true -> VALID"
    );

    // consent="anything_else" (declined), condition=false -> INVALID
    let s5 = TestDeclinedIfStringConditional {
        action_consent: "anything_else".to_string(),
        action_is_restricted: false,
    };
    assert!(
        s5.validate().is_ok(),
        "expected declined_if (String): consent='anything_else', condition=false -> to pass"
    );
}
