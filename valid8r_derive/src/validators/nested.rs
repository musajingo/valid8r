//! This module handles the token generation for the `#[validate(nested)]` attribute.
//!
//! The `nested` validator allows validating fields that are themselves structs
//! implementing the `Validate` trait, or collections of such structs.

use quote::quote;

/// Generates the `proc_macro2::TokenStream` for nested validation logic.
///
/// This function takes the field's identifier and its name as a string
/// to construct the Rust code that will call the `validate()` method
/// on the nested field or its elements at runtime.
///
/// # Arguments
/// * `field_name` - Token stream representing the field being validated (e.g., `self.my_nested_field`).
/// * `field_name_str` - The string name of the field, used as a key for error reporting.
///
/// # Returns
/// A `proc_macro2::TokenStream` containing the Rust code for the nested validation.
pub fn tokens(
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    quote! {
        // Call `validate()` on the nested field/item and merge any resulting errors.
        // The `errors.merge_self` method handles the logic for different kinds of nested errors (struct, list).
        errors.merge_self(#field_name_str, (&#field_name).validate());
    }
}
