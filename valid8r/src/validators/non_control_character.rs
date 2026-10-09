//! Provides validation for ensuring a string does not contain control characters.
//!
//! This module defines the `ValidateNonControlCharacter` trait, which can be implemented
//! by types that represent strings or can be converted to string iterators.

/// A trait for types that can be validated to ensure they do not contain control characters.
pub trait ValidateNonControlCharacter {
    /// Checks if all characters in the implementing type are non-control characters.
    ///
    /// # Returns
    ///
    /// * `true` if no control characters are found.
    /// * `false` if at least one control character is present.
    fn validate_non_control_character(&self) -> bool {
        // Iterates over characters and checks if any are control characters.
        self.as_non_control_character_iterator()
            .all(|code| !code.is_control())
    }

    /// Returns an iterator over the characters of the implementing type.
    ///
    /// This method is used by the default implementation of `validate_non_control_character`.
    fn as_non_control_character_iterator(&self) -> Box<dyn Iterator<Item = char> + '_>;
}

/// Implements `ValidateNonControlCharacter` for any type `T` that implements `AsRef<str>`.
/// This allows types like `String`, `&str`, and `Cow<str>` to use this validation.
impl<T: AsRef<str>> ValidateNonControlCharacter for T {
    fn as_non_control_character_iterator(&self) -> Box<dyn Iterator<Item = char> + '_> {
        // Converts the type to a string slice and returns an iterator over its characters.
        Box::new(self.as_ref().chars())
    }
}

#[cfg(test)]
mod tests {
    use super::ValidateNonControlCharacter;
    use std::borrow::Cow;

    #[test]
    /// Tests various strings for the presence of control characters.
    fn test_non_control_character() {
        let tests = vec![
            ("Himmel", true),
            ("आकाश", true),
            ("வானத்தில்", true),
            ("하늘", true),
            ("небо", true),
            ("2H₂ + O₂ ⇌ 2H₂O", true),
            ("\u{000c}", false),
            ("\u{009F}", false),
        ];

        // Iterate through test cases and assert the expected validation result.
        for (input, expected) in tests {
            assert_eq!(input.validate_non_control_character(), expected);
        }
    }

    #[test]
    /// Tests `ValidateNonControlCharacter` specifically with `Cow<str>` types.
    fn test_non_control_character_cow() {
        let test: Cow<'static, str> = "आकाश".into();
        assert!(test.validate_non_control_character());
        let test: Cow<'static, str> = String::from("வானத்தில்").into();
        assert!(test.validate_non_control_character());
        let test: Cow<'static, str> = "\u{000c}".into();
        assert!(!test.validate_non_control_character());
        let test: Cow<'static, str> = String::from("\u{009F}").into();
        assert!(!test.validate_non_control_character());
    }
}
