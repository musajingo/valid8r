//! Provides validation for checking if a value contains a specific substring or key.
//!
//! This module defines the `ValidateContains` trait and implements it for common types
//! like `String`, `&str`, `Option`, `Cow`, and `HashMap`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// A trait for types that can be checked for the presence of a substring or key.
pub trait ValidateContains {
    /// Checks if the implementing type contains the given `needle`.
    ///
    /// # Arguments
    ///
    /// * `needle`: A string slice representing the substring or key to search for.
    ///
    /// # Returns
    ///
    /// * `true` if the `needle` is found, `false` otherwise.
    fn validate_contains(&self, needle: &str) -> bool;
}

/// Implements `ValidateContains` for `String`.
impl ValidateContains for String {
    fn validate_contains(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}

/// Implements `ValidateContains` for `Option<T>` where `T` itself implements `ValidateContains`.
/// If the `Option` is `None`, it's considered to "contain" the needle (returns `true`),
/// as there's nothing to check against. This behavior might be surprising and
/// could be interpreted as "does not fail the contains check".
impl<T> ValidateContains for Option<T>
where
    T: ValidateContains,
{
    fn validate_contains(&self, needle: &str) -> bool {
        if let Some(v) = self {
            v.validate_contains(needle)
        } else {
            // `None` is considered to pass the "contains" validation.
            true
        }
    }
}

/// Implements `ValidateContains` for references `&T` where `T` implements `ValidateContains`.
impl<T> ValidateContains for &T
where
    T: ?Sized + ValidateContains,
{
    fn validate_contains(&self, needle: &str) -> bool {
        T::validate_contains(self, needle)
    }
}

/// Implements `ValidateContains` for `Cow<'_, T>` where `&'a T` implements `ValidateContains`.
impl<T> ValidateContains for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    for<'a> &'a T: ValidateContains,
{
    fn validate_contains(&self, needle: &str) -> bool {
        self.as_ref().validate_contains(needle)
    }
}

/// Implements `ValidateContains` for string slices `&str`.
impl ValidateContains for &str {
    fn validate_contains(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}

/// Implements `ValidateContains` for `HashMap<String, S, H>`, checking for key presence.
impl<S, H: BuildHasher> ValidateContains for HashMap<String, S, H> {
    fn validate_contains(&self, needle: &str) -> bool {
        self.contains_key(needle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_contains_string() {
        assert!("hey".validate_contains("e"));
    }

    #[test]
    fn test_validate_contains_string_can_fail() {
        assert!(!"hey".validate_contains("o"));
    }

    #[test]
    fn test_validate_contains_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(map.validate_contains("hey"));
    }

    #[test]
    fn test_validate_contains_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(!map.validate_contains("bob"));
    }

    #[test]
    fn test_validate_contains_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(test.validate_contains("e"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(test.validate_contains("e"));
    }

    #[test]
    fn test_validate_contains_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(!test.validate_contains("o"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(!test.validate_contains("o"));
    }
}
