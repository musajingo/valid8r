//! This module handles parsing and token generation for "prohibited" conditional validation attributes
//! like `#[validate(prohibited_if(...))]`, `#[validate(prohibited_with(...))]`, etc.
//!
//! These validators ensure that a field (typically an `Option` or type implementing
//! `valid8r::ValidateProhibited` is absent (e.g., `None`) if certain conditions involving other fields or functions are met.

use darling::FromMeta;
use proc_macro_error2::abort;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Attribute, Field, Ident, LitStr, Type, spanned::Spanned};

use crate::ValidateField;
use crate::utils::{
    CrateName, field_optionality, generate_is_prohibited_check, generate_presence_check, get_attr,
    is_optionish_type, quote_code, quote_message,
};

/// The field under validation must be None if the function returns true.
#[derive(Debug, Clone, FromMeta)]
pub struct ProhibitedIf {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The function to call to determine if the prohibition condition is met.
    pub func: darling::Result<syn::Path>,
}

/// The field under validation must be None if any of the other specified fields are Some.
#[derive(Debug, Clone, FromMeta)]
pub struct ProhibitedWith {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields to check for presence.
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be None if all of the other specified fields are Some.
#[derive(Debug, Clone, FromMeta)]
pub struct ProhibitedWithAll {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields to check for presence.
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be None if any of the other specified fields are None.
#[derive(Debug, Clone, FromMeta)]
pub struct ProhibitedWithout {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields to check for absence.
    pub other_fields: Vec<LitStr>,
}

/// The field under validation must be Some if all of the other fields are None
#[derive(Debug, Clone, FromMeta)]
pub struct ProhibitedWithoutAll {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// The other fields to check for absence.
    pub other_fields: Vec<LitStr>,
}

pub fn check_prohibited_validator_form(
    field: &ValidateField,
    field_name: &str,
    field_attrs: &[Attribute],
    all_fields: &[&Field],
) {
    // Check for `prohibited_if` function path validity
    if let Some(prohibited_if) = &field.prohibited_if
        && let Err(e) = &prohibited_if.func
    {
        abort!(
            e.span(),
            "Invalid attribute #[validate(prohibited_if(...))] on field `{}`. `func` must be a valid path like `function_name` or `path::to::function_name`",
            field_name
        );
    }

    // Determine the error span for attributes like prohibited_with etc.
    // This provides a good fallback span for errors related to the attribute itself.
    let error_span = get_attr(field_attrs, "prohibited_with")
        .or_else(|| get_attr(field_attrs, "prohibited_with_all"))
        .or_else(|| get_attr(field_attrs, "prohibited_without"))
        .or_else(|| get_attr(field_attrs, "prohibited_without_all"))
        .map_or_else(
            || {
                // If no prohibited_*_with/without attributes, use the field's first attribute or call_site
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
                span, // Use the specific attribute's span if available, otherwise the general error_span
                "`#[validate({}(...))]` for prohibited checks expects fields of type `Option<T>` or `Delta<T>`, but field `{}` is of type `{}`",
                attribute_name,
                current_field_name,
                current_field_type.to_token_stream()
            );
        }
    };

    // --- Check `prohibited_with` ---
    if let Some(prohibited_with) = &field.prohibited_with {
        // 1. Check the type of the main field
        check_option_type_and_abort(error_span, field_name, main_field_type, "prohibited_with");

        if prohibited_with.other_fields.is_empty() {
            abort!(
                error_span,
                "Invalid attribute #[validate(prohibited_with(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(prohibited_with(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &prohibited_with.other_fields {
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
                        "prohibited_with",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(), // Point to the specific other_field in the attribute
                        "Invalid attribute for #[prohibited_with(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `prohibited_with_all` ---
    if let Some(prohibited_with_all) = &field.prohibited_with_all {
        // 1. Check the type of the main field
        check_option_type_and_abort(
            error_span,
            field_name,
            main_field_type,
            "prohibited_with_all",
        );

        if prohibited_with_all.other_fields.is_empty() {
            abort!(
                error_span,
                "Invalid attribute #[validate(prohibited_with_all(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(prohibited_with_all(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &prohibited_with_all.other_fields {
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
                        "prohibited_with_all",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[prohibited_with_all(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `prohibited_without` ---
    if let Some(prohibited_without) = &field.prohibited_without {
        // 1. Check the type of the main field
        check_option_type_and_abort(
            error_span,
            field_name,
            main_field_type,
            "prohibited_without",
        );

        if prohibited_without.other_fields.is_empty() {
            abort!(
                error_span,
                "Invalid attribute #[validate(prohibited_without(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(prohibited_without(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &prohibited_without.other_fields {
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
                        "prohibited_without",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[prohibited_without(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }

    // --- Check `prohibited_without_all` ---
    if let Some(prohibited_without_all) = &field.prohibited_without_all {
        // 1. Check the type of the main field
        check_option_type_and_abort(
            error_span,
            field_name,
            main_field_type,
            "prohibited_without_all",
        );

        if prohibited_without_all.other_fields.is_empty() {
            abort!(
                error_span,
                "Invalid attribute #[validate(prohibited_without_all(...))] on field `{}`, other fields not specified. Specify other fields like #[validate(prohibited_without_all(other_field1, other_field2,...))]",
                field_name
            );
        } else {
            for other_field_litstr in &prohibited_without_all.other_fields {
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
                        "prohibited_without_all",
                    );
                } else {
                    abort!(
                        other_field_litstr.span(),
                        "Invalid attribute for #[prohibited_without_all(...)] on field `{}`, unknown other field `{}`",
                        field_name,
                        other_field_name
                    );
                }
            }
        }
    }
}

pub fn prohibited_if_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: ProhibitedIf,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "prohibited_if");

    // Classify the optionality of the field being validated
    let optionality = field_optionality(field_name_str, all_fields);

    // Generate the "is prohibited" check for the field
    let is_prohibited_check = generate_is_prohibited_check(field_name, optionality);

    // It's ok to unwrap here, `check_consent_validator_form` should have any caught errors.
    let func_call = params.func.unwrap();

    quote! {
        // Field is prohibited (must have no value) if func_call is true.
        // For Option<Option<T>>, "has value" means Some(Some(_))
        if #func_call(&self) && !#is_prohibited_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn prohibited_with_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: ProhibitedWith,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "prohibited_with");

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
    // then `prohibition_condition_met` will be `false`.
    let prohibition_condition_met = if other_field_presence_checks.is_empty() {
        quote! { false }
    } else {
        quote! {
            (#(#other_field_presence_checks)||*)
        }
    };

    // Generate the "is prohibited" check for the field
    let is_prohibited_check = generate_is_prohibited_check(field_name, optionality);

    quote! {
        // prohibition_condition_met is true if any of the other fields have a value.
        // For Option<Option<T>>, "has value" means Some(Some(_))
        if #prohibition_condition_met && !#is_prohibited_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn prohibited_with_all_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: ProhibitedWithAll,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "prohibited_with_all");

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
    let prohibition_condition_met = if other_field_presence_checks.is_empty() {
        // If no other fields are listed, this condition is vacuously true for 'all'
        // This means the main field would always be prohibited if the list is empty.
        // Usually, an empty list here would be an error caught by check_prohibited_validator_form.
        quote! { true }
    } else {
        quote! {
            (#(#other_field_presence_checks)&&*)
        }
    };

    // Generate the "is prohibited" check for the field
    let is_prohibited_check = generate_is_prohibited_check(field_name, optionality);

    quote! {
        // Field is prohibited if ALL other fields have a value AND field has a value.
        // For Option<Option<T>>, "has value" means Some(Some(_))
        if #prohibition_condition_met && !#is_prohibited_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn prohibited_without_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: ProhibitedWithout,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "prohibited_without");

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
    let prohibition_condition_met = if other_field_absence_checks.is_empty() {
        // If no other fields are listed, this condition is false for 'any'
        // The main field would never be prohibited based on absence of other fields.
        quote! { false }
    } else {
        quote! {
            (#(#other_field_absence_checks)||*)
        }
    };

    // Generate the "is prohibited" check for the field
    let is_prohibited_check = generate_is_prohibited_check(field_name, optionality);

    quote! {
        // Main field is prohibited if ANY other field is absent, AND main field has a value.
        // For Option<Option<T>>, "absent" means !matches!(field, Some(Some(_)))
        if #prohibition_condition_met && !#is_prohibited_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}

pub fn prohibited_without_all_tokens(
    crate_name: &CrateName,
    field_name: &Ident,
    field_name_str: &str,
    params: ProhibitedWithoutAll,
    all_fields: &[&Field],
) -> TokenStream2 {
    let message = quote_message(params.message);
    let code = quote_code(crate_name, params.code, "prohibited_without_all");

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
    let prohibition_condition_met = if other_field_absence_checks.is_empty() {
        // If no other fields are listed, this condition is vacuously true for 'all'
        // This means the main field would always be prohibited if the list is empty.
        quote! { true }
    } else {
        quote! {
            (#(#other_field_absence_checks)&&*)
        }
    };

    // Generate the "is prohibited" check for the field
    let is_prohibited_check = generate_is_prohibited_check(field_name, optionality);

    quote! {
        // Main field is prohibited if ALL other fields are absent, AND main field has a value.
        // For Option<Option<T>>, "absent" means !matches!(field, Some(Some(_)))
        if #prohibition_condition_met && !#is_prohibited_check {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &self.#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
