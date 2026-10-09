//! This module handles the parsing, compile-time validation, and token generation
//! for the `#[validate(range(...))]` attribute.
//!
//! The `range` validator checks if a numeric value falls within a specified inclusive or exclusive range.
use quote::quote;
use syn::{Attribute, Expr, spanned::Spanned};

use crate::types::ValidateField;
use crate::utils::get_attr;
use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(range(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct Range {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// Minimum value (inclusive).
    pub min: Option<Expr>,

    /// Maximum value (inclusive).
    pub max: Option<Expr>,

    /// Minimum value (exclusive).
    pub exclusive_min: Option<Expr>,

    /// Maximum value (exclusive).
    pub exclusive_max: Option<Expr>,
}

/// Performs compile-time checks on the `#[validate(range(...))]` attribute.
///
/// This function ensures that the `range` validator has at least one argument
/// (`min`, `max`, `exclusive_min`, or `exclusive_max`).
/// It aborts compilation with an error if this condition is not met.
pub fn check_range_validator_form(
    field: &ValidateField,
    field_name: &str,
    field_attrs: &[Attribute],
) {
    if let Some(range) = &field.range {
        // Check if the validator has any arguments.
        if range.min.is_none()
            && range.max.is_none()
            && range.exclusive_min.is_none()
            && range.exclusive_max.is_none()
        {
            // Try to get the span of the specific `range` attribute for better error reporting.
            // If the attribute itself cannot be found (which shouldn't happen if `field.range` is Some),
            // fall back to the first attribute's span or call_site.
            let error_span = get_attr(field_attrs, "range").map_or_else(
                || {
                    field_attrs
                        .first()
                        .map_or_else(proc_macro2::Span::call_site, |a| a.span())
                },
                |a| a.span(),
            );
            proc_macro_error2::abort!(
                error_span,  "Invalid attribute #[validate(range(...))] on field `{}`:", field_name;
                note = "Validator `range` requires at least 1 argument";
                help = "Add the argument `min` or `max`, `exclusive_min` or `exclusive_max`"
            )
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the range validation logic.
///
/// This function takes the parsed `Range` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the range validation at runtime.
pub fn tokens(
    crate_name: &CrateName,
    range: Range,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare tokens for min, max, exclusive_min, and exclusive_max arguments
    // and their corresponding error parameters.
    // If an argument is not provided, it defaults to `None` and an empty token stream for the error parameter.
    let (min, min_err) = if let Some(m) = range.min {
        (
            quote!(Some(#m)),
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#m);),
        )
    } else {
        (quote!(None), quote!())
    };

    let (max, max_err) = if let Some(m) = range.max {
        (
            quote!(Some(#m)),
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#m);),
        )
    } else {
        (quote!(None), quote!())
    };

    let (ex_min, ex_min_err) = if let Some(m) = range.exclusive_min {
        (
            quote!(Some(#m)),
            quote!(err.add_param(::std::borrow::Cow::from("exclusive_min"), &#m);),
        )
    } else {
        (quote!(None), quote!())
    };

    let (ex_max, ex_max_err) = if let Some(m) = range.exclusive_max {
        (
            quote!(Some(#m)),
            quote!(err.add_param(::std::borrow::Cow::from("exclusive_max"), &#m);),
        )
    } else {
        (quote!(None), quote!())
    };

    // Prepare the custom message and code, if provided.
    let message = quote_message(range.message);
    let code = quote_code(crate_name, range.code, "range");

    // Generate the token stream for the range validation.
    quote! {
        if !#field_name.validate_range(#min, #max, #ex_min, #ex_max) {
            #code
            #message
            #min_err
            #max_err
            #ex_min_err
            #ex_max_err
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
