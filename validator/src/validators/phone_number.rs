//! This module defines the `ValidatePhoneNumber` trait, allowing various string-like
//! types to be validated as phone numbers.

use phonenumber::country;
use std::{borrow::Cow, str::FromStr};

/// A trait for types that can be validated as a phone number.
///
/// Implementors must provide a way to get the phone number as a string
/// via the `as_phone_number_string` method.
pub trait ValidatePhoneNumber {
    /// Validates the phone number string using the `phonenumber` crate.
    ///
    /// # Arguments
    ///
    /// * `country_id_str`: An optional `phonenumber::country::Id` as a string.
    ///
    /// # Returns
    /// `true` if the phone number is valid for the given country (or if inferable and valid), `false` otherwise.
    fn validate_phone_number(&self, country_id_str: Option<&str>) -> bool {
        let country_id: Option<country::Id> = match country_id_str {
            Some(str_id) => match country::Id::from_str(str_id) {
                Ok(id) => Some(id),
                Err(_) => {
                    return false;
                }
            },
            None => None,
        };

        let number_str = self.as_phone_number_string();

        match phonenumber::parse(country_id, number_str) {
            Ok(parsed_number) => parsed_number.is_valid(),
            Err(_) => false, // Parsing the number itself failed.
        }
    }

    /// Returns the phone number as a `Cow<str>`.
    fn as_phone_number_string(&self) -> Cow<'_, str>;
}

/// Implements `ValidatePhoneNumber` for any type that can be converted to a string slice (`AsRef<str>`).
impl<T: AsRef<str>> ValidatePhoneNumber for T {
    fn as_phone_number_string(&self) -> Cow<'_, str> {
        Cow::from(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use string literals for country codes in tests for this trait method,
    // as that's what the method accepts.

    #[test]
    fn test_validate_phone_number_international() {
        assert!("+447911123456".validate_phone_number(None)); // Valid UK mobile
        assert!("+12025550104".validate_phone_number(None)); // Valid US number
        assert!(!"+1202555010".validate_phone_number(None)); // Invalid (too short)
        assert!(!"invalid_number".validate_phone_number(None)); // Invalid format
    }

    #[test]
    fn test_validate_phone_number_with_country_hint() {
        assert!("2025550104".validate_phone_number(Some("US"))); // Valid US number with US hint
        assert!("07911123456".validate_phone_number(Some("GB"))); // Valid UK mobile with UK hint
        assert!(!"202555010".validate_phone_number(Some("US"))); // Invalid US number with US hint
        assert!(!"07911123456".validate_phone_number(Some("US"))); // UK number with US hint (should be invalid for US)
        assert!(!"2025550104".validate_phone_number(Some("XX"))); // Invalid country code string
    }

    #[test]
    fn test_validate_phone_number_string_type() {
        let s = String::from("+4917612345678"); // Valid German mobile
        assert!(s.validate_phone_number(None));
    }

    #[test]
    fn test_validate_phone_number_cow_str() {
        let cow_borrowed: Cow<str> = "+33123456789".into(); // Valid French number
        assert!(cow_borrowed.validate_phone_number(None));

        let s = String::from("+33123456789");
        let cow_owned: Cow<str> = s.into();
        assert!(cow_owned.validate_phone_number(None));
    }
}
