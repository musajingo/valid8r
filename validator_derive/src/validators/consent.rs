//! This module provides the structures and token generation functions
//! for consent-based validations like `accepted`, `declined`, `accepted_if`,
//! and `declined_if`. These are used by the `Validate` derive macro.

use quote::quote;

use crate::types::ValidateField;
use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the `#[validate(accepted(...))]` attribute.
///
/// This validation ensures that a field's value is considered "accepted"
/// according to the `ValidateConsent` trait.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Accepted {
    /// Optional custom error code for this validation.
    /// If not provided, defaults to "accepted".
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,
}

/// Represents the `#[validate(declined(...))]` attribute.
///
/// This validation ensures that a field's value is considered "declined"
/// according to the `ValidateConsent` trait.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Declined {
    /// Optional custom error code for this validation.
    /// If not provided, defaults to "declined".
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,
}

/// Represents the `#[validate(accepted_if(...))]` attribute.
///
/// This validation ensures that a field's value is "accepted"
/// *if* a specified condition function returns `true`.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct AcceptedIf {
    /// Optional custom error code for this validation.
    /// If not provided, defaults to "accepted".
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The path to a function that determines if this validation rule should apply.
    /// The function is expected to take `&self` (a reference to the struct instance)
    /// and return a `bool`.
    pub function: darling::Result<syn::Path>,
}

/// Represents the `#[validate(declined_if(...))]` attribute.
///
/// This validation ensures that a field's value is "declined"
/// *if* a specified condition function returns `true`.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct DeclinedIf {
    /// Optional custom error code for this validation.
    /// If not provided, defaults to "declined".
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The path to a function that determines if this validation rule should apply.
    /// The function is expected to take `&self` (a reference to the struct instance)
    /// and return a `bool`.
    pub function: darling::Result<syn::Path>,
}

pub fn check_consent_validator_form(field: &ValidateField, field_name: &str) {
    if let Some(accepted_if) = &field.accepted_if
        && let Err(e) = &accepted_if.function
    {
        proc_macro_error2::abort!(
            e.span(),
            "invalid attribute #[validate(accepted_if(...))] on field `{}`. function must be a valid path like `function_name` or `path::to::function_name`",
            field_name
        );
    }

    if let Some(declined_if) = &field.declined_if
        && let Err(e) = &declined_if.function
    {
        proc_macro_error2::abort!(
            e.span(),
            "invalid attribute #[validate(declined_if(...))] on field `{}`. function must be a valid path like `function_name` or `path::to::function_name`",
            field_name
        );
    }
}

/// Generates the `TokenStream` for the `accepted` validation.
///
/// This code will be part of the `validate` method implementation for the derived struct.
/// It checks if `!field_name.accepted()` and adds an error if true.
///
/// The `ValidateConsent::accepted` method is expected to be available on the field's type.
pub fn accepted_tokens(
    crate_name: &CrateName,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
    params: Accepted,
) -> proc_macro2::TokenStream {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "accepted");

    quote! {
        if !#field_name.accepted() {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

/// Generates the `TokenStream` for the `declined` validation.
///
/// This code will be part of the `validate` method implementation for the derived struct.
/// It checks if `!field_name.declined()` and adds an error if true.
///
/// The `ValidateConsent::declined` method is expected to be available on the field's type.
pub fn declined_tokens(
    crate_name: &CrateName,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
    params: Declined,
) -> proc_macro2::TokenStream {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "declined");

    quote! {
        if !#field_name.declined() {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

/// Generates the `TokenStream` for the `accepted_if` validation.
///
/// This code will be part of the `validate` method implementation for the derived struct.
/// It checks if `condition_function(&self)` is true AND `!field_name.accepted()` is true.
/// If both are true, an error is added.
///
/// The `ValidateConsent::accepted` method is expected to be available on the field's type.
/// The `condition_function` is provided via the `function` field in `AcceptedIf`.
pub fn accepted_if_tokens(
    crate_name: &CrateName,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
    params: AcceptedIf,
) -> proc_macro2::TokenStream {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "accepted_if");

    // It's ok to unwrap here, `check_consent_validator_form` should have any caught errors.
    let func_call = params.function.unwrap();

    quote! {
        if #func_call(&self) && !#field_name.accepted() {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

/// Generates the `TokenStream` for the `declined_if` validation.
///
/// This code will be part of the `validate` method implementation for the derived struct.
/// It checks if `condition_function(&self)` is true AND `!field_name.declined()` is true.
/// If both are true, an error is added.
///
/// The `ValidateConsent::declined` method is expected to be available on the field's type.
/// The `condition_function` is provided via the `function` field in `DeclinedIf`.
pub fn declined_if_tokens(
    crate_name: &CrateName,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
    params: DeclinedIf,
) -> proc_macro2::TokenStream {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "declined_if");

    // It's ok to unwrap here, `check_consent_validator_form` should have any caught errors.
    let func_call = params.function.unwrap();

    quote! {
        if #func_call(&self) && !#field_name.declined() {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
