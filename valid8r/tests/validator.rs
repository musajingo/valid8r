use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use valid8r::*;

// Static Regex for password validation: checks for at least one digit.
static PASSWORD_HAS_DIGIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]").unwrap());
// Static Regex for password validation: checks for at least one uppercase letter.
static PASSWORD_HAS_UPPERCASE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Z]").unwrap());

// The data structure to be validated. (Represents data for a new user registration)
#[derive(Debug)]
struct NewUser {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
    age: Option<u8>,
    referral_code: String,
}

impl Validate for NewUser {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if !self.username.validate_length(Some(3), Some(20), None) {
            // Validate username length (3-20 characters).
            let mut err = ValidationError::new("username_length");
            err.add_param(Cow::from("min"), &3);
            err.add_param(Cow::from("max"), &20);
            errors.add("username", err);
        }

        if !self.username.validate_non_control_character() {
            errors.add(
                // Validate username for non-control characters.
                "username",
                ValidationError::new("username_non_control_char"),
            );
        }

        if !self.email.validate_email() {
            errors.add("email", ValidationError::new("email_invalid"));
            // Validate email format.
        }

        if !self.password.validate_length(Some(8), None, None) {
            errors.add("password", ValidationError::new("password_length_min"));
            // Validate password minimum length (8 characters).
        }
        if !self.password.validate_regex(&*PASSWORD_HAS_DIGIT_RE) {
            errors.add("password", ValidationError::new("password_no_digit"));
            // Validate password for at least one digit.
        }
        if !self.password.validate_regex(&*PASSWORD_HAS_UPPERCASE_RE) {
            errors.add("password", ValidationError::new("password_no_uppercase"));
            // Validate password for at least one uppercase letter.
        }

        if !validate_must_match(&self.password, &self.confirm_password) {
            errors.add(
                // Validate that password and confirm_password match.
                "confirm_password",
                ValidationError::new("password_mismatch"),
            );
        }

        if let Some(age_val) = self.age {
            // Validate age range (18-99) if provided.
            if !age_val.validate_range(Some(18), Some(99), None, None) {
                errors.add("age", ValidationError::new("age_range_invalid"));
            }
        }

        if !self.referral_code.validate_does_not_contain("INVALID") {
            errors.add(
                // Validate referral code does not contain "INVALID".
                "referral_code",
                ValidationError::new("referral_invalid_substring"),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[test]
/// Tests a valid `NewUser` instance.
fn test_new_user_valid() {
    let user = NewUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
        age: Some(25),
        referral_code: "VALIDCODE".to_string(),
    };
    assert!(
        user.validate().is_ok(),
        "Expected valid user to pass validation"
    );
}

#[test]
/// Tests `NewUser` with an invalid email and an out-of-range age.
fn test_new_user_invalid_email_and_age() {
    let user = NewUser {
        username: "testuser".to_string(),
        email: "testexample.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
        age: Some(17),
        referral_code: "VALIDCODE".to_string(),
    };
    let result = user.validate();
    assert!(
        result.is_err(),
        "Expected invalid user data to fail validation"
    );
    let errs = result.unwrap_err();
    assert!(
        errs.errors().contains_key("email"),
        "Expected an error for the email field"
    );
    assert!(
        errs.errors().contains_key("age"),
        "Expected an error for the age field"
    );
    assert_eq!(
        errs.field_errors().get(&Cow::from("email")).unwrap()[0].code,
        "email_invalid",
        "Email error code mismatch"
    );
    assert_eq!(
        errs.field_errors().get(&Cow::from("age")).unwrap()[0].code,
        "age_range_invalid",
        "Age error code mismatch"
    );
}

#[test]
/// Tests `NewUser` with an invalid referral code.
fn test_new_user_invalid_referral_code() {
    let user = NewUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "Password123".to_string(),
        confirm_password: "Password123".to_string(),
        age: Some(30),
        referral_code: "SOMEINVALIDCODE".to_string(),
    };
    let result = user.validate();
    assert!(
        result.is_err(),
        "Expected invalid referral code to fail validation"
    );
    let errs = result.unwrap_err();
    assert!(
        errs.errors().contains_key("referral_code"),
        "Expected an error for referral_code field"
    );
}

#[derive(Debug)]
/// Represents payment details for a transaction.
struct PaymentDetails {
    #[cfg(feature = "cards")]
    card_number: String,
    card_holder_name: Option<String>,
    billing_address_ip: String,
    #[cfg(feature = "url")]
    website_url: Option<String>,
}

impl Validate for PaymentDetails {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        #[cfg(feature = "cards")]
        if !self.card_number.validate_credit_card() {
            // Validate credit card number format and checksum.
            errors.add("card_number", ValidationError::new("cc_invalid"));
        }

        if !self.card_holder_name.validate_required() {
            // Validate card holder name is present.
            errors.add("card_holder_name", ValidationError::new("required"));
        } else if let Some(name) = &self.card_holder_name {
            // Validate card holder name length if present.
            if !name.validate_length(Some(2), Some(100), None) {
                errors.add("card_holder_name", ValidationError::new("name_length"));
            }
        }

        if !self.billing_address_ip.validate_ip() {
            // Validate billing address IP format.
            errors.add("billing_address_ip", ValidationError::new("ip_invalid"));
        }

        #[cfg(feature = "url")]
        if let Some(url) = &self.website_url {
            // Validate website URL format if present.
            if !url.validate_url() {
                errors.add("website_url", ValidationError::new("url_invalid"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[test]
/// Tests a valid `PaymentDetails` instance.
fn test_payment_details_valid() {
    let details = PaymentDetails {
        #[cfg(feature = "cards")]
        card_number: "4539571147647251".to_string(), // Visa, valid
        card_holder_name: Some("Test Holder".to_string()),
        billing_address_ip: "192.168.1.1".to_string(),
        #[cfg(feature = "url")]
        website_url: Some("https://example.com".to_string()),
    };
    assert!(
        details.validate().is_ok(),
        "Expected valid payment details to pass validation"
    );
}

#[derive(Debug)]
/// Represents a contact method, which can be an email or phone.
struct ContactMethod {
    method_type: String,
    value: String,
}

impl Validate for ContactMethod {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        match self.method_type.as_str() {
            "email" => {
                // If method type is email, validate the value as an email.
                if !self.value.validate_email() {
                    errors.add("value", ValidationError::new("email_invalid"));
                }
            }
            "phone" => {
                // If method type is phone, validate the value as a phone number.
                #[cfg(feature = "phone_number")]
                if !self.value.validate_phone_number(None) {
                    errors.add("value", ValidationError::new("phone_invalid"));
                }
            }
            _ => {
                // If method type is unknown, add an error.
                errors.add("method_type", ValidationError::new("unknown_type"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
/// Represents a user's profile information.
struct UserProfile {
    user_id: Option<String>,
    contact_methods: Vec<ContactMethod>,
    preferences: HashMap<String, String>,
}

impl Validate for UserProfile {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut overall_errors = ValidationErrors::new();

        if !self.user_id.validate_required() {
            // Validate that user_id is present.
            overall_errors.add("user_id", ValidationError::new("required"));
        }

        // Validate Vec<ContactMethod>
        // The blanket impl for Vec<T> where T: Validate will produce
        // errors under "_tmp_validator" key if any ContactMethod is invalid.
        // merge_self handles this by extracting the List and putting it under "contact_methods".
        overall_errors.merge_self("contact_methods", self.contact_methods.validate());

        let mut failing_prefs = Vec::new();
        for (key, value) in &self.preferences {
            // Validate that each preference value contains "pref_".
            if !value.validate_contains("pref_") {
                failing_prefs.push(key.clone());
            }
        }
        if !failing_prefs.is_empty() {
            let mut err = ValidationError::new("invalid_preferences_format");
            err.add_param(Cow::from("failing_keys"), &failing_prefs);
            overall_errors.add("preferences", err);
        }

        if overall_errors.is_empty() {
            Ok(())
        } else {
            Err(overall_errors)
        }
    }
}

#[test]
/// Tests a valid `UserProfile` instance.
fn test_user_profile_valid() {
    let profile = UserProfile {
        user_id: Some("user123".to_string()),
        contact_methods: vec![
            ContactMethod {
                method_type: "email".to_string(),
                value: "test@example.com".to_string(),
            },
            ContactMethod {
                method_type: "phone".to_string(),
                value: "+12025550104".to_string(),
            },
        ],
        preferences: HashMap::from([("theme".to_string(), "dark_pref_mode".to_string())]),
    };
    assert!(
        profile.validate().is_ok(),
        "Expected valid user profile to pass validation"
    );
}

#[test]
/// Tests `UserProfile` with invalid contact methods and preferences.
fn test_user_profile_invalid_contact_and_preference() {
    let profile = UserProfile {
        user_id: Some("user123".to_string()),
        contact_methods: vec![
            ContactMethod {
                method_type: "email".to_string(),
                value: "test@example.com".to_string(),
            },
            ContactMethod {
                method_type: "phone".to_string(),
                value: "invalidphone".to_string(),
            }, // Invalid phone number
            ContactMethod {
                method_type: "fax".to_string(),
                value: "123".to_string(),
            }, // Invalid contact method type
        ],
        preferences: HashMap::from([
            ("theme".to_string(), "dark_mode".to_string()), // Invalid, missing "pref_"
            ("lang".to_string(), "en_pref_US".to_string()),
        ]),
    };
    let result = profile.validate();
    assert!(result.is_err());
    let errs = result.unwrap_err();

    assert!(errs.errors().contains_key("contact_methods"));
    assert!(errs.errors().contains_key("preferences"));

    if let Some(ValidationErrorsKind::List(list_errors)) = errs.errors().get("contact_methods") {
        assert!(list_errors.contains_key(&2)); // Third contact method (fax)

        // The second contact method (phone) is only validated when the
        // `phone_number` feature is enabled.
        #[cfg(feature = "phone_number")]
        {
            assert!(list_errors.contains_key(&1)); // Second contact method (phone)
            if let Some(phone_err_kind) = list_errors.get(&1) {
                // phone_err_kind is Box<ValidationErrors>
                assert!(phone_err_kind.errors().contains_key("value"));
            } else {
                panic!("Expected error for contact_methods[1]");
            }
        }
    } else {
        panic!("Expected List errors for contact_methods");
    }
}
