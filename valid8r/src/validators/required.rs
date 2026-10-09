//! Provides the `ValidateRequired` trait to check if an optional value is present.

/// A trait for types that can be validated for presence.
///
/// This is primarily used to check if an `Option<T>` contains a `Some(value)`.
pub trait ValidateRequired {
    /// Checks if the value is present (e.g., `Some` for an `Option`).
    ///
    /// This method has a default implementation that relies on `is_some()`.
    fn validate_required(&self) -> bool {
        self.is_some()
    }

    /// Determines if the underlying value is considered "some" or present.
    ///
    /// Implementors must provide a concrete definition for this method.
    fn is_some(&self) -> bool;
}

/// Implementation of `ValidateRequired` for `Option<T>`.
///
/// An `Option<T>` is considered "some" if it is the `Some` variant,
/// and not "some" (i.e., "none") if it is the `None` variant.
impl<T> ValidateRequired for Option<T> {
    /// Returns `true` if the `Option` is `Some(T)`, `false` otherwise.
    fn is_some(&self) -> bool {
        self.is_some()
    }
}
