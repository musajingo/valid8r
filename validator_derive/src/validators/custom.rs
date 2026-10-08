//! This module handles the parsing, compile-time validation, and token generation
//! for the `#[validate(custom(...))]` attribute.
//!
//! The `custom` validator allows users to define their own validation functions.

use quote::quote;

use crate::types::ValidateField;
use crate::utils::quote_message;

/// Represents the arguments parsed from a `#[validate(custom(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct Custom {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The path to the custom validation function.
    /// This is expected to be a `syn::Path` to a function that takes the field's value
    /// (and optionally a `ValidationContext`) and returns a `Result<(), validator::ValidationError>`.
    pub function: darling::Result<syn::Path>,

    /// Whether the custom function requires the validation context.
    /// If `true`, the custom function will be called with `(field_value, context)`.
    /// If `false` or `None`, it will be called with `(field_value)`.
    pub use_context: Option<bool>,
}

/// Performs compile-time checks on the `#[validate(custom(...))]` attribute.
///
/// This function ensures that the `function` argument provided to the `custom`
/// validator is a valid path. It aborts compilation with an error if not.
pub fn check_custom_validator_form(field: &ValidateField, field_name: &str) {
    for c in &field.custom {
        // If function is not a path
        if let Err(e) = &c.function {
            proc_macro_error2::abort!(
                e.span(), "Invalid attribute #[validate(custom(...))] on field `{}`:", field_name;
                note = "Invalid argument for `custom` validator, only paths are allowed";
                help = "Try formatting the argument like `path::to::function` or `\"path::to::function\"`"
            );
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the custom validation logic.
///
/// This function takes the parsed `Custom` arguments and the field's identifier
/// to construct the Rust code that will call the user-defined custom validation function
/// at runtime.
pub fn tokens(
    custom: Custom,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Unwrap the function path; `check_custom_validator_form` should have caught errors.
    let fn_call = custom.function.unwrap();

    // Determine the arguments to pass to the custom validation function
    // based on the `use_context` flag.
    let args = if let Some(arg) = custom.use_context {
        // If `use_context` is true, pass both field value and context.
        if arg {
            quote!(#field_name, args)
        } else {
            quote!(#field_name)
        }
    } else {
        quote!(#field_name)
    };

    // Prepare the custom message, if provided.
    let message = quote_message(custom.message);

    // Prepare the custom error code, if provided.
    let code = if let Some(c) = custom.code {
        quote!(
            err.code = ::std::borrow::Cow::from(#c);
        )
    } else {
        quote!()
    };

    // Generate the token stream for calling the custom function and handling its result.
    quote! {
        match #fn_call(#args) {
            ::std::result::Result::Ok(()) => {}
            ::std::result::Result::Err(mut err) => {
                #code
                #message
                // Add the validated field's value as a parameter to the error.
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    }
}
