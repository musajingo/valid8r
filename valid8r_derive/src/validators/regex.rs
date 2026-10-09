//! This module handles the parsing and token generation for the `#[validate(regex(...))]` attribute.
//!
//! The `regex` validator checks if a string value matches a given regular expression.
//! The regular expression itself is provided as a path to a static `Regex` instance.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(regex(path = ...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct Regex {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The path to the `Regex` static/constant.
    /// This is expected to be a `syn::Expr` that resolves to a `&'static regex::Regex`
    /// or a type that dereferences to it (like `LazyLock<Regex>`).
    pub path: syn::Expr,
}

/// Generates the `proc_macro2::TokenStream` for the regex validation logic.
///
/// This function takes the parsed `Regex` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the regex match at runtime.
pub fn tokens(
    crate_name: &CrateName,
    regex: Regex,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Extract the path to the Regex static/constant.
    let path = regex.path;
    // Prepare the custom message and code, if provided.
    let message = quote_message(regex.message);
    let code = quote_code(crate_name, regex.code, "regex");

    // Generate the token stream for the regex validation.
    // It calls `validate_regex()` on the field, passing the regex path.
    quote! {
        if !&#field_name.validate_regex(&#path) {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
