//! # Validator Derive - Procedural Macros for Validation
//!
//! This crate provides the `#[derive(Validate)]` procedural macro that generates
//! validation implementations for Rust structs. It is part of the validation
//! framework and works in conjunction with the `validator` crate.
//!
//! ## Overview
//!
//! The derive macro parses `#[validate(...)]` attributes on struct fields and
//! generates the `impl Validate for T` block that performs runtime validation.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Your Struct Definition                       │
//! │  #[derive(Validate)]                                           │
//! │  struct User {                                                  │
//! │      #[validate(email)]                                        │
//! │      email: String,                                            │
//! │  }                                                              │
//! └─────────────────────────────────────────────────────────────────┘
//!                                │
//!                                ▼ (compile time)
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    validator_derive (This Crate)                │
//! │                                                                 │
//! │  1. Parse #[validate(...)] attributes using darling            │
//! │  2. Generate validation code for each field                     │
//! │  3. Output impl Validate for User { ... }                      │
//! └─────────────────────────────────────────────────────────────────┘
//!                                │
//!                                ▼ (runtime)
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  user.validate()  →  Runs generated validation code            │
//! │                   →  Returns Ok(()) or Err(ValidationErrors)   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct CreateUser {
//!     #[validate(email)]
//!     email: String,
//!
//!     #[validate(length(min = 8))]
//!     password: String,
//! }
//!
//! let user = CreateUser {
//!     email: "test@example.com".to_string(),
//!     password: "securepass".to_string(),
//! };
//!
//! assert!(user.validate().is_ok());
//! ```
//!
//! ## Struct-Level Attributes
//!
//! These attributes are placed on the struct itself:
//!
//! ### `context`
//!
//! Pass a validation context to custom validators:
//!
//! ```rust,ignore
//! use validator::{Validate, ValidateArgs, ValidationError};
//!
//! struct MyContext {
//!     max_length: usize,
//! }
//!
//! fn validate_with_ctx(val: &str, ctx: &MyContext) -> Result<(), ValidationError> {
//!     if val.len() > ctx.max_length {
//!         return Err(ValidationError::new("too_long"));
//!     }
//!     Ok(())
//! }
//!
//! #[derive(Validate)]
//! #[validate(context = MyContext)]
//! struct Form {
//!     #[validate(custom(function = validate_with_ctx, use_context))]
//!     value: String,
//! }
//!
//! let form = Form { value: "test".to_string() };
//! let ctx = MyContext { max_length: 10 };
//! form.validate_with_args(&ctx);
//! ```
//!
//! ### `mutable`
//!
//! Allow mutable access to the context:
//!
//! ```rust,ignore
//! #[derive(Validate)]
//! #[validate(context = MyContext, mutable)]
//! struct Form { /* ... */ }
//! ```
//!
//! ### `nest_all_fields`
//!
//! Automatically validate all nested structs:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct Address {
//!     #[validate(length(min = 1))]
//!     street: String,
//! }
//!
//! #[derive(Validate)]
//! #[validate(nest_all_fields)]
//! struct User {
//!     home: Address,     // Automatically validated
//!     work: Address,     // Automatically validated
//! }
//! ```
//!
//! ### `schema`
//!
//! Add struct-level validation functions:
//!
//! ```rust,ignore
//! use validator::{Validate, ValidationError};
//!
//! fn validate_range(form: &DateRange) -> Result<(), ValidationError> {
//!     if form.start > form.end {
//!         return Err(ValidationError::new("invalid_range"));
//!     }
//!     Ok(())
//! }
//!
//! #[derive(Validate)]
//! #[validate(schema(function = validate_range))]
//! struct DateRange {
//!     start: u32,
//!     end: u32,
//! }
//! ```
//!
//! Schema options:
//! - `function = path::to::fn` - The validation function
//! - `skip_on_field_errors = false` - Run even if field validation failed (default: true)
//!
//! ---
//!
//! ## Field-Level Attributes
//!
//! These attributes are placed on struct fields using `#[validate(...)]`:
//!
//! ### Format Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `email` | Valid email address | `message`, `code` |
//! | `url` | Valid URL | `message`, `code` |
//! | `phone_number` | Valid phone number | `country`, `message`, `code` |
//! | `credit_card` | Valid credit card (Luhn) | `message`, `code` |
//! | `ip` | Valid IP address | `format` ("v4"/"v6"), `message`, `code` |
//! | `non_control_character` | No control characters | `message`, `code` |
//! | `regex` | Matches regex pattern | `path`, `message`, `code` |
//!
//! ### Numeric Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `range` | Numeric range | `min`, `max`, `exclusive_min`, `exclusive_max`, `message`, `code` |
//! | `length` | String/collection length | `min`, `max`, `equal`, `message`, `code` |
//!
//! ### Presence Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `required` | Field must be Some | `message`, `code` |
//! | `required_if` | Required if condition | `func`, `message`, `code` |
//! | `required_with` | Required if any field present | `other_fields(...)`, `message`, `code` |
//! | `required_with_all` | Required if all fields present | `other_fields(...)`, `message`, `code` |
//! | `required_without` | Required if any field absent | `other_fields(...)`, `message`, `code` |
//! | `required_without_all` | Required if all fields absent | `other_fields(...)`, `message`, `code` |
//!
//! ### Prohibition Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `prohibited_if` | Must be None if condition | `func`, `message`, `code` |
//! | `prohibited_with` | Must be None if any field present | `other_fields(...)`, `message`, `code` |
//! | `prohibited_with_all` | Must be None if all fields present | `other_fields(...)`, `message`, `code` |
//! | `prohibited_without` | Must be None if any field absent | `other_fields(...)`, `message`, `code` |
//! | `prohibited_without_all` | Must be None if all fields absent | `other_fields(...)`, `message`, `code` |
//!
//! ### Container Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `contains` | Contains value | `pattern`, `message`, `code` |
//! | `does_not_contain` | Doesn't contain value | `pattern`, `message`, `code` |
//!
//! ### Comparison Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `must_match` | Equals another field | `other`, `message`, `code` |
//!
//! ### Consent Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `accepted` | Boolean true or accepted string | `message`, `code` |
//! | `declined` | Boolean false or declined string | `message`, `code` |
//! | `accepted_if` | Accepted if condition | `function`, `message`, `code` |
//! | `declined_if` | Declined if condition | `function`, `message`, `code` |
//!
//! ### Other Validators
//!
//! | Attribute | Description | Options |
//! |-----------|-------------|---------|
//! | `custom` | Custom validation function | `function`, `use_context`, `message`, `code` |
//! | `nested` | Validate nested struct | - |
//! | `skip` | Skip this field | - |
//!
//! ---
//!
//! ## Multiple Validators
//!
//! Apply multiple validators to a single field:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct User {
//!     // All validators must pass
//!     #[validate(length(min = 3, max = 20))]
//!     #[validate(regex(path = *USERNAME_RE))]
//!     username: String,
//! }
//!
//! # use std::sync::LazyLock;
//! # use regex::Regex;
//! # static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z]+$").unwrap());
//! ```
//!
//! Or combine in a single attribute:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct User {
//!     #[validate(email, required)]
//!     email: Option<String>,
//! }
//! ```
//!
//! ---
//!
//! ## Option, `Option<Option<T>>`, and `PatchData<T>` Handling
//!
//! The derive macro automatically handles optional fields:
//!
//! ### `Option<T>`
//!
//! - `None` → Validation skipped (unless `required`)
//! - `Some(value)` → Validate the inner value
//!
//! ### `Option<Option<T>>` (PATCH semantics)
//!
//! For partial update operations, the macro auto-detects `Option<Option<T>>`:
//!
//! - `None` → Field omitted, validation skipped
//! - `Some(None)` → Clearing field, validation skipped
//! - `Some(Some(value))` → Validate the inner value
//!
//! This affects cross-field validators like `required_with`:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct UpdateAddress {
//!     // country_id required only when area_id has a value (Some(Some(_)))
//!     // NOT required when area_id is being cleared (Some(None))
//!     #[validate(required_with(other_fields("area_id")))]
//!     country_id: Option<Option<i32>>,
//!
//!     area_id: Option<Option<i32>>,
//! }
//!
//! // Clearing area_id - country_id NOT required
//! let update = UpdateAddress {
//!     country_id: None,
//!     area_id: Some(None),  // Clearing
//! };
//! assert!(update.validate().is_ok());
//!
//! // Setting area_id - country_id IS required
//! let update = UpdateAddress {
//!     country_id: None,
//!     area_id: Some(Some(123)),  // Has value
//! };
//! assert!(update.validate().is_err());
//! ```
//!
//! ### `PatchData<T>` (Recommended for PATCH)
//!
//! `PatchData<T>` from `common::utils` is a wrapper that provides the same semantics
//! as `Option<Option<T>>` with less boilerplate. The derive macro automatically
//! recognizes `PatchData<T>` and treats it as equivalent to `Option<Option<T>>`.
//!
//! ```rust,ignore
//! use help_ttp::patch_request::PatchData;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[derive(Deserialize, Validate)]
//! struct UpdateVendor {
//!     // Only needs #[serde(default)] - no custom deserializer required
//!     #[serde(default)]
//!     #[validate(length(min = 1, max = 100))]
//!     name: PatchData<String>,
//!
//!     #[serde(default)]
//!     #[validate(email)]
//!     email: PatchData<String>,
//!
//!     #[serde(default)]
//!     #[validate(url(nullable))]
//!     website: PatchData<String>,
//!
//!     // Cross-field validation works with PatchData
//!     #[serde(default)]
//!     #[validate(required_with(other_fields("area_id")))]
//!     country_id: PatchData<i32>,
//!
//!     #[serde(default)]
//!     area_id: PatchData<i32>,
//! }
//! ```
//!
//! **Why use `PatchData<T>` over `Option<Option<T>>`?**
//!
//! | `Option<Option<T>>` | `PatchData<T>` |
//! |---------------------|----------------|
//! | Requires `#[serde(default, deserialize_with = "...")]` | Only `#[serde(default)]` |
//! | No helper methods | `is_absent()`, `is_null()`, `has_value()` |
//! | Direct pattern matching | Via `Deref` (`*field`) or helper methods |
//!
//! **Semantics (identical for both types):**
//!
//! | State | `Option<Option<T>>` | `PatchData<T>` | Validators |
//! |-------|---------------------|----------------|------------|
//! | Absent | `None` | `PatchData(None)` | Skipped |
//! | Null | `Some(None)` | `PatchData(Some(None))` | Skipped |
//! | Value | `Some(Some(v))` | `PatchData(Some(Some(v)))` | Run |
//!
//! ---
//!
//! ## Custom Validators
//!
//! ### Basic Custom Validator
//!
//! ```rust,ignore
//! use validator::{Validate, ValidationError};
//!
//! fn validate_username(name: &str) -> Result<(), ValidationError> {
//!     if name.starts_with("admin") {
//!         return Err(ValidationError::new("reserved_prefix")
//!             .with_message("Username cannot start with 'admin'".into()));
//!     }
//!     Ok(())
//! }
//!
//! #[derive(Validate)]
//! struct User {
//!     #[validate(custom(function = validate_username))]
//!     username: String,
//! }
//! ```
//!
//! ### Custom Validator with Context
//!
//! ```rust,ignore
//! use validator::{Validate, ValidateArgs, ValidationError};
//!
//! struct Config {
//!     forbidden_words: Vec<String>,
//! }
//!
//! fn validate_no_forbidden(text: &str, ctx: &Config) -> Result<(), ValidationError> {
//!     for word in &ctx.forbidden_words {
//!         if text.contains(word) {
//!             return Err(ValidationError::new("forbidden_word"));
//!         }
//!     }
//!     Ok(())
//! }
//!
//! #[derive(Validate)]
//! #[validate(context = Config)]
//! struct Post {
//!     #[validate(custom(function = validate_no_forbidden, use_context))]
//!     content: String,
//! }
//! ```
//!
//! ---
//!
//! ## Generated Code
//!
//! For a struct like:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct User {
//!     #[validate(email)]
//!     email: String,
//!
//!     #[validate(length(min = 8))]
//!     password: String,
//! }
//! ```
//!
//! The macro generates approximately:
//!
//! ```rust,ignore
//! impl validator::Validate for User {
//!     fn validate(&self) -> Result<(), validator::ValidationErrors> {
//!         use validator::ValidateEmail;
//!         use validator::ValidateLength;
//!
//!         let mut errors = validator::ValidationErrors::new();
//!
//!         // Email validation
//!         if !self.email.validate_email() {
//!             let mut err = validator::ValidationError::new("email");
//!             err.add_param(std::borrow::Cow::from("value"), &self.email);
//!             errors.add("email", err);
//!         }
//!
//!         // Length validation
//!         if !self.password.validate_length(Some(8), None, None) {
//!             let mut err = validator::ValidationError::new("length");
//!             err.add_param(std::borrow::Cow::from("value"), &self.password);
//!             err.add_param(std::borrow::Cow::from("min"), &8);
//!             errors.add("password", err);
//!         }
//!
//!         if errors.is_empty() {
//!             Ok(())
//!         } else {
//!             Err(errors)
//!         }
//!     }
//! }
//! ```
//!
//! ---
//!
//! ## Error Messages and Codes
//!
//! All validators accept `message` and `code` parameters:
//!
//! ```rust,ignore
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct Form {
//!     #[validate(email(
//!         code = "INVALID_EMAIL",
//!         message = "Please enter a valid email address"
//!     ))]
//!     email: String,
//!
//!     #[validate(length(
//!         min = 8,
//!         max = 100,
//!         code = "PASSWORD_LENGTH",
//!         message = "Password must be 8-100 characters"
//!     ))]
//!     password: String,
//! }
//! ```
//!
//! ---
//!
//! ## Feature Flags
//!
//! The derive macro respects the feature flags from the `validator` crate:
//!
//! - `email` - Email validation with IDN support
//! - `url` - URL validation
//! - `phone_number` - Phone number validation
//! - `cards` - Credit card validation
//!
//! ---
//!
//! ## Implementation Details
//!
//! This crate uses:
//!
//! - [`darling`](https://docs.rs/darling) for attribute parsing
//! - [`syn`](https://docs.rs/syn) for Rust syntax parsing
//! - [`quote`](https://docs.rs/quote) for code generation
//! - [`proc-macro2`](https://docs.rs/proc-macro2) for token manipulation
//!
//! The macro generates both `impl Validate` and `impl ValidateArgs` to support
//! validation with and without context.

mod types;
mod utils;
mod validators;

use darling::FromDeriveInput;
use proc_macro_error2::proc_macro_error;
use quote::quote;
use syn::{DeriveInput, GenericParam, parse_macro_input};
use types::{ValidateField, ValidationData};
use utils::{CrateName, quote_use_statements};

/// The main procedural macro function for `#[derive(Validate)]`.
///
/// This function is invoked by the Rust compiler when `#[derive(Validate)]`
/// is encountered. It takes the `TokenStream` of the struct definition as input
/// and returns a `TokenStream` containing the generated `impl Validate for ...` block.
///
/// # Generated Traits
///
/// - `impl Validate for T` - Simple validation without context
/// - `impl ValidateArgs<'_> for T` - Validation with optional context
///
/// # Steps
///
/// 1. Parses the input `TokenStream` into a `syn::DeriveInput`
/// 2. Uses `ValidationData::from_derive_input` (powered by `darling`) to parse
///    attributes into the `ValidationData` struct
/// 3. Extracts necessary information like struct identifier, generics, and parsed field data
/// 4. Generates `use` statements for validator traits
/// 5. Generates code for struct-level schema validations
/// 6. Iterates over fields, generating field-specific validation logic
/// 7. Assembles the final `impl ValidateArgs for ...` and optionally `impl Validate for ...` blocks
#[proc_macro_error]
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    // Parse the input `DeriveInput` into the `ValidationData` struct using `darling`.
    // This extracts all `#[validate(...)]` attributes from the struct and its fields.
    let validation_data = match ValidationData::from_derive_input(&input) {
        Ok(data) => data,
        Err(e) => return e.write_errors().into(),
    };

    let crate_name = validation_data.crate_name;

    // Determine the type of the custom validation context, if any.
    // This is specified by `#[validate(context = "MyContext")]` on the struct.
    let custom_context = if let Some(context) = &validation_data.context {
        if let Some(mutable) = validation_data.mutable {
            if mutable {
                quote!(&'v_a mut #context)
            } else {
                quote!(&'v_a #context)
            }
        } else {
            quote!(&'v_a #context)
        }
    } else {
        quote!(())
    };

    // Extract parsed field validation data and original fields from `ValidationData`.
    let struct_data = validation_data.data.take_struct().unwrap();

    // Collect references to the original `syn::Field`s for cross-field validators
    // like `required_with` and `prohibited_with` which need to detect Option nesting levels.
    let original_fields: Vec<syn::Field> = struct_data
        .fields
        .iter()
        .map(|f| f.original.clone())
        .collect();
    let all_fields: Vec<&syn::Field> = original_fields.iter().collect();

    let mut validation_fields: Vec<ValidateField> = struct_data
        .fields
        .into_iter()
        .map(|f| f.parsed)
        // Skip fields explicitly marked with `#[validate(skip)]`.
        .filter(|f| if let Some(s) = f.skip { !s } else { true })
        // Populate the `crate_name` for each field, used in token generation.
        .map(|f| ValidateField {
            crate_name: crate_name.clone(),
            ..f
        })
        .collect();

    // If `#[validate(nest_all_fields)]` is present on the struct,
    // mark all fields for nested validation.
    if let Some(nest_all_fields) = validation_data.nest_all_fields
        && nest_all_fields
    {
        validation_fields = validation_fields
            .iter_mut()
            .map(|f| {
                f.nested = Some(true);
                f.to_owned()
            })
            .collect();
    }

    let use_statements = quote_use_statements(&crate_name, &validation_fields);

    // Generate code for struct-level schema validations, specified by `#[validate(schema(...))]`.
    let schema = validation_data.schema.iter().fold(quote!(), |acc, s| {
        let st = validators::schema::tokens(s.clone());
        let acc = quote! {
            #acc
            #st
        };
        acc
    });

    let ident = validation_data.ident;
    let (imp, ty, whr) = validation_data.generics.split_for_impl();

    // Prepare generics for the `impl ValidateArgs` block.
    // We need to remove default types from generic parameters as they are not allowed in trait impls.
    let struct_generics_quote =
        validation_data
            .generics
            .params
            .iter()
            .fold(quote!(), |mut q, g| {
                if let GenericParam::Type(t) = g {
                    // Default types (e.g., `T = String`) are not allowed in trait impl generic parameter lists.
                    if t.default.is_some() {
                        let mut t2 = t.clone();
                        t2.default = None;
                        let g2 = GenericParam::Type(t2);
                        q.extend(quote!(#g2, ));
                    } else {
                        q.extend(quote!(#g, ));
                    }
                } else {
                    q.extend(quote!(#g, ));
                }
                q
            });

    // Construct the generic arguments for the `impl ValidateArgs` line, including the `'v_a` lifetime.
    let imp_args = if struct_generics_quote.is_empty() {
        quote!(<'v_a>)
    } else {
        quote!(<'v_a, #struct_generics_quote>)
    };

    // If no custom validation context (`args`) is specified on the struct,
    // also generate a simpler `impl Validate for ...` that calls `validate_with_args(())`.
    let argless_validation = if validation_data.context.is_none() {
        quote! {
            impl #imp #crate_name::Validate for #ident #ty #whr {
                fn validate(&self) -> ::std::result::Result<(), #crate_name::ValidationErrors> {
                    use #crate_name::ValidateArgs;
                    self.validate_with_args(())
                }
            }
        }
    } else {
        quote!()
    };

    // Generate validation tokens for each field, passing all_fields for cross-field validators
    let field_validation_tokens: Vec<proc_macro2::TokenStream> = validation_fields
        .iter()
        .map(|f| f.generate_tokens(&all_fields))
        .collect();

    // The main generated code block.
    quote!(
        #argless_validation

        impl #imp_args #crate_name::ValidateArgs<'v_a> for #ident #ty #whr {
            // Define the associated type `Args` for the validation context.
            // This will be `()` if no context is specified, or `&'v_a MyContext` / `&'v_a mut MyContext` otherwise.
            type Args = #custom_context;

            fn validate_with_args(&self, args: Self::Args)
            -> ::std::result::Result<(), #crate_name::ValidationErrors>
             {
                // Include the generated `use` statements.
                #use_statements

                // Initialize an empty `ValidationErrors` collection.
                let mut errors = #crate_name::ValidationErrors::new();

                // Insert the token streams for each field's validation logic.
                // Each field's validation tokens check its specific rules and add to `errors` if they fail.
                #(#field_validation_tokens)*
                // Insert the token stream for struct-level schema validations.
                #schema

                if errors.is_empty() {
                    ::std::result::Result::Ok(())
                } else {
                    ::std::result::Result::Err(errors)
                }
            }
        }
    )
    .into()
}
