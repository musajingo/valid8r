//! This module handles the parsing and token generation for the `#[validate(ip(...))]` attribute.
//!
//! The `ip` validator checks if a string value is a valid IP address,
//! optionally specifying whether it should be IPv4 or IPv6.

use quote::quote;

use crate::utils::{CrateName, quote_code, quote_message};

/// Represents the arguments parsed from a `#[validate(ip(...))]` attribute.
///
/// This struct is used by `darling` to deserialize the attribute's meta items.
/// It allows users to specify whether the IP should be validated as IPv4 (`v4 = true`),
/// IPv6 (`v6 = true`), or either (default, or if both are false or not specified).
/// Custom error `message` and `code` can also be provided.
#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Ip {
    /// Optional custom error code for this validation.
    pub code: Option<String>,

    /// Optional custom error message for this validation.
    pub message: Option<String>,

    /// Whether to validate as IPv4.
    pub v4: Option<bool>,

    /// Whether to validate as IPv6.
    pub v6: Option<bool>,
}
/// Generates the `proc_macro2::TokenStream` for the IP address validation logic.
///
/// This function takes the parsed `Ip` arguments, the crate name identifier,
/// the field's identifier, and the field's name as a string to construct
/// the Rust code that will perform the IP validation at runtime.
/// It determines whether to call `validate_ip()`, `validate_ipv4()`, or `validate_ipv6()`
/// based on the `v4` and `v6` flags in the `Ip` struct.
pub fn tokens(
    crate_name: &CrateName,
    ip: Ip,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    // Prepare the custom message and code, if provided.
    let message = quote_message(ip.message);
    let code = quote_code(crate_name, ip.code, "ip");

    // Determine which validation function to call based on the v4 and v6 flags.
    let version = match (ip.v4, ip.v6) {
        (Some(v4), Some(v6)) => match (v4, v6) {
            (true, false) => quote!(validate_ipv4()),
            (false, true) => quote!(validate_ipv6()),
            _ => quote!(validate_ip()),
        },
        (Some(v4), None) => {
            // If only v4 is specified
            if v4 {
                quote!(validate_ipv4())
            } else {
                quote!(validate_ip())
            }
        }
        (None, Some(v6)) => {
            // If only v6 is specified
            if v6 {
                quote!(validate_ipv6())
            } else {
                quote!(validate_ip())
            }
        }
        _ => quote!(validate_ip()),
    };

    // Generate the token stream for the IP validation.
    quote! {
        if !#field_name.#version {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
