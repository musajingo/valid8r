use validator::Validate;

#[derive(Debug, Validate)]
struct TestCountrySpecific {
    #[validate(phone_number(country = "UG"))]
    phone_number: String,
}

#[test]
fn test_valid_country_specific_phone_number() {
    let s = TestCountrySpecific {
        phone_number: "+256 762 646177".to_string(),
    };
    assert!(s.validate().is_ok(), "Valid UG phone number should pass");

    let s2 = TestCountrySpecific {
        phone_number: "0762646177".to_string(), // National format for UG
    };
    assert!(
        s2.validate().is_ok(),
        "Valid national UG phone number should pass"
    );
}

#[test]
fn test_invalid_country_specific_phone_number() {
    let s = TestCountrySpecific {
        phone_number: "555-123-4567".to_string(), // US number (without country code)), not UG
    };
    let res = s.validate();
    assert!(
        res.is_err(),
        "US phone number should fail for UG country validation"
    );
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("phone_number"));
    assert_eq!(errs["phone_number"][0].code, "phone_number");
}

#[test]
fn test_invalid_format_for_country_specific() {
    let s = TestCountrySpecific {
        phone_number: "12345".to_string(), // Invalid format
    };
    assert!(s.validate().is_err(), "Invalid format should fail");
}

#[derive(Debug, Validate)]
struct TestInternational {
    #[validate(phone_number)] // No country hint, relies on international format
    phone_number: String,
}

#[test]
fn test_valid_international_phone_number() {
    let s = TestInternational {
        phone_number: "+44 20 7946 0958".to_string(), // UK
    };
    assert!(
        s.validate().is_ok(),
        "Valid international UK phone number should pass"
    );

    let s2 = TestInternational {
        phone_number: "+16502530000".to_string(), // US
    };
    assert!(
        s2.validate().is_ok(),
        "Valid international US phone number should pass"
    );
}

#[test]
fn test_invalid_international_phone_number() {
    let s = TestInternational {
        phone_number: "02079460958".to_string(), // National UK, but no country hint
    };
    assert!(
        s.validate().is_err(),
        "National UK phone number without hint should fail"
    );

    let s2 = TestInternational {
        phone_number: "12345".to_string(), // Invalid format
    };
    assert!(s2.validate().is_err(), "Invalid format should fail");
}

#[derive(Debug, Validate)]
struct TestCustomError {
    #[validate(phone_number(
        country = "US",
        code = "custom_phone",
        message = "Invalid US phone."
    ))]
    us_phone: String,
}

#[test]
fn test_custom_error_message_and_code() {
    let s = TestCustomError {
        us_phone: "0762646177".to_string(), // Not a US number
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("us_phone"));
    assert_eq!(errs["us_phone"].len(), 1);
    assert_eq!(errs["us_phone"][0].code, "custom_phone");
    assert_eq!(
        errs["us_phone"][0].clone().message.unwrap(),
        "Invalid US phone."
    );
}

#[derive(Debug, Validate)]
struct TestOptionalPhone {
    #[validate(phone_number(country = "GB"))]
    phone: Option<String>,
}

#[test]
fn test_optional_phone_number() {
    let s_none = TestOptionalPhone { phone: None };
    assert!(s_none.validate().is_ok(), "None optional phone should pass");

    let s_valid = TestOptionalPhone {
        phone: Some("+447911123456".to_string()),
    };
    assert!(
        s_valid.validate().is_ok(),
        "Valid optional phone should pass"
    );

    let s_invalid = TestOptionalPhone {
        phone: Some("123".to_string()),
    };
    assert!(
        s_invalid.validate().is_err(),
        "Invalid optional phone should fail"
    );
}

#[test]
fn nullable_phone_accepts_empty_string() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone_number(nullable, message = "phone must be valid"))]
        phone: String,
    }

    let s = TestStruct {
        phone: "".to_string(),
    };
    assert!(s.validate().is_ok());
}

#[test]
fn nullable_phone_accepts_valid_phone() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone_number(nullable, message = "phone must be valid"))]
        phone: String,
    }

    let s = TestStruct {
        phone: "+16502530000".to_string(),
    };
    assert!(s.validate().is_ok());
}

#[test]
fn nullable_phone_rejects_invalid_phone() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone_number(nullable, message = "phone must be valid"))]
        phone: String,
    }

    let s = TestStruct {
        phone: "invalid".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("phone"));
    assert_eq!(errs["phone"].len(), 1);
    assert_eq!(errs["phone"][0].code, "phone_number");
}

#[test]
fn non_nullable_phone_rejects_empty_string() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(phone_number(message = "phone must be valid"))]
        phone: String,
    }

    let s = TestStruct {
        phone: "".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("phone"));
    assert_eq!(errs["phone"].len(), 1);
    assert_eq!(errs["phone"][0].code, "phone_number");
}
