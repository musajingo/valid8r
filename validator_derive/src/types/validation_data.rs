use crate::{CrateName, validators};

use darling::{FromDeriveInput, ast::Data, util::WithOriginal};
use proc_macro_error2::abort;
use syn::{Field, Path, PathArguments};

// The main struct we get from parsing the attributes
// The "supports(struct_named)" attribute guarantees only named structs to work with this macro
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(validate), supports(struct_named))]
#[darling(and_then = "ValidationData::validate")]
pub struct ValidationData {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: Data<(), WithOriginal<super::ValidateField, syn::Field>>,

    #[darling(multiple)]
    pub schema: Vec<validators::schema::Schema>,

    pub context: Option<Path>,
    pub mutable: Option<bool>,
    pub nest_all_fields: Option<bool>,

    /// The name of the crate to use for the generated code,
    /// defaults to `validator`.
    #[darling(rename = "crate", default)]
    pub crate_name: CrateName,
}

impl ValidationData {
    /// Performs additional validation checks on the parsed `ValidationData`.
    ///
    /// This method is called by `darling` via the `and_then` attribute on the `ValidationData` struct.
    /// It ensures that the parsed attributes are valid before code generation proceeds.
    ///
    /// Currently, it checks:
    /// 1. If a `context` is provided, its lifetime argument must be `'v_a`.
    /// 2. Calls `validate` on each parsed `ValidateField` for field-specific checks.
    fn validate(self) -> darling::Result<Self> {
        if let Some(context) = &self.context {
            // Check if context lifetime is not `'v_a`
            for segment in &context.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Lifetime(lt) = arg
                            && lt.ident != "v_a"
                        {
                            abort! {
                                lt.ident, "Invalid argument reference";
                                note = "The lifetime `'{}` is not supported.", lt.ident;
                                help = "Please use the validator lifetime `'v_a`";
                            }
                        }
                    }
                }
            }
        }

        // If the derive is on a struct (which is guaranteed by `supports(struct_named)`),
        // iterate through its fields.
        if let Data::Struct(fields) = &self.data {
            // Collect references to the original `syn::Field`s for field-specific validation checks.
            let original_fields: Vec<&Field> = fields.fields.iter().map(|f| &f.original).collect();

            // For each parsed field (`ValidateField`), call its `validate` method.
            for f in &fields.fields {
                f.parsed
                    .validate(&self.ident, &original_fields, &f.original);
            }
        }

        Ok(self)
    }
}
