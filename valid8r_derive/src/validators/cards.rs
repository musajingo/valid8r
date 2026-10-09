//! This module provides functions for generating token streams related to credit card validation.
//! It's part of a procedural macro and helps in constructing the validation logic
//! that will be injected into the user's code.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};
/// Represents the arguments parsed from a `#[validate(credit_card(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to optionally specify a custom error `message` and `code`
/// for the credit card validation. If not provided, default values will be used.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Card {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,
}

/// Generates the `proc_macro2::TokenStream` for credit card validation.
///
/// This function takes the necessary components for building a validation check
/// for a field that is expected to be a credit card number. It constructs a
/// code snippet that, when executed, will validate the field's value as a
/// credit card. If the validation fails, it adds an error to a validation
/// errors collection.
///
/// # Arguments
///
/// * `crate_name` - A reference to `CrateName`, used to correctly qualify paths to items
///   from the `valid8r` crate (e.g., `::valid8r::ValidationErrors`).
/// * `credit_card` - A `Card` struct instance containing metadata for the validation,
///   such as a custom error message or error code.
/// * `field_name` - A `proc_macro2::TokenStream` representing the identifier of the field
///   being validated (e.g., `self.my_card_field`).
/// * `field_name_str` - A string slice (`&str`) representing the name of the field,
///   used as a key when adding errors to the `ValidationErrors` map.
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the Rust code for the credit card validation logic.
/// This token stream is intended to be interpolated into the derived `Validate` trait
/// implementation.
pub fn tokens(
    crate_name: &CrateName,
    credit_card: Card,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    let message = quote_message(credit_card.message);
    // Prepare the custom error code, if any, qualified with the crate name.
    let code = quote_code(crate_name, credit_card.code, "credit_card");

    // Construct the token stream for the validation logic.
    // This code checks if the field value is a valid credit card.
    // If not, it creates a new validation error, sets its code and message,
    // adds the invalid value as a parameter to the error, and then adds
    // this error to the `errors` collection.
    quote! {
        // Call the `validate_credit_card` method on the field.
        if !#field_name.validate_credit_card() {
            // If validation fails, set up the error code and message.
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
