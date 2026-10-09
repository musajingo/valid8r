use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

/// Represents a single validation error.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// A code identifying the type of error (e.g., "length", "required").
    pub code: Cow<'static, str>,

    /// An optional user-friendly message describing the error.
    pub message: Option<Cow<'static, str>>,

    /// A map of parameters associated with the error, providing more context (e.g., min_length: 5).
    pub params: HashMap<Cow<'static, str>, Value>,
}

impl ValidationError {
    /// Creates a new `ValidationError` with a given error code.
    pub fn new(code: &'static str) -> Self {
        Self {
            code: Cow::from(code),
            message: None,
            params: HashMap::new(),
        }
    }

    /// Adds a parameter to the `ValidationError`.
    /// The parameter value is serialized to a `serde_json::Value`.
    pub fn add_param<T>(&mut self, key: Cow<'static, str>, val: &T)
    where
        T: serde::ser::Serialize,
    {
        // Inserts the parameter key and its JSON representation into the params map.
        // .unwrap() is used here, assuming serialization to Value will not fail for common types.
        self.params.insert(key, to_value(val).unwrap());
    }

    /// Adds a custom message to a `ValidationError` that will be used when displaying the
    /// `ValidationError`, instead of an auto-generated description.
    /// This method consumes the `ValidationError` and returns a new one with the message set.
    pub fn with_message(mut self, message: Cow<'static, str>) -> Self {
        self.message = Some(message);
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = self.message.as_ref() {
            return write!(fmt, "{}", msg);
        }
        write!(fmt, "Validation error: {} [{:?}]", self.code, self.params)
    }
}

/// Implements the standard `Error` for `ValidationError`.
impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_error_new() {
        let error = ValidationError::new("required");
        assert_eq!(error.code, "required");
        assert_eq!(error.message, None);
        assert!(error.params.is_empty());
    }

    #[test]
    fn test_validation_error_add_param() {
        let mut error = ValidationError::new("length");
        error.add_param(Cow::from("min"), &5);
        error.add_param(Cow::from("actual"), &3);

        assert_eq!(error.params.get("min"), Some(&json!(5)));
        assert_eq!(error.params.get("actual"), Some(&json!(3)));
    }

    #[test]
    fn test_validation_error_with_message() {
        let error = ValidationError::new("custom_error")
            .with_message(Cow::from("This is a custom message."));
        assert_eq!(error.code, "custom_error");
        assert_eq!(error.message, Some(Cow::from("This is a custom message.")));
    }

    #[test]
    fn test_validation_error_display_with_message() {
        let error = ValidationError::new("irrelevant_code")
            .with_message(Cow::from("User-friendly message takes precedence."));
        assert_eq!(
            format!("{}", error),
            "User-friendly message takes precedence."
        );
    }
}
