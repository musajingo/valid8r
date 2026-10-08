//! This module defines the data structures used by the `validator_derive`
//! procedural macro to parse and represent the `#[validate(...)]` attributes
//! applied to structs and their fields.
//!
//! It uses the `darling` crate for attribute parsing.

use darling::{FromField, util::Override};
use proc_macro_error2::abort;
use quote::{ToTokens, quote};
use std::sync::LazyLock;
use syn::{Field, Ident, spanned::Spanned};

use crate::utils::CrateName;
use crate::validators;

static OPTIONS_TYPE: [&str; 3] = ["Option|", "std|option|Option|", "core|option|Option|"];

/// Types that represent PATCH semantics (`Delta<T>`: Unchanged / Clear / Set).
/// These are treated as having 2 levels of Option nesting: only `Delta::Set(_)`
/// counts as a value to validate.
static DELTA_TYPE: [&str; 2] = ["Delta|", "delta|Delta|"];

// A static list of string representations of common numeric types,
// including their `Option`, `Option<Option>` and `Delta` variants.
//
// This is used to determine if a field's type is numeric for specific
// validation logic (e.g., how to pass arguments to custom validators).
pub(crate) static NUMBER_TYPES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let number_types = [
        quote!(usize),
        quote!(u8),
        quote!(u16),
        quote!(u32),
        quote!(u64),
        quote!(u128),
        quote!(isize),
        quote!(i8),
        quote!(i16),
        quote!(i32),
        quote!(i64),
        quote!(i128),
        quote!(f32),
        quote!(f64),
    ];
    let mut tys = Vec::with_capacity(number_types.len() * 4);
    for ty in number_types {
        tys.push(ty.to_string());
        tys.push(quote!(Option<#ty>).to_string());
        tys.push(quote!(Option<Option<#ty> >).to_string());
        tys.push(quote!(Delta<#ty>).to_string());
    }
    tys
});

/// This struct holds all the validation information parsed from a single field's
/// `#[validate(...)]` attributes.
///
/// The `ident` and `ty` fields are automatically populated by `darling`.
/// The other fields correspond to the specific validation attributes like `#[validate(email(...))]`.
#[derive(Clone, Debug, FromField)]
#[darling(attributes(validate))]
pub struct ValidateField {
    pub ty: syn::Type,

    pub ident: Option<syn::Ident>,

    pub credit_card: Option<Override<validators::cards::Card>>,

    /// Parsed `#[validate(contains(...))]` attribute.
    pub contains: Option<validators::contains::Contains>,

    /// Parsed `#[validate(does_not_contain(...))]` attribute.
    pub does_not_contain: Option<validators::does_not_contain::DoesNotContain>,

    /// Parsed `#[validate(email(...))]` attribute. `Override` handles `#[validate(email)]`.
    pub email: Option<Override<validators::email::Email>>,

    /// Parsed `#[validate(ip(...))]` attribute. `Override` handles `#[validate(ip)]`.
    pub ip: Option<Override<validators::ip::Ip>>,

    /// Parsed `#[validate(length(...))]` attribute.
    pub length: Option<validators::length::Length>,

    /// Parsed `#[validate(must_match(...))]` attribute.
    pub must_match: Option<validators::must_match::MustMatch>,

    /// Parsed `#[validate(non_control_character(...))]` attribute. `Override` handles `#[validate(non_control_character)]`.
    pub non_control_character:
        Option<Override<validators::non_control_character::NonControlCharacter>>,

    /// Parsed `#[validate(range(...))]` attribute.
    pub range: Option<validators::range::Range>,

    /// Parsed `#[validate(required(...))]` attribute. `Override` handles `#[validate(required)]`.
    pub required: Option<Override<validators::required::Required>>,

    /// Parsed `#[validate(require_if(func = ...))]` attribute.
    pub required_if: Option<validators::required::RequiredIf>,

    /// Parsed `#[validate(require_with(field1, field2,...))]` attribute.
    pub required_with: Option<validators::required::RequiredWith>,

    /// Parsed `#[validate(require_with_all(field1, field2,...))]` attribute.
    pub required_with_all: Option<validators::required::RequiredWithAll>,

    /// Parsed `#[validate(require_without(field1, field2,...))]` attribute.
    pub required_without: Option<validators::required::RequiredWithout>,

    /// Parsed `#[validate(require_without_all(field1, field2,...))]` attribute.
    pub required_without_all: Option<validators::required::RequiredWithoutAll>,

    /// Parsed `#[validate(prohibited_if(func = ...))]` attribute.
    pub prohibited_if: Option<validators::prohibited::ProhibitedIf>,

    /// Parsed `#[validate(prohibited_with(field1, field2,...))]` attribute.
    pub prohibited_with: Option<validators::prohibited::ProhibitedWith>,

    /// Parsed `#[validate(prohibited_with_all(field1, field2,...))]` attribute.
    pub prohibited_with_all: Option<validators::prohibited::ProhibitedWithAll>,

    /// Parsed `#[validate(prohibited_without(field1, field2,...))]` attribute.
    pub prohibited_without: Option<validators::prohibited::ProhibitedWithout>,

    /// Parsed `#[validate(prohibited_without_all(field1, field2,...))]` attribute.
    pub prohibited_without_all: Option<validators::prohibited::ProhibitedWithoutAll>,

    /// Parsed `#[validate(url(...))]` attribute. `Override` handles `#[validate(url)]`.
    pub url: Option<Override<validators::url::Url>>,

    /// Parsed `#[validate(regex(...))]` attribute.
    pub regex: Option<validators::regex::Regex>,

    /// Parsed `#[validate(custom(...))]` attributes (multiple allowed).
    #[darling(multiple)]
    pub custom: Vec<validators::custom::Custom>,

    /// Parsed `#[validate(skip)]` attribute.
    pub skip: Option<bool>,

    /// Parsed `#[validate(sensitive)]` attribute: the field's value is stripped
    /// from every error it produces, so passwords and one-time codes never reach
    /// a 422 body or a log line.
    pub sensitive: Option<bool>,

    /// Parsed `#[validate(nested)]` attribute.
    pub nested: Option<bool>,

    /// Parsed `#[validate(phone_number(...))]`
    pub phone_number: Option<Override<validators::phone_number::PhoneNumber>>,

    /// Parsed `#[validate(accepted(...))]`
    pub accepted: Option<Override<validators::consent::Accepted>>,

    /// Parsed `#[validate(declined(...))]`
    pub declined: Option<Override<validators::consent::Declined>>,

    /// Parsed `#[validate(accepted_if(...))]`
    pub accepted_if: Option<validators::consent::AcceptedIf>,

    /// Parsed `#[validate(declined_if(...))]`
    pub declined_if: Option<validators::consent::DeclinedIf>,

    /// Placeholder for the crate name, filled in by the [`ValidationData`](crate::ValidationData) value.
    /// This is marked `#[darling(skip)]` because it's not parsed from attributes
    /// on the field itself, but inherited from the struct-level `#[validate(crate = "...")]`
    /// or the default.
    #[darling(skip)]
    pub crate_name: CrateName,
}

impl ValidateField {
    /// Performs validation checks on the parsed field attributes.
    ///
    /// This method is called by `ValidationData::validate` after parsing.
    /// It checks for common errors like conflicting attributes or missing arguments.
    ///
    /// # Arguments
    /// - `struct_ident`: The identifier of the struct the field belongs to.
    /// - `all_fields`: A slice of all fields in the struct (original `syn::Field`s).
    /// - `current_field`: The original `syn::Field` for this `ValidateField`.
    pub fn validate(&self, struct_ident: &Ident, all_fields: &[&Field], current_field: &Field) {
        let field_name = self
            .ident
            .clone()
            .expect("expected field to be a named field")
            .to_string();

        let field_attrs = &current_field.attrs;
        for attr in field_attrs {
            if attr.path().is_ident("validate") && matches!(attr.meta, syn::Meta::Path(_)) {
                abort!(
                    current_field.span(), "expected at least one validator on field `{}`", field_name;
                    note = "If you want nested validation, use `#[validate(nested)]`"
                )
            }
        }

        // Check `custom` validator.
        validators::custom::check_custom_validator_form(self, &field_name);

        // Check `length` validator.
        validators::length::check_length_validator_form(self, &field_name, field_attrs);

        // Validate `must_match` validator.
        validators::must_match::check_must_match_validator_form(
            self,
            &field_name,
            struct_ident,
            all_fields,
        );

        // Validate `range` validator.
        validators::range::check_range_validator_form(self, &field_name, field_attrs);

        // Validate `phone_number` validator
        validators::phone_number::check_phone_number_validator_form(self);

        // Validate `accepted_if` and `decline_if` validators
        validators::consent::check_consent_validator_form(self, &field_name);

        // validate `required_if`, `required_with`, `required_with_all`, `required_without`, `required_without_all`
        validators::required::check_required_validator_form(
            self,
            &field_name,
            field_attrs,
            all_fields,
        );

        // validate `prohibited_if`, `prohibited_with`, `prohibited_with_all`, `prohibited_without`, `prohibited_without_all`
        validators::prohibited::check_prohibited_validator_form(
            self,
            &field_name,
            field_attrs,
            all_fields,
        );
    }

    /// Determines how many levels of `Option` wrap the field's actual type.
    /// How many Option<Option< are there before the actual field
    ///
    /// This is used to generate the correct `if let Some(...) = ...` pattern
    /// for validating optional fields.
    /// Returns the number of `Option` wrappers (0, 1, or 2). Aborts for more than 2.
    ///
    /// Note: `Delta<T>` is treated as equivalent to `Option<Option<T>>` (returns 2).
    pub fn number_options(&self) -> u8 {
        fn find_option(mut count: u8, ty: &syn::Type) -> u8 {
            if let syn::Type::Path(p) = ty {
                let idents_of_path = p.path.segments.iter().fold(String::new(), |mut acc, v| {
                    acc.push_str(&v.ident.to_string());
                    acc.push('|');
                    acc
                });

                // Check for Delta<T> first - it has three states (Unchanged/Clear/Set),
                // so it counts as 2 levels of optionality.
                if DELTA_TYPE.contains(&idents_of_path.as_str()) {
                    return 2;
                }

                if OPTIONS_TYPE.contains(&idents_of_path.as_str()) {
                    count += 1;
                    if let Some(p) = p.path.segments.first()
                        && let syn::PathArguments::AngleBracketed(ref params) = p.arguments
                        && let syn::GenericArgument::Type(ty) = params.args.first().unwrap()
                    {
                        count = find_option(count, ty);
                    }
                }
            }
            count
        }

        find_option(0, &self.ty)
    }

    /// Returns `true` if the field's type is `Delta<T>`.
    ///
    /// `Delta<T>` is an enum, so its inner value is reached through `.value()`
    /// (which returns `Option<&T>`) rather than through `Some(Some(_))` patterns.
    fn is_delta_type(&self) -> bool {
        if let syn::Type::Path(p) = &self.ty {
            let idents_of_path = p.path.segments.iter().fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
            return DELTA_TYPE.contains(&idents_of_path.as_str());
        }
        false
    }

    /// Generates the token stream for accessing the field value, potentially
    /// unwrapping `Option`s, and provides a closure to wrap validation logic
    /// in the necessary `if let Some(...)` blocks.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `proc_macro2::TokenStream`: The path to the actual value being validated (e.g., `self.field_name` or `field_name` after `if let`).
    /// 2. `Box<dyn Fn(proc_macro2::TokenStream) -> proc_macro2::TokenStream>`: A closure that takes the validation logic tokens
    ///    and wraps them in the appropriate `if let Some(...)` block(s) based on the number of `Option` wrappers.
    pub fn if_let_option_wrapper(
        &self,
        field_name: &Ident,
        is_number_type: bool,
    ) -> (
        proc_macro2::TokenStream,
        Box<dyn Fn(proc_macro2::TokenStream) -> proc_macro2::TokenStream>,
    ) {
        let number_options = self.number_options();
        let is_delta = self.is_delta_type();
        let field_name = field_name.clone();
        let actual_field = if number_options > 0 {
            quote!(#field_name)
        } else {
            quote!(self.#field_name)
        };
        let binding_pattern = if is_number_type {
            quote!(#field_name)
        } else {
            quote!(ref #field_name)
        };

        match number_options {
            0 => (actual_field.clone(), Box::new(move |tokens| tokens)),
            1 => (
                actual_field.clone(),
                Box::new(move |tokens| {
                    quote!(
                        if let Some(#binding_pattern) = self.#field_name {
                            #tokens
                        }
                    )
                }),
            ),
            2 => (
                actual_field.clone(),
                Box::new(move |tokens| {
                    // For Delta<T>, reach the inner value through `.value()`, which
                    // returns Option<&T>: Some(&v) for Delta::Set(v), None otherwise.
                    // The plain binding on Option<&T> yields a &T, matching what the
                    // `ref` binding yields in the Option<Option<T>> branch; numeric
                    // types (Copy) destructure the reference so they bind by value,
                    // matching the Option<Option<T>> calling convention.
                    if is_delta {
                        let delta_binding = if is_number_type {
                            quote!(&#field_name)
                        } else {
                            quote!(#field_name)
                        };
                        quote!(
                            if let Some(#delta_binding) = self.#field_name.value() {
                                #tokens
                            }
                        )
                    } else {
                        quote!(
                            if let Some(Some(#binding_pattern)) = self.#field_name {
                                #tokens
                            }
                        )
                    }
                }),
            ),
            _ => abort!(
                field_name.span(),
                "Validation on values nested in more than 2 Option are not supported"
            ),
        }
    }
}

impl ValidateField {
    /// Generates the `TokenStream` for all validation rules applied to a single field.
    ///
    /// This method iterates through all the validation attributes parsed for a field
    /// (e.g., `length`, `email`, `custom`, etc.) and calls the corresponding
    /// token-generating functions from the validator's module.
    ///
    /// It handles `Option` wrappers by generating code that only validates `Some(value)`.
    ///
    /// # Arguments
    /// - `all_fields`: A slice of all fields in the struct (needed for cross-field validators
    ///   like `required_with` and `prohibited_with` to detect Option nesting levels).
    pub fn generate_tokens(&self, all_fields: &[&Field]) -> proc_macro2::TokenStream {
        let field_name = self.ident.clone().unwrap();

        // Get the field name as a string, used for error messages and keys.
        let field_name_str = self.ident.clone().unwrap().to_string();

        // Get the type of the field as a string to check if it's a number or Cow.
        let type_name = self.ty.to_token_stream().to_string();
        let is_number = NUMBER_TYPES.contains(&type_name);

        // Call `if_let_option_wrapper` to get two things:
        // 1. `actual_field`: A `TokenStream` that represents the path to the actual
        //    value to be validated. This handles cases where the field might be
        //    already unwrapped from an `Option` (e.g., if it's `Option<Option<T>>`,
        //    `actual_field` would refer to the inner `T` after unwrapping).
        // 2. `wrapper_closure`: A closure that takes a `TokenStream` (the validation logic)
        //    and wraps it in the necessary `if let Some(...) = ... { ... }` blocks
        //    if the field is an `Option` or `Option<Option<T>>`. This ensures validation
        //    is only performed on `Some` values.
        let (actual_field, wrapper_closure) = self.if_let_option_wrapper(&field_name, is_number);

        // ---- Length Validation ---- //

        // Generate token stream for `length` validation if `#[validate(length(...))]` is present.
        let length = if let Some(length) = self.length.clone() {
            // If `length` validation is specified, generate the validation tokens
            // using `validators::length::tokens`.
            // The `wrapper_closure` ensures this logic is correctly placed inside
            // `if let Some(...)` blocks if the field is optional.
            wrapper_closure(validators::length::tokens(
                &self.crate_name, // The path to the validator crate (e.g., `::validator`).
                length,           // The parsed `Length` validator data.
                &actual_field,    // The token stream to access the field's value.
                &field_name_str,  // The name of the field as a string, for error messages.
            ))
        } else {
            quote!()
        };

        // ---- Email validation ---- //

        // The `Override` type from `darling` allows attributes to be specified as `#[validate(email)]` (Inherit, uses default)
        // or `#[validate(email(message = "...", code = "..."))]` (Explicit).
        let email = if let Some(email) = self.email.clone() {
            wrapper_closure(validators::email::tokens(
                &self.crate_name,
                match email {
                    Override::Inherit => validators::email::Email::default(),
                    Override::Explicit(e) => e,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Credit card validation ---- //

        // Similar pattern for other simple validations: check if the attribute is present, then generate tokens.
        let card = if let Some(credit_card) = self.credit_card.clone() {
            wrapper_closure(validators::cards::tokens(
                &self.crate_name,
                match credit_card {
                    Override::Inherit => validators::cards::Card::default(),
                    Override::Explicit(c) => c,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Url validation ---- //

        let url = if let Some(url) = self.url.clone() {
            wrapper_closure(validators::url::tokens(
                &self.crate_name,
                match url {
                    Override::Inherit => validators::url::Url::default(),
                    Override::Explicit(u) => u,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Ip address validation --- //

        let ip = if let Some(ip) = self.ip.clone() {
            wrapper_closure(validators::ip::tokens(
                &self.crate_name,
                match ip {
                    Override::Inherit => validators::ip::Ip::default(),
                    Override::Explicit(i) => i,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Non control character validation ---- //

        let ncc = if let Some(ncc) = self.non_control_character.clone() {
            wrapper_closure(validators::non_control_character::tokens(
                &self.crate_name,
                match ncc {
                    Override::Inherit => {
                        validators::non_control_character::NonControlCharacter::default()
                    }
                    Override::Explicit(n) => n,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Range validation ---- //

        let range = if let Some(range) = self.range.clone() {
            wrapper_closure(validators::range::tokens(
                &self.crate_name,
                range,
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Required validation ---- //

        let required = if let Some(required) = self.required.clone() {
            validators::required::required_tokens(
                &self.crate_name,
                match required {
                    Override::Inherit => validators::required::Required::default(),
                    Override::Explicit(r) => r,
                },
                &field_name,
                &field_name_str,
                all_fields,
            )
        } else {
            quote!()
        };

        let required_if = if let Some(required_if) = self.required_if.clone() {
            validators::required::required_if_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                required_if,
                all_fields,
            )
        } else {
            quote!()
        };

        let required_with = if let Some(required_with) = self.required_with.clone() {
            validators::required::required_with_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                required_with,
                all_fields,
            )
        } else {
            quote!()
        };

        let required_with_all = if let Some(required_with_all) = self.required_with_all.clone() {
            validators::required::required_with_all_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                required_with_all,
                all_fields,
            )
        } else {
            quote!()
        };

        let required_without = if let Some(required_without) = self.required_without.clone() {
            validators::required::required_without_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                required_without,
                all_fields,
            )
        } else {
            quote!()
        };

        let required_without_all =
            if let Some(required_without_all) = self.required_without_all.clone() {
                validators::required::required_without_all_tokens(
                    &self.crate_name,
                    &field_name,
                    &field_name_str,
                    required_without_all,
                    all_fields,
                )
            } else {
                quote!()
            };

        // ---- Prohibited validation ---- //

        let prohibited_if = if let Some(prohibited_if) = self.prohibited_if.clone() {
            validators::prohibited::prohibited_if_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                prohibited_if,
                all_fields,
            )
        } else {
            quote!()
        };

        let prohibited_with = if let Some(prohibited_with) = self.prohibited_with.clone() {
            validators::prohibited::prohibited_with_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                prohibited_with,
                all_fields,
            )
        } else {
            quote!()
        };

        let prohibited_with_all =
            if let Some(prohibited_with_all) = self.prohibited_with_all.clone() {
                validators::prohibited::prohibited_with_all_tokens(
                    &self.crate_name,
                    &field_name,
                    &field_name_str,
                    prohibited_with_all,
                    all_fields,
                )
            } else {
                quote!()
            };

        let prohibited_without = if let Some(prohibited_without) = self.prohibited_without.clone() {
            validators::prohibited::prohibited_without_tokens(
                &self.crate_name,
                &field_name,
                &field_name_str,
                prohibited_without,
                all_fields,
            )
        } else {
            quote!()
        };

        let prohibited_without_all =
            if let Some(prohibited_without_all) = self.prohibited_without_all.clone() {
                validators::prohibited::prohibited_without_all_tokens(
                    &self.crate_name,
                    &field_name,
                    &field_name_str,
                    prohibited_without_all,
                    all_fields,
                )
            } else {
                quote!()
            };

        // ---- Contains validation ---- //

        let contains = if let Some(contains) = self.contains.clone() {
            wrapper_closure(validators::contains::tokens(
                &self.crate_name,
                contains,
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Does not contain validation ---- //

        let does_not_contain = if let Some(does_not_contain) = self.does_not_contain.clone() {
            wrapper_closure(validators::does_not_contain::tokens(
                &self.crate_name,
                does_not_contain,
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // --- Must match validation --- //

        let must_match = if let Some(must_match) = self.must_match.clone() {
            // must_match normalizes both fields to Option<&T> itself (covering
            // Option, Option<Option> and Delta on either side), so it is not
            // wrapped by `wrapper_closure`.
            validators::must_match::tokens(
                &self.crate_name,
                must_match,
                &field_name,
                &field_name_str,
                all_fields,
            )
        } else {
            quote!()
        };

        // ---- Regex validation ---- //

        let regex = if let Some(regex) = self.regex.clone() {
            wrapper_closure(validators::regex::tokens(
                &self.crate_name,
                regex,
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Phone number validation ---- //

        let phone_number = if let Some(phone_number) = self.phone_number.clone() {
            wrapper_closure(validators::phone_number::tokens(
                &self.crate_name,
                match phone_number {
                    Override::Inherit => validators::phone_number::PhoneNumber::default(),
                    Override::Explicit(pn) => pn,
                },
                &actual_field,
                &field_name_str,
            ))
        } else {
            quote!()
        };

        // ---- Consent validation ---- //

        let accepted = if let Some(accepted) = self.accepted.clone() {
            wrapper_closure(validators::consent::accepted_tokens(
                &self.crate_name,
                &actual_field,
                &field_name_str,
                match accepted {
                    Override::Inherit => validators::consent::Accepted::default(),
                    Override::Explicit(e) => e,
                },
            ))
        } else {
            quote!()
        };

        let declined = if let Some(declined) = self.declined.clone() {
            wrapper_closure(validators::consent::declined_tokens(
                &self.crate_name,
                &actual_field,
                &field_name_str,
                match declined {
                    Override::Inherit => validators::consent::Declined::default(),
                    Override::Explicit(e) => e,
                },
            ))
        } else {
            quote!()
        };

        let accepted_if = if let Some(accepted_if) = self.accepted_if.clone() {
            wrapper_closure(validators::consent::accepted_if_tokens(
                &self.crate_name,
                &actual_field,
                &field_name_str,
                accepted_if,
            ))
        } else {
            quote!()
        };

        let declined_if = if let Some(declined_if) = self.declined_if.clone() {
            wrapper_closure(validators::consent::declined_if_tokens(
                &self.crate_name,
                &actual_field,
                &field_name_str,
                declined_if,
            ))
        } else {
            quote!()
        };

        // ---- Custom validation ---- //

        let mut custom = quote!();
        // For custom validation functions, we try to be smart about how arguments are passed.
        // If the field type is `Cow<'_, T>`, we pass `field.as_ref()`.
        let is_cow = type_name.contains("Cow <");
        let custom_actual_field = if is_cow {
            quote!(#actual_field.as_ref())
        } else if is_number || type_name.starts_with("&") {
            quote!(#actual_field)
        } else {
            quote!(&#actual_field)
        };

        // Iterate over all `custom` attributes for the field.
        for c in &self.custom {
            let tokens =
                validators::custom::tokens(c.clone(), &custom_actual_field, &field_name_str);
            custom = quote!(
                #custom

                #tokens
            );
        }
        // If there were any custom validations, wrap them with the option handler.
        if !self.custom.is_empty() {
            custom = wrapper_closure(custom);
        }

        // Nested validation: if `#[validate(nested)]` is present.
        let nested = if let Some(n) = self.nested {
            if n {
                wrapper_closure(validators::nested::tokens(&actual_field, &field_name_str))
            } else {
                quote!()
            }
        } else {
            quote!()
        };

        let redact = if self.sensitive.unwrap_or(false) {
            quote! { errors.redact_value(#field_name_str); }
        } else {
            quote!()
        };

        quote! {
            #length
            #email
            #card
            #url
            #ip
            #ncc
            #range
            #required
            #required_if
            #required_with
            #required_with_all
            #required_without
            #required_without_all
            #prohibited_if
            #prohibited_with
            #prohibited_with_all
            #prohibited_without
            #prohibited_without_all
            #contains
            #does_not_contain
            #must_match
            #regex
            #phone_number
            #accepted
            #accepted_if
            #declined
            #declined_if
            #custom
            #nested
            #redact
        }
    }
}

impl quote::ToTokens for ValidateField {
    /// Backwards-compatible token generation that doesn't support Option<Option<T>> for cross-field validators.
    /// Use `generate_tokens(all_fields)` instead for full Option<Option<T>> support.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Call generate_tokens with an empty slice for backwards compatibility
        // Note: This won't correctly detect Option<Option<T>> for required_with/prohibited_with
        // but maintains existing behavior for simple validations
        tokens.extend(self.generate_tokens(&[]));
    }
}
