//! This module handles the parsing and token generation for the `#[validate(does_not_contain(...))]` attribute.
//!
//! The `does_not_contain` validator checks if a value (e.g., a string or collection)
//! does *not* contain a specific pattern or element.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(does_not_contain(pattern = "..."))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct DoesNotContain {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The pattern that should *not* be contained.
    pub pattern: String,
}

/// Generates the `proc_macro2::TokenStream` for the `does_not_contain` validation logic.
///
/// This function takes the parsed `DoesNotContain` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the check at runtime.
pub fn tokens(
    crate_name: &CrateName,
    does_not_contain: DoesNotContain,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Extract the pattern that should not be contained.
    let p = does_not_contain.pattern;

    // Quote the pattern itself for use in the validation call, and quote the error parameter for it.
    let (needle, needle_err) = (
        quote!(#p),
        quote!(err.add_param(::std::borrow::Cow::from("needle"), &#p);),
    );

    // Prepare the custom message and code, if provided.
    let message = quote_message(does_not_contain.message);
    let code = quote_code(crate_name, does_not_contain.code, "does_not_contain");

    // Generate the token stream for the does_not_contain validation.
    quote! {
        if !#field_name.validate_does_not_contain(#needle) {
            #code
            #message
            #needle_err
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
