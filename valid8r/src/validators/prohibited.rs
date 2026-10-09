//! Provides the `ValidateProhibited` trait, used for "prohibited" validation rules
//! like `prohibited_if`. This trait helps determine if a field is absent,
//! which is the valid state when a prohibition is active.

/// A trait for types that can be checked for absence, particularly in "prohibited" validation scenarios.
///
/// If a field is subject to a prohibition (e.g., a `prohibited_if` condition is met),
/// it must be absent (e.g., `None`) to pass validation. This trait facilitates that check.
pub trait ValidateProhibited {
    /// Checks if the value is absent (e.g., `None` for an `Option`).
    ///
    /// Returns `true` if the value is absent, meaning the field's state complies with a prohibition.
    /// Returns `false` if the value is present.
    ///
    /// For a validator like `prohibited_if(condition)`, an error typically occurs if `condition`
    /// is true AND this method returns `false` (i.e., the value is present when it should be absent).
    ///
    /// The default implementation relies on `is_none()`.
    fn validate_prohibited(&self) -> bool {
        self.is_none()
    }

    /// Determines if the underlying value is considered "none" or absent.
    ///
    /// Implementors must provide a concrete definition for this method.
    ///
    /// # Returns
    /// * `true` if the value is absent (e.g., `Option::None`).
    /// * `false` if the value is present (e.g., `Option::Some`).
    fn is_none(&self) -> bool;
}

/// Implementation of `ValidateProhibited` for `Option<T>`.
///
/// An `Option<T>` is "absent" if it is the `None` variant.
impl<T> ValidateProhibited for Option<T> {
    /// Returns `true` if the `Option` is `None`, otherwise `false`.
    fn is_none(&self) -> bool {
        self.is_none()
    }
}
