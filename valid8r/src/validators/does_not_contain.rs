//! Provides validation for checking if a value does *not* contain a specific substring or key.
//!
//! This module defines the `ValidateDoesNotContain` trait. It leverages the existing
//! `ValidateContains` trait by simply negating its result. This allows any type
//! that implements `ValidateContains` to automatically support `ValidateDoesNotContain`.

use crate::ValidateContains;

/// A trait for types that can be checked for the absence of a substring or key.
pub trait ValidateDoesNotContain {
    /// Checks if the implementing type does *not* contain the given `needle`.
    ///
    /// # Arguments
    ///
    /// * `needle`: A string slice representing the substring or key to check for absence.
    ///
    /// # Returns
    ///
    /// * `true` if the `needle` is *not* found, `false` if it is found.
    fn validate_does_not_contain(&self, needle: &str) -> bool;
}

/// Implements `ValidateDoesNotContain` for any type `T` that already implements `ValidateContains`.
impl<T> ValidateDoesNotContain for T
where
    T: ValidateContains,
{
    fn validate_does_not_contain(&self, needle: &str) -> bool {
        // The core logic: simply negate the result of `validate_contains`.
        !self.validate_contains(needle)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_validate_does_not_contain_string() {
        assert!("hey".validate_does_not_contain("g"));
    }

    #[test]
    fn test_validate_does_not_contain_string_can_fail() {
        assert!(!"hey".validate_does_not_contain("e"));
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(map.validate_does_not_contain("bob"));
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(!map.validate_does_not_contain("hey"));
    }

    #[test]
    fn test_validate_does_not_contain_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(test.validate_does_not_contain("b"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(test.validate_does_not_contain("b"));
    }

    #[test]
    fn test_validate_does_not_contain_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(!test.validate_does_not_contain("e"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(!test.validate_does_not_contain("e"));
    }
}
