//! Provides the `ValidateConsent` trait and implementations for common types
//! to determine if a value represents an "accepted" or "declined" state.
//! This is typically used for validating fields like terms of service agreement,
//! newsletter opt-ins, etc.

/// A trait for types that can be validated as representing consent.
///
/// This trait allows various types (like `bool`, `String`, `&str`) to be
/// interpreted as either an "accepted" or "declined" state.
pub trait ValidateConsent {
    /// Checks if the value represents an "accepted" state.
    /// By default, this is true if `as_bool()` is true.
    fn accepted(&self) -> bool {
        self.as_bool()
    }

    /// Checks if the value represents a "declined" state.
    /// By default, this is true if `as_bool()` is false.
    fn declined(&self) -> bool {
        !self.as_bool()
    }

    /// Converts the value to a boolean representation of consent.
    /// `true` generally means accepted, `false` generally means declined.
    fn as_bool(&self) -> bool;
}

/// Implementation of `ValidateConsent` for `bool`.
/// - `true` is considered "accepted".
/// - `false` is considered "declined".
impl ValidateConsent for bool {
    fn as_bool(&self) -> bool {
        *self
    }
}

/// Implementation of `ValidateConsent` for `String`.
/// Recognizes "true", "1", "on", "yes" (case-insensitive) as "accepted".
/// All other string values are considered "declined".
impl ValidateConsent for String {
    fn as_bool(&self) -> bool {
        ["true", "1", "on", "yes"].contains(&self.to_ascii_lowercase().as_str())
    }
}

/// Implementation of `ValidateConsent` for `&str`.
/// Recognizes "true", "1", "on", "yes" (case-insensitive) as "accepted".
/// All other string slice values are considered "declined".
impl ValidateConsent for &str {
    fn as_bool(&self) -> bool {
        ["true", "1", "on", "yes"].contains(&self.to_ascii_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_consent_bool() {
        assert!(true.accepted(), "Expected 'true' to be accepted");
        assert!(!true.declined(), "Expected 'true' not to be declined");

        assert!(false.declined(), "Expected 'false' to be declined");
        assert!(!false.accepted(), "Expected 'false' not to be accepted");
    }

    #[test]
    fn test_validate_consent_string_accepted() {
        for val_str in ["true", "TRUE", "1", "on", "ON", "yes", "YES"] {
            let s = val_str.to_string();
            assert!(s.accepted(), "Expected string '{}' to be accepted", s);
            assert!(!s.declined(), "Expected string '{}' not to be declined", s);
        }
    }

    #[test]
    fn test_validate_consent_string_declined() {
        let declined_values = vec![
            "false",
            "FALSE",
            "0",
            "off",
            "OFF",
            "no",
            "NO",
            "anything_else",
        ];
        for val_str in declined_values {
            let s = val_str.to_string();
            assert!(!s.accepted(), "Expected string '{}' not to be accepted", s);
            assert!(s.declined(), "Expected string '{}' to be declined", s);
        }
    }

    #[test]
    fn test_validate_consent_str_accepted() {
        for val_str in ["true", "TRUE", "1", "on", "ON", "yes", "YES"] {
            assert!(val_str.accepted(), "Expected '{}' to be accepted", val_str);
            assert!(
                !val_str.declined(),
                "Expected '{}' not to be declined",
                val_str
            );
        }
    }

    #[test]
    fn test_validate_consent_str_declined() {
        let declined_values = vec![
            "false",
            "FALSE",
            "0",
            "off",
            "OFF",
            "no",
            "NO",
            "random_string",
        ];
        for val_str in declined_values {
            assert!(
                !val_str.accepted(),
                "Expected '{}' not to be accepted",
                val_str
            );
            assert!(val_str.declined(), "Expected '{}' to be declined", val_str);
        }
    }

    #[test]
    fn test_empty_string_declined() {
        let empty_string = "".to_string();
        assert!(
            !empty_string.accepted(),
            "Expected empty String not to be accepted"
        );
        assert!(
            empty_string.declined(),
            "Expected empty String to be declined"
        );

        assert!(!"".accepted(), "Expected empty &str not to be accepted");
        assert!("".declined(), "Expected empty &str to be declined");
    }
}
