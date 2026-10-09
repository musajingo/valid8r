//! This module handles the parsing and token generation for struct-level
//! `#[validate(schema(...))]` attributes.
//!
//! Schema validation allows defining custom validation functions that operate
//! on the entire struct instance, enabling cross-field validation logic.

use quote::quote;

use crate::utils::quote_message;

/// Represents the arguments parsed from a `#[validate(schema(...))]` attribute
/// applied at the struct level.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
#[derive(Debug, Clone, darling::FromMeta)]
pub struct Schema {
    /// Optional custom error code for this schema validation.
    pub code: Option<String>,

    /// Optional custom error message for this schema validation.
    pub message: Option<String>,

    /// The path to the custom schema validation function.
    /// This function is expected to take a reference to the struct instance
    /// (and optionally a `ValidationContext`) and return a `Result<(), valid8r::ValidationError>`.
    pub function: syn::Path,

    /// Whether the custom schema function requires the validation context.
    /// If `true`, the function will be called with `(&struct_instance, context)`.
    /// If `false` or `None`, it will be called with `(&struct_instance)`.
    pub use_context: Option<bool>,

    /// If `true` (the default), this schema validation will be skipped if any field-level
    /// validations have already failed. If `false`, it will run regardless of field errors.
    pub skip_on_field_errors: Option<bool>,
}

/// Generates the `proc_macro2::TokenStream` for a single struct-level schema validation.
///
/// This function takes the parsed `Schema` arguments and constructs the Rust code
/// that will call the user-defined schema validation function at runtime.
pub fn tokens(schema: Schema) -> proc_macro2::TokenStream {
    let fn_call = schema.function;
    // Determine the arguments to pass to the schema validation function
    // based on the `use_context` flag.
    let args = if let Some(args) = schema.use_context {
        if args {
            quote!(&self, args)
        } else {
            quote!(&self)
        }
    } else {
        quote!(&self)
    };

    // Determine whether to skip this schema validation if field errors already exist.
    // Defaults to true if not specified.
    let skip_on_errors = schema.skip_on_field_errors.unwrap_or(true);

    let message = quote_message(schema.message);

    // Prepare the custom error code, if provided.
    let code = if let Some(c) = schema.code {
        quote!(
            err.code = ::std::borrow::Cow::from(#c);
        )
    } else {
        quote!()
    };

    // Generate the token stream for calling the schema function and handling its result.
    // Errors from schema validation are typically added with the "__all__" key.
    let fn_call = quote! {
        match #fn_call(#args) {
            ::std::result::Result::Ok(()) => {}
            ::std::result::Result::Err(mut err) => {
                #code
                #message
                // Schema-level errors are added to the "__all__" field key.
                errors.add("__all__", err);
            }
        }
    };

    // If `skip_on_errors` is true, wrap the function call in a condition on the
    // field-error snapshot taken before any schema ran. The snapshot includes
    // nested struct/list errors (which `errors.field_errors()` would hide) and
    // excludes errors added by earlier schemas, so sibling schemas still run
    // when all fields were valid.
    if skip_on_errors {
        quote! {
            if !__validator_has_field_errors {
                #fn_call
            }
        }
    } else {
        quote! {
            #fn_call
        }
    }
}
