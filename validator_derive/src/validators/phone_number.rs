use darling::util::Override;
use quote::quote;
use std::str::FromStr;
use syn::LitStr;

use crate::{
    types::ValidateField,
    utils::{CrateName, quote_code, quote_message},
};

#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct PhoneNumber {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// Optional ISO 3166-1 alpha-2 country code
    pub country: Option<LitStr>,

    /// If true, empty strings are considered valid (for nullable fields).
    #[darling(default)]
    pub nullable: bool,
}

pub fn check_phone_number_validator_form(field: &ValidateField) {
    if let Some(Override::Explicit(args)) = &field.phone_number
        && let Some(country) = args.country.as_ref()
    {
        let country_str = country.value();
        if phonenumber::country::Id::from_str(&country_str).is_err() {
            proc_macro_error2::abort!(
                country.span(),
                "Invalid country code `{}` for phone_number; must be a valid ISO 3166-1 alpha-2 code like `UG`, `KE`, `US`",
                country_str
            )
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the phone number validation logic.
///
/// This function takes the parsed `PhoneNumber` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the phone number validation at runtime.
pub fn tokens(
    crate_name: &CrateName,
    phone_number: PhoneNumber,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    let message = quote_message(phone_number.message);
    let code = quote_code(crate_name, phone_number.code, "phone_number");

    let country_id_arg = match phone_number.country {
        Some(lit_str) => {
            let country_value = lit_str.value();
            quote! {
                Some(#country_value)
            }
        }
        None => {
            quote! { None }
        }
    };

    // Generate validation code based on nullable flag
    if phone_number.nullable {
        // If nullable, allow empty strings
        quote! {
            // Skip validation if the value is empty (will be converted to NULL)
            if !#field_name.is_empty() && !#field_name.validate_phone_number(#country_id_arg) {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    } else {
        // Original behavior - empty strings are invalid
        quote! {
            if !#field_name.validate_phone_number(#country_id_arg) {
                #code
                #message
                err.add_param(::std::borrow::Cow::from("value"), &#field_name);
                errors.add(#field_name_str, err);
            }
        }
    }
}
