//! This module handles the parsing, compile-time validation, and token generation
//! for the `#[validate(length(...))]` attribute.
//!
//! The `length` validator checks if a value's length falls within a specified range or equals a specific value.

use proc_macro_error2::abort;
use quote::quote;
use syn::{Attribute, Expr, spanned::Spanned};

use crate::types::ValidateField;
use crate::utils::get_attr;
use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(length(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct Length {
    pub code: Option<String>,
    pub message: Option<String>,

    /// Minimum length (inclusive).
    pub min: Option<Expr>,

    /// Maximum length (inclusive).
    pub max: Option<Expr>,

    /// Exact length.
    pub equal: Option<Expr>,
}

/// Performs compile-time checks on the `#[validate(length(...))]` attribute.
///
/// This function ensures that:
/// 1. The `length` validator is not used with both `equal` and (`min` or `max`) arguments simultaneously.
/// 2. The `length` validator has at least one argument (`min`, `max`, or `equal`).
///
/// It aborts compilation with an error if any of these conditions are not met.
pub fn check_length_validator_form(
    field: &ValidateField,
    field_name: &str,
    field_attrs: &[Attribute],
) {
    if let Some(length) = &field.length {
        // Check for conflicting arguments: `equal` with `min` or `max`.
        // If length has both `equal` and `min` or `max` argument
        if length.equal.is_some() && (length.min.is_some() || length.max.is_some()) {
            abort! {
                length.equal.clone().unwrap().span(), "Invalid attribute #[validate(length(...))] on field `{}`:", field_name;
                note = "Both `equal` and `min` or `max` have been set";
                help = "Exclusively use either the `equal` or `min` and `max` attributes"
            }
        }

        // Check if the validator has any arguments.
        // Check if `length` validator has no arguments.
        if length.equal.is_none() && length.min.is_none() && length.max.is_none() {
            // Try to get the span of the specific `length` attribute for better error reporting.
            let error_span = get_attr(field_attrs, "length").map_or_else(
                || {
                    field_attrs
                        .first()
                        .map_or_else(proc_macro2::Span::call_site, |a| a.span())
                },
                |a| a.span(),
            );
            abort!(
                error_span, "Invalid attribute #[validate(length(...))] on field `{}`:", field_name;
                note = "Validator `length` requires at least 1 argument";
                help = "Add the argument `equal`, `min` or `max`"
            )
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the length validation logic.
///
/// This function takes the parsed `Length` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the length validation at runtime.
pub fn tokens(
    crate_name: &CrateName,
    length: Length,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare tokens for min, max, and equal arguments and their corresponding error parameters.
    // If an argument is not provided, it defaults to `None` and an empty token stream for the error parameter.

    let (min, min_err) = if let Some(v) = length.min.as_ref() {
        (
            quote!(Some(#v)),
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);),
        )
    } else {
        (quote!(None), quote!())
    };
    let (max, max_err) = if let Some(v) = length.max {
        (
            quote!(Some(#v)),
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);),
        )
    } else {
        (quote!(None), quote!())
    };
    let (equal, equal_err) = if let Some(v) = length.equal {
        (
            quote!(Some(#v)),
            quote!(err.add_param(::std::borrow::Cow::from("equal"), &#v);),
        )
    } else {
        (quote!(None), quote!())
    };

    // Prepare the custom message and code, if provided.
    let message = quote_message(length.message);
    let code = quote_code(crate_name, length.code, "length");

    // Generate the token stream for the length validation.
    quote! {
        if !#field_name.validate_length(#min, #max, #equal) {
            #code
            #message
            #min_err
            #max_err
            #equal_err
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
