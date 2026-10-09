//! This module handles the parsing and token generation for the `#[validate(email(...))]` attribute.
//!
//! The `email` validator checks if a string value (or a type implementing `ValidateEmail`)
//! is a valid email address.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(email(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to optionally specify a custom error `message` and `code`
/// for the email validation. If not provided, default values will be used.
/// The `Default` derive is used when `#[validate(email)]` is specified without arguments.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Email {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// If true, empty strings are considered valid (for nullable fields).
    #[darling(default)]
    pub nullable: bool,
}

/// Generates the `proc_macro2::TokenStream` for the email validation logic.
///
/// This function takes the parsed `Email` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the email validation at runtime.
pub fn tokens(
    crate_name: &CrateName,
    email: Email,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare the custom message and code, if provided.
    let message = quote_message(email.message);
    let code = quote_code(crate_name, email.code, "email");

    // Generate validation code based on nullable flag
    if email.nullable {
        // If nullable, allow empty strings
        quote! {
            // Skip validation if the value is empty (will be converted to NULL)
            if !#field_name.is_empty() && !#field_name.validate_email() {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    } else {
        // Original behavior - empty strings are invalid
        quote! {
            if !#field_name.validate_email() {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    }
}
