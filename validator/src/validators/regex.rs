//! This module provides traits and implementations for validating strings against regular expressions.
//!
//! It defines two main traits:
//! - `AsRegex`: For types that can provide a `Regex` instance, potentially wrapped or borrowed.
//! - `ValidateRegex`: For types whose string representation can be validated against a `Regex`.

use regex::Regex;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

/// A trait for types that can be converted or provide access to a `Regex` instance.
///
/// This trait is used to abstract over different ways a `Regex` might be stored or accessed,
/// such as a direct `Regex` object, or one wrapped in `OnceLock`, `LazyLock`, etc.
pub trait AsRegex {
    /// Returns a `Cow<Regex>`, allowing for borrowed or owned `Regex` instances.
    fn as_regex(&self) -> Cow<'_, Regex>;
}

/// Implements `AsRegex` for `Regex` itself.
impl AsRegex for Regex {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Borrows the Regex directly.
        Cow::Borrowed(self)
    }
}

impl<T> AsRegex for &T
where
    T: AsRegex,
{
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Delegates to the inner type's AsRegex implementation.
        T::as_regex(self)
    }
}

/// Implements `AsRegex` for a reference to `OnceLock<Regex>`.
/// Assumes the `Regex` has been initialized.
impl AsRegex for &OnceLock<Regex> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Borrows the Regex from the OnceLock. Panics if not initialized.
        Cow::Borrowed(self.get().unwrap())
    }
}

/// Implements `AsRegex` for a reference to `Mutex<OnceCell<Regex>>`.
/// Assumes the `Regex` has been initialized.
impl AsRegex for &Mutex<OnceCell<Regex>> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Clones the Regex from the Mutex-guarded OnceCell. Panics if not initialized.
        Cow::Owned(self.lock().unwrap().get().unwrap().clone())
    }
}

/// Implements `AsRegex` for a reference to `Mutex<OnceLock<Regex>>`.
/// Assumes the `Regex` has been initialized.
impl AsRegex for &Mutex<OnceLock<Regex>> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Clones the Regex from the Mutex-guarded OnceLock. Panics if not initialized.
        Cow::Owned(self.lock().unwrap().get().unwrap().clone())
    }
}

/// Implements `AsRegex` for a reference to `Arc<Mutex<OnceCell<Regex>>>`.
/// Assumes the `Regex` has been initialized.
impl AsRegex for &Arc<Mutex<OnceCell<Regex>>> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Clones the Regex from the Arc-ed Mutex-guarded OnceCell. Panics if not initialized.
        Cow::Owned(self.lock().unwrap().get().unwrap().clone())
    }
}

/// Implements `AsRegex` for a reference to `Arc<Mutex<OnceLock<Regex>>>`.
/// Assumes the `Regex` has been initialized.
impl AsRegex for &Arc<Mutex<OnceLock<Regex>>> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Clones the Regex from the Arc-ed Mutex-guarded OnceLock. Panics if not initialized.
        Cow::Owned(self.lock().unwrap().get().unwrap().clone())
    }
}

/// Implements `AsRegex` for `LazyLock<Regex>`.
impl AsRegex for LazyLock<Regex> {
    fn as_regex(&self) -> Cow<'_, Regex> {
        // Borrows the Regex from the LazyLock.
        Cow::Borrowed(self)
    }
}

/// A trait for types that can be validated against a regular expression.
pub trait ValidateRegex {
    /// Validates the implementing type against the given regular expression.
    ///
    /// # Arguments
    ///
    /// * `regex`: An instance of a type that implements `AsRegex`, providing the `Regex` to match against.
    ///
    /// # Returns
    ///
    /// `true` if the type's string representation matches the regex, `false` otherwise.
    /// For `Option<T>`, returns `true` if `None`, otherwise validates the `Some` value.
    fn validate_regex(&self, regex: impl AsRegex) -> bool;
}

/// Implements `ValidateRegex` for references to types that implement `ValidateRegex`.
impl<T> ValidateRegex for &T
where
    T: ValidateRegex,
{
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        T::validate_regex(self, regex)
    }
}

/// Implements `ValidateRegex` for `Option<T>` where `T` implements `ValidateRegex`.
/// If the `Option` is `None`, validation passes (returns `true`).
impl<T> ValidateRegex for Option<T>
where
    T: ValidateRegex,
{
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        if let Some(h) = self {
            T::validate_regex(h, regex)
        } else {
            true
        }
    }
}

/// Implements `ValidateRegex` for `Cow<'_, T>` where `&'a T` implements `ValidateRegex`.
impl<T> ValidateRegex for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    for<'a> &'a T: ValidateRegex,
{
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        self.as_ref().validate_regex(regex)
    }
}

/// Implements `ValidateRegex` for `String`.
impl ValidateRegex for String {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        regex.as_regex().is_match(self)
    }
}

/// Implements `ValidateRegex` for `&str`.
impl ValidateRegex for &str {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        regex.as_regex().is_match(self)
    }
}

/// Implements `ValidateRegex` for `str`.
impl ValidateRegex for str {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        regex.as_regex().is_match(self)
    }
}

/// Implements `ValidateRegex` for `Box<T>` where `T` implements `ValidateRegex`.
impl<T: ValidateRegex> ValidateRegex for Box<T> {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        self.as_ref().validate_regex(regex)
    }
}

/// Implements `ValidateRegex` for `Rc<T>` where `T` implements `ValidateRegex`.
impl<T: ValidateRegex> ValidateRegex for Rc<T> {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        self.as_ref().validate_regex(regex)
    }
}

/// Implements `ValidateRegex` for `Arc<T>` where `T` implements `ValidateRegex`.
impl<T: ValidateRegex> ValidateRegex for Arc<T> {
    fn validate_regex(&self, regex: impl AsRegex) -> bool {
        self.as_ref().validate_regex(regex)
    }
}
