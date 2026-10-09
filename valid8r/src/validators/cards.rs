//! This module defines a trait `ValidateCreditCard` that types representing
//! credit card numbers can implement to easily validate their format and checksum
//! using the `card-validate` crate.

use card_validate::Validate as CardValidate;
use std::borrow::Cow;

/// A trait for types that can be validated as a credit card number.
///
/// Implementors must provide a way to get the credit card number as a string slice
/// or owned string via the `as_credit_card_string` method.
pub trait ValidateCreditCard {
    /// Validates the credit card number string using the `card-validate` crate.
    ///
    /// This is a default implementation that calls `as_credit_card_string`
    /// and then uses `card_validate::Validate::from` to perform the validation.
    fn validate_credit_card(&self) -> bool {
        let card_string = self.as_credit_card_string();
        CardValidate::from(&card_string).is_ok()
    }

    /// Returns the credit card number as a `Cow<str>`.
    fn as_credit_card_string(&self) -> Cow<'_, str>;
}

/// Implements `ValidateCreditCard` for any type that can be converted to a string slice (`AsRef<str>`).
impl<T: AsRef<str>> ValidateCreditCard for T {
    fn as_credit_card_string(&self) -> Cow<'_, str> {
        Cow::from(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::ValidateCreditCard;
    use std::borrow::Cow;

    #[test]
    fn test_credit_card() {
        let tests = vec![
            ("4539571147647251", true),
            ("343380440754432", true),
            ("zduhefljsdfKJKJZHUI", false),
            ("5236313877109141", false),
        ];

        // Iterate through test cases and assert the expected validation result.
        for (input, expected) in tests {
            assert_eq!(input.validate_credit_card(), expected);
        }
    }

    #[test]
    fn test_credit_card_cow() {
        let test: Cow<'static, str> = "4539571147647251".into();
        assert!(test.validate_credit_card());
        let test: Cow<'static, str> = String::from("4539571147647251").into();
        assert!(test.validate_credit_card());
        let test: Cow<'static, str> = "5236313877109141".into();
        assert!(!test.validate_credit_card());
        let test: Cow<'static, str> = String::from("5236313877109141").into();
        assert!(!test.validate_credit_card());
    }
}
