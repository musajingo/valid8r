#![doc = include_str!("../README.md")]

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
                // Snapshot whether any field-level validation failed (including
                // nested struct/list errors) BEFORE any schema runs, so
                // `skip_on_field_errors` decisions are based on field errors
                // only — one schema's failure never suppresses another.
                let __validator_has_field_errors = !errors.is_empty();
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
