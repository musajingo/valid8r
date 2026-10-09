//! Provides validation for URLs.
//!
//! This module defines the `ValidateUrl` trait, which allows various string-like
//! types to be checked if they represent valid URLs using the `url` crate.

use std::{
    borrow::Cow,
    cell::{Ref, RefMut},
    rc::Rc,
    sync::Arc,
};
use url::Url;

/// A trait for types that can be validated as a URL.
pub trait ValidateUrl {
    /// Validates whether the implementing type represents a valid URL.
    ///
    /// This default implementation calls `as_url_string` and then attempts to parse
    /// the result using `url::Url::parse`. If `as_url_string` returns `None`
    /// (e.g., for an `Option::None` value), validation is considered successful.
    ///
    /// # Returns
    /// `true` if the string is a valid URL or if `as_url_string` returns `None`, `false` otherwise.
    fn validate_url(&self) -> bool {
        if let Some(u) = self.as_url_string() {
            // Scheme allow-list: these values render as <a href>/<img src> in
            // admin UIs, so a scheme-agnostic parse would accept javascript:
            // and data: — stored XSS.
            Url::parse(&u).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
        } else {
            // If there's no string to parse (e.g. Option::None), consider it valid.
            true
        }
    }

    /// Returns the string representation of the URL to be validated.
    ///
    /// This method should return `None` if the value itself represents an absence
    /// (e.g., an `Option::None`) and thus shouldn't be validated as a URL string.
    fn as_url_string(&self) -> Option<Cow<'_, str>>;
}

/// Macro to implement `ValidateUrl` for common smart pointer types that dereference to `T: ValidateUrl`.
macro_rules! validate_type_that_derefs {
    ($type_:ty) => {
        impl<T> ValidateUrl for $type_
        where
            T: ValidateUrl,
        {
            fn as_url_string(&self) -> Option<Cow<'_, str>> {
                // Delegates to the inner type's as_url_string implementation.
                T::as_url_string(self)
            }
        }
    };
}

validate_type_that_derefs!(&T);
validate_type_that_derefs!(Arc<T>);
validate_type_that_derefs!(Box<T>);
validate_type_that_derefs!(Rc<T>);
validate_type_that_derefs!(Ref<'_, T>);
validate_type_that_derefs!(RefMut<'_, T>);

/// Macro to implement `ValidateUrl` for basic string types.
macro_rules! validate_type_of_str {
    ($type_:ty) => {
        impl ValidateUrl for $type_ {
            fn as_url_string(&self) -> Option<Cow<'_, str>> {
                // Borrows the string slice.
                Some(Cow::Borrowed(self))
            }
        }
    };
}

validate_type_of_str!(str);
validate_type_of_str!(&str);
validate_type_of_str!(String);

/// Implements `ValidateUrl` for `Option<T>` where `T` implements `ValidateUrl`.
/// If the `Option` is `None`, `as_url_string` returns `None`, leading `validate_url` to return `true`.
impl<T> ValidateUrl for Option<T>
where
    T: ValidateUrl,
{
    fn as_url_string(&self) -> Option<Cow<'_, str>> {
        let Some(u) = self else {
            // If Option is None, there's no string to validate.
            return None;
        };

        // Delegates to the inner Some(T)'s implementation.
        T::as_url_string(u)
    }
}

/// Implements `ValidateUrl` for `Cow<'_, str>`.
impl ValidateUrl for Cow<'_, str> {
    fn as_url_string(&self) -> Option<Cow<'_, str>> {
        // Borrows the inner string slice.
        Some(Cow::Borrowed(self.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::ValidateUrl;

    #[test]
    fn test_validate_url() {
        let tests = vec![
            ("http", false),
            ("https://google.com", true),
            ("http://localhost:80", true),
            // Outside the http/https allow-list — see validate_url.
            ("ftp://localhost:80", false),
            ("javascript:alert(1)", false),
        ];

        for (url, expected) in tests {
            assert_eq!(url.validate_url(), expected);
        }
    }

    #[test]
    fn test_validate_url_cow() {
        let test: Cow<'static, str> = "http://localhost:80".into();
        assert!(test.validate_url());
        let test: Cow<'static, str> = String::from("http://localhost:80").into();
        assert!(test.validate_url());
        let test: Cow<'static, str> = "http".into();
        assert!(!test.validate_url());
        let test: Cow<'static, str> = String::from("http").into();
        assert!(!test.validate_url());
    }
}
