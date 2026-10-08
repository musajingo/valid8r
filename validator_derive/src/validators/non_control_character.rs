//! This module handles the parsing and token generation for the `#[validate(non_control_character)]`
//! and `#[validate(non_control_character(...))]` attributes.
//!
//! The `non_control_character` validator checks if a string (or a type that can be
//! iterated as characters) contains any control characters.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(non_control_character(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to optionally specify a custom error `message` and `code`
/// for the `non_control_character` validation. If not provided, default values will be used.
/// The `Default` derive is used when `#[validate(non_control_character)]` is specified without arguments.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct NonControlCharacter {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,
}

/// Generates the `proc_macro2::TokenStream` for the `non_control_character` validation logic.
///
/// This function takes the parsed `NonControlCharacter` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the check at runtime.
pub fn tokens(
    crate_name: &CrateName,
    non_control_char: NonControlCharacter,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare the custom message and code, if provided.
    let message = quote_message(non_control_char.message);
    let code = quote_code(crate_name, non_control_char.code, "non_control_character");

    // Generate the token stream for the non_control_character validation.
    // It calls `validate_non_control_character()` on the field and adds an error if it returns false.
    quote! {
        if !#field_name.validate_non_control_character() {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
