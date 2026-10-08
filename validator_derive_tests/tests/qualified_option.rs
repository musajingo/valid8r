//! Fully qualified `Option` paths (`std::option::Option`, `core::option::Option`)
//! must unwrap exactly like the bare `Option` spelling, including when nested
//! and when the inner value is handed to a custom validator.

use validator::{Validate, ValidationError};

fn not_admin(name: &str) -> Result<(), ValidationError> {
    if name == "admin" {
        return Err(ValidationError::new("reserved_name"));
    }
    Ok(())
}

// ============================================================================
// std::option::Option nested twice
// ============================================================================

#[derive(Validate, Debug)]
struct StdQualified {
    #[validate(custom(function = not_admin))]
    name: std::option::Option<std::option::Option<String>>,
}

#[test]
fn std_qualified_nested_option_reaches_custom_validator() {
    let bad = StdQualified {
        name: Some(Some("admin".to_string())),
    };
    let errors = bad.validate().unwrap_err();
    assert_eq!(errors.field_errors()["name"][0].code, "reserved_name");

    let good = StdQualified {
        name: Some(Some("bob".to_string())),
    };
    assert!(good.validate().is_ok());
}

#[test]
fn std_qualified_nested_option_skips_when_absent_or_null() {
    assert!(StdQualified { name: None }.validate().is_ok());
    assert!(StdQualified { name: Some(None) }.validate().is_ok());
}

// ============================================================================
// core::option::Option nested twice
// ============================================================================

#[derive(Validate, Debug)]
struct CoreQualified {
    #[validate(custom(function = not_admin))]
    name: core::option::Option<core::option::Option<String>>,
}

#[test]
fn core_qualified_nested_option_reaches_custom_validator() {
    let bad = CoreQualified {
        name: Some(Some("admin".to_string())),
    };
    assert!(bad.validate().is_err());

    let good = CoreQualified {
        name: Some(Some("bob".to_string())),
    };
    assert!(good.validate().is_ok());
}

// ============================================================================
// Mixed qualified/unqualified nesting
// ============================================================================

#[derive(Validate, Debug)]
struct MixedQualified {
    #[validate(custom(function = not_admin))]
    name: Option<std::option::Option<String>>,
}

#[test]
fn mixed_qualified_nested_option_reaches_custom_validator() {
    let bad = MixedQualified {
        name: Some(Some("admin".to_string())),
    };
    assert!(bad.validate().is_err());

    let good = MixedQualified {
        name: Some(Some("bob".to_string())),
    };
    assert!(good.validate().is_ok());
    assert!(MixedQualified { name: Some(None) }.validate().is_ok());
}

// ============================================================================
// Qualified single Option with a built-in validator
// ============================================================================

#[derive(Validate, Debug)]
struct StdQualifiedLength {
    #[validate(length(min = 3))]
    name: std::option::Option<String>,
}

#[test]
fn std_qualified_single_option_runs_builtin_validator() {
    let bad = StdQualifiedLength {
        name: Some("ab".to_string()),
    };
    let errors = bad.validate().unwrap_err();
    assert_eq!(errors.field_errors()["name"][0].code, "length");

    assert!(StdQualifiedLength { name: None }.validate().is_ok());
}
