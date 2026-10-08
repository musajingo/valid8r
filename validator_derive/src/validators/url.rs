//! This module handles the parsing and token generation for the `#[validate(url(...))]` attribute.
//!
//! The `url` validator checks if a string value is a valid URL.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(url(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to optionally specify a custom error `message` and `code`
/// for the URL validation. If not provided, default values will be used.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Url {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// If true, empty strings are considered valid (for nullable fields).
    #[darling(default)]
    pub nullable: bool,
}

/// Generates the `proc_macro2::TokenStream` for the URL validation logic.
///
/// This function takes the parsed `Url` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the URL validation at runtime.
///
/// # Arguments
/// * `crate_name` - Identifier for the validator crate (e.g., `::validator`).
/// * `url` - The parsed `Url` struct containing custom code or message.
/// * `field_name` - Token stream representing the field being validated (e.g., `self.my_url_field`).
/// * `field_name_str` - The string name of the field, used for error reporting.
///
/// # Returns
/// A `proc_macro2::TokenStream` containing the Rust code for the URL validation.
pub fn tokens(
    crate_name: &CrateName,
    url: Url,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare the custom message and code, if provided.
    let message = quote_message(url.message);
    let code = quote_code(crate_name, url.code, "url");

    // Generate validation code based on nullable flag
    if url.nullable {
        // If nullable, allow empty strings
        quote! {
            // Skip validation if the value is empty (will be converted to NULL)
            if !#field_name.is_empty() && !#field_name.validate_url() {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    } else {
        // Original behavior - empty strings are invalid
        quote! {
            if !#field_name.validate_url() {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    }
}
