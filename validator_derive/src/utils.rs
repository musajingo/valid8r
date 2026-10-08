//! This module provides utility functions and types used by the `validator_derive`
//! procedural macro. This includes helpers for generating code snippets,
//! handling the validator crate name, and parsing attributes.

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Attribute, Field, Ident, Path, Type};

use crate::ValidateField;

/// A wrapper around `syn::Path` to represent the name of the `validator` crate.
/// This allows users to rename the `validator` crate in their `Cargo.toml`
/// and specify the new name via `#[validate(crate = "my_validator_crate")]`.
#[derive(Debug, Clone)]
pub struct CrateName {
    /// The actual path to the validator crate (e.g., `::validator` or `::my_validator_crate`).
    inner: Path,
}

impl ToTokens for CrateName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.inner.to_tokens(tokens);
    }
}

impl darling::FromMeta for CrateName {
    /// Parses the crate name from a string literal in an attribute.
    /// Example: `#[validate(crate = "my_validator")]`
    fn from_string(value: &str) -> darling::Result<Self> {
        Path::from_string(value).map(|inner| CrateName { inner })
    }

    /// Parses the crate name from a literal value in an attribute.
    /// This is typically used when the attribute is like `#[validate(crate = ::my_validator)]`
    /// (though less common for crate names).
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        Path::from_value(value).map(|inner| CrateName { inner })
    }

    /// Parses the crate name from an expression in an attribute.
    /// Example: `#[validate(crate = path::to::my_validator)]`
    fn from_expr(value: &syn::Expr) -> darling::Result<Self> {
        Path::from_expr(value).map(|inner| CrateName { inner })
    }
}

impl Default for CrateName {
    fn default() -> Self {
        // By default, assume the crate is named `validator` and accessible via `::validator`.
        CrateName {
            inner: syn::parse_str("::validator").expect("invalid valid crate name"),
        }
    }
}

/// Generates a token stream to set the `message` field of a `ValidationError`.
/// If `message` is `Some(m)`, it quotes `err.message = Some(Cow::from(m));`.
pub fn quote_message(message: Option<String>) -> proc_macro2::TokenStream {
    // If a custom message string is provided, generate code to assign it to the error.
    if let Some(m) = message {
        quote!(
            err.message = Some(::std::borrow::Cow::from(#m));
        )
    } else {
        quote!()
    }
}

/// Generates a token stream to create a `ValidationError` with a specific `code`.
/// If `code` is `Some(c)`, it uses `c`; otherwise, it uses the `default` code.
pub fn quote_code(
    crate_name: &CrateName,
    code: Option<String>,
    default: &str,
) -> proc_macro2::TokenStream {
    // If a custom error code string is provided, use it.
    if let Some(c) = code {
        quote!(
            let mut err = #crate_name::ValidationError::new(#c);
        )
    } else {
        quote!(
            let mut err = #crate_name::ValidationError::new(#default);
            // Otherwise, use the default error code for this type of validation.
        )
    }
}

/// This function generates `use` statements for various validation traits
/// (e.g., `ValidateLength`, `ValidateEmail`) based on which validations
/// are actually used in the struct's fields. This helps keep the generated code cleaner.
pub fn quote_use_statements(
    crate_name: &CrateName,
    fields: &[ValidateField],
) -> proc_macro2::TokenStream {
    let mut length = quote!();
    let mut email = quote!();
    let mut card = quote!();
    let mut url = quote!();
    let mut ip = quote!();
    let mut ncc = quote!();
    let mut range = quote!();
    let mut required = quote!();
    let mut contains = quote!();
    let mut does_not_contain = quote!();
    let mut regex = quote!();
    let mut phone_number = quote!();
    let mut consent = quote!();
    let mut prohibited = quote!();

    // Iterate over all fields that have validation attributes.
    for f in fields {
        if f.length.is_some() {
            length = quote!(
                use #crate_name::ValidateLength;
            );
        }

        if f.email.is_some() {
            email = quote!(
                use #crate_name::ValidateEmail;
            );
        }

        if f.credit_card.is_some() {
            card = quote!(
                use #crate_name::ValidateCreditCard;
            );
        }

        if f.url.is_some() {
            url = quote!(
                use #crate_name::ValidateUrl;
            );
        }

        if f.ip.is_some() {
            ip = quote!(
                use #crate_name::ValidateIp;
            );
        }

        if f.non_control_character.is_some() {
            ncc = quote!(
                use #crate_name::ValidateNonControlCharacter;
            );
        }

        if f.range.is_some() {
            range = quote!(
                use #crate_name::ValidateRange;
            );
        }

        if f.required.is_some()
            || f.required_if.is_some()
            || f.required_with.is_some()
            || f.required_with_all.is_some()
            || f.required_without.is_some()
            || f.required_without_all.is_some()
        {
            required = quote!(
                use #crate_name::ValidateRequired;
            );
        }

        if f.prohibited_if.is_some()
            || f.prohibited_with.is_some()
            || f.prohibited_with_all.is_some()
            || f.prohibited_without.is_some()
            || f.prohibited_without_all.is_some()
        {
            prohibited = quote!(
                use #crate_name::ValidateProhibited;
            );
        }

        if f.contains.is_some() {
            contains = quote!(
                use #crate_name::ValidateContains;
            );
        }

        if f.does_not_contain.is_some() {
            does_not_contain = quote!(
                use #crate_name::ValidateDoesNotContain;
            );
        }

        if f.regex.is_some() {
            regex = quote!(
                use #crate_name::ValidateRegex;
            );
        }

        if f.phone_number.is_some() {
            phone_number = quote!(
                use #crate_name::ValidatePhoneNumber;
            )
        }

        if f.accepted.is_some()
            || f.declined_if.is_some()
            || f.declined.is_some()
            || f.accepted_if.is_some()
        {
            consent = quote!(
                use #crate_name::ValidateConsent;
            );
        }
    }

    // Combine all necessary `use` statements.
    quote!(
        #length
        #email
        #card
        #url
        #ip
        #ncc
        #range
        #required
        #prohibited
        #contains
        #does_not_contain
        #regex
        #phone_number
        #consent
    )
}

/// How a field's type wraps its inner value, for presence/absence purposes.
///
/// Cross-field validators (`required_with`, `prohibited_with`, ...) and the
/// `required`/`prohibited` checks themselves need to know what "has a value"
/// means for a given field. For PATCH payloads, a field that is being
/// *cleared* (explicit null) must not count as having a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldOptionality {
    /// Not an optional type — always counts as present.
    NotOptional,
    /// `Option<T>` — present when `Some(_)`.
    Option,
    /// `Option<Option<T>>` — PATCH semantics; present only when `Some(Some(_))`.
    OptionOption,
    /// `Delta<T>` — PATCH semantics; present only when `Delta::Set(_)`.
    Delta,
}

/// Classifies a type by how it wraps its inner value.
///
/// Detection is purely syntactic (by the type path's last segment), like the
/// rest of the derive: `Option`, `Option<Option<...>>` and `Delta` are
/// recognized; everything else is `NotOptional`.
pub fn type_optionality(ty: &Type) -> FieldOptionality {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        if segment.ident == "Delta" {
            return FieldOptionality::Delta;
        }

        if segment.ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                && matches!(
                    type_optionality(inner),
                    FieldOptionality::Option | FieldOptionality::OptionOption
                )
            {
                return FieldOptionality::OptionOption;
            }
            return FieldOptionality::Option;
        }
    }
    FieldOptionality::NotOptional
}

/// Returns `true` if the type participates in presence semantics at all
/// (`Option<T>`, `Option<Option<T>>` or `Delta<T>`). Used by the form checks
/// of cross-field validators, which reject plain types.
pub fn is_optionish_type(ty: &Type) -> bool {
    type_optionality(ty) != FieldOptionality::NotOptional
}

/// Looks up a field by name and classifies its type. Unknown names fall back
/// to `NotOptional`; the form checks report unknown field names separately.
pub fn field_optionality(field_name: &str, all_fields: &[&Field]) -> FieldOptionality {
    all_fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == field_name))
        .map(|f| type_optionality(&f.ty))
        .unwrap_or(FieldOptionality::NotOptional)
}

/// Generates a boolean expression that is `true` when the field has a value
/// (`check_present = true`) or when it is absent (`check_present = false`).
///
/// - `NotOptional`: always present
/// - `Option<T>`: `self.field.is_some()`
/// - `Option<Option<T>>`: `matches!(self.field, Some(Some(_)))`
/// - `Delta<T>`: `self.field.is_set()`
pub fn generate_presence_check(
    field_ident: &Ident,
    optionality: FieldOptionality,
    check_present: bool,
) -> TokenStream2 {
    let present = match optionality {
        FieldOptionality::NotOptional => quote! { true },
        FieldOptionality::Option => quote! { self.#field_ident.is_some() },
        FieldOptionality::OptionOption => quote! { matches!(self.#field_ident, Some(Some(_))) },
        FieldOptionality::Delta => quote! { self.#field_ident.is_set() },
    };

    if check_present {
        present
    } else {
        quote! { !(#present) }
    }
}

/// Generates the "has value" check for the field a `required*` validator sits
/// on. PATCH-aware types get inline checks; everything else goes through the
/// `ValidateRequired` trait.
pub fn generate_has_value_check(
    field_ident: &Ident,
    optionality: FieldOptionality,
) -> TokenStream2 {
    match optionality {
        FieldOptionality::OptionOption => quote! { matches!(self.#field_ident, Some(Some(_))) },
        FieldOptionality::Delta => quote! { self.#field_ident.is_set() },
        _ => quote! { self.#field_ident.validate_required() },
    }
}

/// Generates the "is correctly absent" check for the field a `prohibited*`
/// validator sits on (`true` means the field complies with the prohibition).
/// PATCH-aware types get inline checks; everything else goes through the
/// `ValidateProhibited` trait.
pub fn generate_is_prohibited_check(
    field_ident: &Ident,
    optionality: FieldOptionality,
) -> TokenStream2 {
    match optionality {
        FieldOptionality::OptionOption => quote! { !matches!(self.#field_ident, Some(Some(_))) },
        FieldOptionality::Delta => quote! { !self.#field_ident.is_set() },
        _ => quote! { self.#field_ident.validate_prohibited() },
    }
}

/// Helper function to find a specific attribute (by `name`) within a slice of attributes.
/// This is used, for example, to get the span of a `#[validate(length(...))]` attribute
/// for error reporting.
pub fn get_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    // Iterate through the attributes of a field or struct.
    attrs.iter().find(|a| match &a.meta {
        syn::Meta::List(list) => list.tokens.clone().into_iter().any(|t| match t {
            proc_macro2::TokenTree::Ident(i) => i == name,
            _ => false,
        }),
        _ => false,
    })
}
