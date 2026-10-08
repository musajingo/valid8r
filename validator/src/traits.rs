use crate::types::{ValidationErrors, ValidationErrorsKind};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::btree_map::BTreeMap;

/// A trait for data structures that can be validated.
///
/// Implementors of this trait provide a `validate` method that checks
/// if the instance conforms to predefined validation rules.
pub trait Validate {
    /// Validates the current instance.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation is successful.
    /// * `Err(ValidationErrors)` if validation fails, containing details about the errors.
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Enables validation for references to types that implement `Validate`.
///
/// This implementation allows calling `validate()` directly on a reference
/// (e.g., `&my_struct.validate()`) without needing to dereference it first.
impl<T: Validate> Validate for &T {
    /// Delegates validation to the underlying type `T`.
    fn validate(&self) -> Result<(), ValidationErrors> {
        // Calls the validate method on the dereferenced type T.
        T::validate(self)
    }
}

/// Macro to implement the `Validate` trait for various list-like collection types.
///
/// This macro generates an implementation of `Validate` for a given container type `$container`.
/// The implementation iterates over the items in the container, calls `validate()` on each item,
/// and collects any `ValidationErrors`. If any item fails validation, the collected errors
/// are returned as `ValidationErrorsKind::List`, wrapped in a `ValidationErrors` struct
/// with a temporary key `_tmp_validator`. This key is intended to be processed by
/// a parent validator's merge logic.
macro_rules! impl_validate_list {
    ($container:ty) => {
        impl<T: Validate> Validate for $container {
            fn validate(&self) -> Result<(), ValidationErrors> {
                // BTreeMap to store errors, mapping the index of the item to its ValidationErrors.
                let mut vec_err: BTreeMap<usize, Box<ValidationErrors>> = BTreeMap::new();

                for (index, item) in self.iter().enumerate() {
                    // Validate each item in the collection.
                    if let Err(e) = item.validate() {
                        vec_err.insert(index, Box::new(e));
                    }
                }

                if vec_err.is_empty() {
                    Ok(())
                } else {
                    // If there are errors, wrap them in ValidationErrorsKind::List.
                    let err_kind = ValidationErrorsKind::List(vec_err);
                    let errors = ValidationErrors(std::collections::HashMap::from([(
                        // Use a temporary key for these list errors, to be handled by merge logic.
                        Cow::Borrowed("_tmp_validator"),
                        err_kind,
                    )]));
                    Err(errors)
                }
            }
        }
    };
}

// Implement `Validate` for `std::collections::HashSet<T>`.
impl_validate_list!(std::collections::HashSet<T>);
// Implement `Validate` for `std::collections::BTreeSet<T>`.
impl_validate_list!(std::collections::BTreeSet<T>);
// Implement `Validate` for `std::collections::BinaryHeap<T>`.
impl_validate_list!(std::collections::BinaryHeap<T>);
// Implement `Validate` for `std::collections::LinkedList<T>`.
impl_validate_list!(std::collections::LinkedList<T>);
// Implement `Validate` for `std::collections::VecDeque<T>`.
impl_validate_list!(std::collections::VecDeque<T>);
// Implement `Validate` for `std::vec::Vec<T>`.
impl_validate_list!(std::vec::Vec<T>);
// Implement `Validate` for slices `[T]`.
impl_validate_list!([T]);

// Implement `Validate` for fixed-size arrays `[T; N]`.
impl<T: Validate, const N: usize> Validate for [T; N] {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut vec_err: BTreeMap<usize, Box<ValidationErrors>> = BTreeMap::new();

        for (index, item) in self.iter().enumerate() {
            if let Err(e) = item.validate() {
                vec_err.insert(index, Box::new(e));
            }
        }

        if vec_err.is_empty() {
            Ok(())
        } else {
            let err_kind = ValidationErrorsKind::List(vec_err);
            let errors = ValidationErrors(std::collections::HashMap::from([(
                Cow::Borrowed("_tmp_validator"),
                err_kind,
            )]));
            Err(errors)
        }
    }
}

/// Implement `Validate` for references to `std::collections::HashMap<K, V, S>`.
///
/// This implementation iterates over the *values* of the HashMap, calls `validate()` on each value,
/// and collects any `ValidationErrors` by their iteration index. Errors are returned as
/// `ValidationErrorsKind::List` with the temporary key `_tmp_validator`.
impl<K, V: Validate, S> Validate for &HashMap<K, V, S> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut vec_err: BTreeMap<usize, Box<ValidationErrors>> = BTreeMap::new();

        // Iterate over values and validate each one.
        for (index, (_key, value)) in self.iter().enumerate() {
            if let Err(e) = value.validate() {
                vec_err.insert(index, Box::new(e));
            }
        }

        if vec_err.is_empty() {
            Ok(())
        } else {
            let err_kind = ValidationErrorsKind::List(vec_err);
            // Wrap errors in ValidationErrorsKind::List with a temporary key.
            let errors =
                ValidationErrors(HashMap::from([(Cow::Borrowed("_tmp_validator"), err_kind)]));
            // Return the collected errors.
            Err(errors)
        }
    }
}

/// Implement `Validate` for references to `std::collections::BTreeMap<K, V>`.
///
/// This implementation iterates over the *values* of the BTreeMap, calls `validate()` on each value,
/// and collects any `ValidationErrors` by their iteration index. Errors are returned as
/// `ValidationErrorsKind::List` with the temporary key `_tmp_validator`.
impl<K, V: Validate> Validate for &BTreeMap<K, V> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut vec_err: BTreeMap<usize, Box<ValidationErrors>> = BTreeMap::new();

        // Iterate over values and validate each one.
        for (index, (_key, value)) in self.iter().enumerate() {
            if let Err(e) = value.validate() {
                vec_err.insert(index, Box::new(e)); // Store errors by iteration index.
            }
        }

        if vec_err.is_empty() {
            Ok(())
        } else {
            let err_kind = ValidationErrorsKind::List(vec_err);
            // Wrap errors in ValidationErrorsKind::List with a temporary key.
            let errors =
                ValidationErrors(HashMap::from([(Cow::Borrowed("_tmp_validator"), err_kind)]));
            // Return the collected errors.
            Err(errors)
        }
    }
}

/// This trait will be implemented by deriving `Validate`. This implementation can take one
/// argument and pass this on to custom validators. The default `Args` type will be `()` if
/// there is no custom validation with defined arguments.
///
/// The `Args` type can use the lifetime `'v_a` to pass references onto the validator.
pub trait ValidateArgs<'v_a> {
    type Args;
    fn validate_with_args(&self, args: Self::Args) -> Result<(), ValidationErrors>;
}

impl<'v_a, T, U> ValidateArgs<'v_a> for Option<T>
where
    T: ValidateArgs<'v_a, Args = U>,
{
    type Args = U;

    fn validate_with_args(&self, args: Self::Args) -> Result<(), ValidationErrors> {
        if let Some(nested) = self {
            T::validate_with_args(nested, args)
        } else {
            Ok(())
        }
    }
}
