//! This module handles the parsing, compile-time validation, and token generation
//! for the `#[validate(must_match(...))]` attribute.
//!
//! The `must_match` validator ensures that the value of the annotated field
//! is equal to the value of another specified field within the same struct.

use quote::quote;
use syn::spanned::Spanned;

use crate::types::ValidateField;
use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(must_match(other = "field_name"))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct MustMatch {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The path to the other field to compare against.
    /// This is expected to be a simple identifier for another field in the same struct.
    pub other: syn::Path,
}

/// Performs compile-time checks on the `#[validate(must_match(...))]` attribute.
///
/// This function ensures that the `other` field specified in the `must_match`
/// attribute actually exists within the struct being validated.
/// It aborts compilation with an error if the `other` field is not found.
pub fn check_must_match_validator_form(
    field: &ValidateField,
    field_name: &str,
    struct_ident: &syn::Ident,
    all_fields: &[&syn::Field],
) {
    if let Some(must_match) = &field.must_match {
        let other_field = must_match
            .other
            .get_ident()
            .expect("Cannot get ident from `other` field value")
            .to_string();

        // Check if the `other_field_name` exists in `all_fields` of the struct.
        if !all_fields
            .iter()
            .any(|f| f.ident.as_ref().is_some_and(|i| i == &other_field))
        {
            proc_macro_error2::abort!(
                must_match.other.span(), "Invalid attribute for #[validate(must_match(...))] on field `{}`:", field_name;
                note =  "The `other` field doesn't exist in the struct `{}`", struct_ident;
                help = "Add the field `{}` to the struct", other_field
            )
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the `must_match` validation logic.
///
/// This function takes the parsed `MustMatch` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the comparison at runtime.
pub fn tokens(
    crate_name: &CrateName,
    must_match: MustMatch,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Get the identifier for the 'other' field.
    let o = must_match.other;
    // Quote the path to the 'other' field's value and the error parameter for it.
    let (other, other_err) = (
        quote!(self.#o),
        quote!(err.add_param(::std::borrow::Cow::from("other"), &self.#o);),
    );

    // Prepare the custom message and code, if provided.
    let message = quote_message(must_match.message);
    let code = quote_code(crate_name, must_match.code, "must_match");

    // Generate the token stream for the must_match validation.
    quote! {
        if !#crate_name::validate_must_match(&#field_name, &(#other)) {
            #code
            #message
            #other_err
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
