//! This module handles the parsing, compile-time validation, and token generation
//! for the `#[validate(must_match(...))]` attribute.
//!
//! The `must_match` validator ensures that the value of the annotated field
//! is equal to the value of another specified field within the same struct.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

use crate::types::ValidateField;
use crate::utils::{
    CrateName, FieldOptionality, field_optionality, is_field_sensitive, quote_code, quote_message,
};

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

/// Generates an expression that views a field's value as `Option<&T>`,
/// so the two sides of a `must_match` comparison always have the same type
/// regardless of how each field wraps its inner value.
fn value_as_option_ref(expr: TokenStream2, optionality: FieldOptionality) -> TokenStream2 {
    match optionality {
        FieldOptionality::NotOptional => quote!(::std::option::Option::Some(&#expr)),
        FieldOptionality::Option => quote!(#expr.as_ref()),
        FieldOptionality::OptionOption => quote!(#expr.as_ref().and_then(|inner| inner.as_ref())),
        FieldOptionality::Delta => quote!(#expr.value()),
    }
}

/// Generates the `proc_macro2::TokenStream` for the `must_match` validation logic.
///
/// Both fields are normalized to `Option<&T>` before comparing, so any mix of
/// `T`, `Option<T>`, `Option<Option<T>>` and `Delta<T>` works. The check only
/// runs when the annotated field has a value (absent/cleared fields have
/// nothing to confirm); the other field then counts as a mismatch unless it
/// holds an equal value — in PATCH terms, `Clear` and `Unchanged` never match
/// a set value.
pub fn tokens(
    crate_name: &CrateName,
    must_match: MustMatch,
    field_name: &syn::Ident,
    field_name_str: &str,
    all_fields: &[&syn::Field],
) -> TokenStream2 {
    // Get the identifier for the 'other' field.
    let o = must_match.other;
    let other_name = o
        .get_ident()
        .expect("Cannot get ident from `other` field value")
        .to_string();

    // Normalize both sides to `Option<&T>` based on each field's type.
    let main_expr = value_as_option_ref(
        quote!(self.#field_name),
        field_optionality(field_name_str, all_fields),
    );
    let other_expr =
        value_as_option_ref(quote!(self.#o), field_optionality(&other_name, all_fields));

    // Prepare the custom message and code, if provided.
    let message = quote_message(must_match.message);
    let code = quote_code(crate_name, must_match.code, "must_match");

    // A `#[validate(sensitive)]` field's value must never be serialized into
    // error params — not even as the `other` param of a different field's
    // must_match error — so the params are omitted at codegen time rather
    // than redacted afterwards.
    let other_param = if is_field_sensitive(&other_name, all_fields) {
        quote!()
    } else {
        quote!(err.add_param(::std::borrow::Cow::from("other"), &must_match_other);)
    };
    let value_param = if is_field_sensitive(field_name_str, all_fields) {
        quote!()
    } else {
        quote!(err.add_param(::std::borrow::Cow::from("value"), &must_match_value);)
    };

    // Generate the token stream for the must_match validation.
    quote! {
        if let ::std::option::Option::Some(must_match_value) = #main_expr {
            let must_match_other = #other_expr;
            if !#crate_name::validate_must_match(
                ::std::option::Option::Some(must_match_value),
                must_match_other,
            ) {
                #code
                #message
                #other_param
                #value_param
                errors.add(#field_name_str, err);
            }
        }
    }
}
