//! This module handles the parsing and token generation for the `#[validate(required)]` attribute.
//!
//! The `required` validator ensures that an `Option` or other type implementing
//! `ValidateRequired` contains a value (i.e., is not `None` or its equivalent).

use darling::FromMeta;
use proc_macro_error2::abort;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Attribute, Field, Ident, LitStr, Type, spanned::Spanned};

use crate::ValidateField;
use crate::utils::{
    CrateName, field_optionality, generate_has_value_check, generate_presence_check, get_attr,
    is_optionish_type, quote_code, quote_message,
};

/// Represents the arguments parsed from a `#[validate(required(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to optionally specify a custom error `message` and `code`
/// for the `required` validation. If not provided, default values will be used.
/// The `Default` derive is used when `#[validate(required)]` is specified without arguments.
#[derive(Debug, Clone, FromMeta, Default)]
pub struct Required {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,
}

/// The field under validation must be Some if the function returns true
#[derive(Debug, Clone, FromMeta)]
pub struct RequiredIf {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    pub func: darling::Result<syn::Path>,
}

/// The field under validation must be Some if any of the other fields are Some
#[derive(Debug, Clone, FromMeta)]
pub struct RequiredWith {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be Some if all of the other fields are Some
#[derive(Debug, Clone, FromMeta)]
pub struct RequiredWithAll {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be Some if any of the other fields are None
#[derive(Debug, Clone, FromMeta)]
pub struct RequiredWithout {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be Some if all of the other fields are None
#[derive(Debug, Clone, FromMeta)]
pub struct RequiredWithoutAll {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields
    pub other_fields: Vec<LitStr>,
}

pub fn check_required_validator_form(
    field: &ValidateField,
    field_name: &str,
    field_attrs: &[Attribute],
    all_fields: &[&Field],
) {
    // Check for `required_if` function path validity
    if let Some(required_if) = &field.required_if
        && let Err(e) = &required_if.func
    {
        abort!(
            e.span(),
            "Invalid attribute #[validate(required_if(...))] on field `{}`. `func` must be a valid path like `function_name` or `path::to::function_name`",
            field_name
        );
    }

    // Determine the error span for attributes like required_with etc.
    // This provides a good fallback span for errors related to the attribute itself.
    let error_span = get_attr(field_attrs, "required_with")
        .or_else(|| get_attr(field_attrs, "required_with_all"))
        .or_else(|| get_attr(field_attrs, "required_without"))
        .or_else(|| get_attr(field_attrs, "required_without_all"))
        .map_or_else(
            || {
                // If no required_*_with/without attributes, use the field's first attribute or call_site
                field_attrs
                    .first()
                    .map_or_else(proc_macro2::Span::call_site, |a| a.span())
            },
            |a| a.span(),
        );

    // Get the type of the main field being validated
    let main_field_type = &field.ty;

    // Helper closure to check and report if a field is not Option<T> or Delta<T>
    let check_option_type_and_abort = |span: proc_macro2::Span,
                                       current_field_name: &str,
                                       current_field_type: &Type,
                                       attribute_name: &str| {
        if !is_optionish_type(current_field_type) {
            abort!(
                span,
                "`#[validate({}(...))]` only works with fields of type `Option<T>` or `Delta<T>`, field `{}` is of type `{}`",
                attribute_name,
                current_field_name,
                current_field_type.to_token_stream()
            );
        }
    };

    // --- Check `required_with` ---
    if let Some(required_with) = &field.required_with {
        // 1. Check the type of the main field
        check_option_type_and_abort(error_span, field_name, main_field_type, "required_with");

        if required_with.other_fields.is_empty() {
            abort!(
                error_span,
                "invalid attribute #[validate(required_with(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(required_with(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &required_with.other_fields {
                // other_field here is a LitStr
                let other_field_name = other_field_litstr.value(); // Get the string name

                // Find the syn::Field corresponding to `other_field_name`
                let other_field_syn_field = all_fields
                    .iter()
                    .find(|f| f.ident.as_ref().is_some_and(|i| i == &other_field_name));

                if let Some(f) = other_field_syn_field {
                    // 2. Check the type of the 'other field'
                    check_option_type_and_abort(
                        other_field_litstr.span(),
                        &other_field_name,
                        &f.ty,
                        "required_with",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(), // Point to the specific other_field in the attribute
                        "Invalid attribute for #[required_with(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `required_with_all` ---
    if let Some(required_with_all) = &field.required_with_all {
        // 1. Check the type of the main field
        check_option_type_and_abort(error_span, field_name, main_field_type, "required_with_all");

        if required_with_all.other_fields.is_empty() {
            abort!(
                error_span,
                "invalid attribute #[validate(required_with_all(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(required_with_all(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &required_with_all.other_fields {
                let other_field_name = other_field_litstr.value();

                let other_field_syn_field = all_fields
                    .iter()
                    .find(|f| f.ident.as_ref().is_some_and(|i| i == &other_field_name));

                if let Some(f) = other_field_syn_field {
                    // 2. Check the type of the 'other field'
                    check_option_type_and_abort(
                        other_field_litstr.span(),
                        &other_field_name,
                        &f.ty,
                        "required_with_all",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[required_with_all(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `required_without` ---
    if let Some(required_without) = &field.required_without {
        // 1. Check the type of the main field
        check_option_type_and_abort(error_span, field_name, main_field_type, "required_without");

        if required_without.other_fields.is_empty() {
            abort!(
                error_span,
                "invalid attribute #[validate(required_without(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(required_without(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &required_without.other_fields {
                let other_field_name = other_field_litstr.value();

                let other_field_syn_field = all_fields
                    .iter()
                    .find(|f| f.ident.as_ref().is_some_and(|i| i == &other_field_name));

                if let Some(f) = other_field_syn_field {
                    // 2. Check the type of the 'other field'
                    check_option_type_and_abort(
                        other_field_litstr.span(),
                        &other_field_name,
                        &f.ty,
                        "required_without",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[required_without(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `required_without_all` ---
    if let Some(required_without_all) = &field.required_without_all {
        // 1. Check the type of the main field
        check_option_type_and_abort(
            error_span,
            field_name,
            main_field_type,
            "required_without_all",
        );

        if required_without_all.other_fields.is_empty() {
            abort!(
                error_span,
                "invalid attribute #[validate(required_without_all(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(required_without_all(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &required_without_all.other_fields {
                let other_field_name = other_field_litstr.value();

                let other_field_syn_field = all_fields
                    .iter()
                    .find(|f| f.ident.as_ref().is_some_and(|i| i == &other_field_name));

                if let Some(f) = other_field_syn_field {
                    // 2. Check the type of the 'other field'
                    check_option_type_and_abort(
                        other_field_litstr.span(),
                        &other_field_name,
                        &f.ty,
                        "required_without_all",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[required_without_all(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }
}

/// Generates the `proc_macro2::TokenStream` for the `required` validation logic.
///
/// This function takes the parsed `Required` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the check at runtime.
pub fn required_tokens(
    crate_name: &CrateName,
    required: Required,
    field_name: &Ident,
    field_name_str: &str,
    all_fields: &[&Field],
) -> TokenStream2 {
    // Prepare the custom message and code, if provided.
    let message = quote_message(required.message);
    let code = quote_code(crate_name, required.code, "required");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate the "has value" check for the field
    let has_value_check = generate_has_value_check(field_name, optionality);

    // Generate the token stream for the required validation.
    // For Option<Option<T>>, "has value" means Some(Some(_)), not just Some
    quote! {
        if !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn required_if_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: RequiredIf,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "required_if");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate the "has value" check for the field
    let has_value_check = generate_has_value_check(field_name, optionality);

    // It's ok to unwrap here, `check_consent_validator_form` should have any caught errors.
    let func_call = params.func.unwrap();

    quote! {
        if #func_call(&self) && !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn required_with_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: RequiredWith,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "required_with");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate the individual checks for each "other" field.
    // Uses appropriate presence check based on Option nesting level.
    let other_field_presence_checks: Vec<TokenStream2> = params
        .other_fields
        .iter()
        .map(|other_field| {
            let other_field_str = other_field.value();
            let other_field_ident = Ident::new(&other_field_str, other_field.span());

            // Classify the other field's optionality and generate the appropriate presence check
            let other_optionality = field_optionality(&other_field_str, all_fields);
            generate_presence_check(&other_field_ident, other_optionality, true)
        })
        .collect();

    // Combine these checks with the logical OR (`||`) operator.
    // If `other_field_presence_checks` is empty (no other fields listed),
    // then `should_be_required` will be `false`.
    let should_be_required = if other_field_presence_checks.is_empty() {
        quote! { false }
    } else {
        quote! {
            (#(#other_field_presence_checks)||*)
        }
    };

    // Generate the "has value" check for the main field
    let has_value_check = generate_has_value_check(field_name, optionality);

    quote! {
        // should_be_required is true if any of the other fields have a value
        // For Option<Option<T>>, "has value" means Some(Some(_)), not just Some
        if #should_be_required && !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn required_with_all_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: RequiredWithAll,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "required_with_all");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate individual checks for each "other" field.
    let other_field_presence_checks: Vec<TokenStream2> = params
        .other_fields
        .iter()
        .map(|other_field_litstr| {
            let other_field_str = other_field_litstr.value();
            let other_field_ident = Ident::new(&other_field_str, other_field_litstr.span());

            // Classify the other field's optionality and generate the appropriate presence check
            let other_optionality = field_optionality(&other_field_str, all_fields);
            generate_presence_check(&other_field_ident, other_optionality, true)
        })
        .collect();

    // Combine these checks with the logical AND (`&&`) operator.
    // The condition is true if ALL other fields have a value.
    let should_be_required_condition = if other_field_presence_checks.is_empty() {
        // If no other fields are listed, this condition is vacuously true for 'all'
        // This means the main field would always be required if the list is empty.
        // Usually, an empty list here would be an error caught by check_required_validator_form.
        quote! { true }
    } else {
        quote! {
            (#(#other_field_presence_checks)&&*)
        }
    };

    // Generate the "has value" check for the main field
    let has_value_check = generate_has_value_check(field_name, optionality);

    quote! {
        if #should_be_required_condition && !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn required_without_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: RequiredWithout,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "required_without");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate individual checks for each "other" field.
    let other_field_absence_checks: Vec<TokenStream2> = params
        .other_fields
        .iter()
        .map(|other_field_litstr| {
            let other_field_str = other_field_litstr.value();
            let other_field_ident = Ident::new(&other_field_str, other_field_litstr.span());

            // Classify the other field's optionality and generate the appropriate absence check
            let other_optionality = field_optionality(&other_field_str, all_fields);
            generate_presence_check(&other_field_ident, other_optionality, false)
        })
        .collect();

    // Combine these checks with the logical OR (`||`) operator.
    // The condition is true if ANY other field is absent (has no value).
    let should_be_required_condition = if other_field_absence_checks.is_empty() {
        // If no other fields are listed, this condition is false for 'any'
        // The main field would never be required based on absence.
        quote! { false }
    } else {
        quote! {
            (#(#other_field_absence_checks)||*)
        }
    };

    // Generate the "has value" check for the main field
    let has_value_check = generate_has_value_check(field_name, optionality);

    quote! {
        // Main field is required if ANY other field is absent, AND main field has no value.
        // For Option<Option<T>>, "absent" means !matches!(field, Some(Some(_)))
        if #should_be_required_condition && !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn required_without_all_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: RequiredWithoutAll,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "required_without_all");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate individual checks for each "other" field.
    let other_field_absence_checks: Vec<TokenStream2> = params
        .other_fields
        .iter()
        .map(|other_field_litstr| {
            let other_field_str = other_field_litstr.value();
            let other_field_ident = Ident::new(&other_field_str, other_field_litstr.span());

            // Classify the other field's optionality and generate the appropriate absence check
            let other_optionality = field_optionality(&other_field_str, all_fields);
            generate_presence_check(&other_field_ident, other_optionality, false)
        })
        .collect();

    // Combine these checks with the logical AND (`&&`) operator.
    // The condition is true if ALL other fields are absent (have no value).
    let should_be_required_condition = if other_field_absence_checks.is_empty() {
        // If no other fields are listed, this condition is vacuously true for 'all'
        // This means the main field would always be required if the list is empty.
        quote! { true }
    } else {
        quote! {
            (#(#other_field_absence_checks)&&*)
        }
    };

    // Generate the "has value" check for the main field
    let has_value_check = generate_has_value_check(field_name, optionality);

    quote! {
        // Main field is required if ALL other fields are absent, AND main field has no value.
        // For Option<Option<T>>, "absent" means !matches!(field, Some(Some(_)))
        if #should_be_required_condition && !#has_value_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
