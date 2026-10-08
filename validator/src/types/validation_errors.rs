use crate::ValidationError;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};

/// Represents the different kinds of validation errors that can occur for a field.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ValidationErrorsKind {
    /// A list of specific `ValidationError`s for a simple field.
    Field(Vec<ValidationError>),

    /// Validation errors for a nested struct.
    Struct(Box<ValidationErrors>),

    /// Validation errors for a list/vector of items, indexed by their position.
    List(BTreeMap<usize, Box<ValidationErrors>>),
}

/// A collection of validation errors, typically mapping field names to their respective errors.
/// It's newtype wrapper around a `HashMap`.
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ValidationErrors(pub HashMap<Cow<'static, str>, ValidationErrorsKind>);

impl ValidationErrors {
    /// Creates a new, empty `ValidationErrors` collection.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Returns a boolean indicating whether a validation result includes validation errors for a
    /// given field. May be used as a condition for performing nested struct validations on a field
    /// in the absence of field-level validation errors.
    #[must_use]
    pub fn has_error(result: &Result<(), ValidationErrors>, field: &'static str) -> bool {
        match result {
            Ok(()) => false,
            Err(errs) => errs.has_key(field),
        }
    }

    /// Merges validation errors from a `child` validation result into `self` for a given `field`.
    /// Modifies `self` in place and returns a mutable reference to it.
    pub fn merge_self(
        &mut self,
        field: &'static str,
        child: Result<(), ValidationErrors>,
    ) -> &mut ValidationErrors {
        match child {
            Ok(()) => self, // If the child validation is Ok, no changes are needed.
            Err(mut errors) => {
                // This is a bit of a hack to be able to support collections which return a
                // `ValidationErrors` with a made-up `_tmp_validator` entry which we need to strip
                // off.
                if let Some(collection) = errors.0.remove("_tmp_validator") {
                    // If a special "_tmp_validator" key exists, its value is treated as nested errors.
                    self.add_nested(field, collection);
                } else {
                    // Otherwise, the entire `errors` object is treated as errors for a nested struct.
                    self.add_nested(field, ValidationErrorsKind::Struct(Box::new(errors)));
                }
                self // Returns the modified ValidationErrors.
            }
        }
    }

    /// Returns the combined outcome of a struct's validation result along with the nested
    /// validation result for one of its fields.
    /// If `child` has errors, they are added to `parent`'s errors under the specified `field`.
    pub fn merge(
        parent: Result<(), ValidationErrors>,
        field: &'static str,
        child: Result<(), ValidationErrors>,
    ) -> Result<(), ValidationErrors> {
        match child {
            Ok(()) => parent, // If child validation is Ok, return the parent's result.
            Err(errors) => {
                // If child has errors, ensure parent is an Err, then add child's errors.
                parent
                    .and_then(|_| Err(ValidationErrors::new()))
                    .map_err(|mut parent_errors| {
                        parent_errors
                            .add_nested(field, ValidationErrorsKind::Struct(Box::new(errors)));
                        parent_errors
                    })
            }
        }
    }

    /// Returns the combined outcome of a struct's validation result along with the nested
    /// validation result for one of its fields where that field is a vector of validating structs.
    /// Collects errors from `children` and merges them into `parent`'s errors under `field` as a `List`.
    pub fn merge_all(
        parent: Result<(), ValidationErrors>,
        field: &'static str,
        children: Vec<Result<(), ValidationErrors>>,
    ) -> Result<(), ValidationErrors> {
        // Process each child result:
        let errors = children
            .into_iter()
            .enumerate() // Get index along with the result.
            .filter_map(|(i, res)| res.err().map(|mut err| (i, err.remove(field)))) // Keep only errors, remove specific field from child's errors.
            .filter_map(|(i, entry)| match entry {
                // Ensure the removed entry was a Struct kind.
                Some(ValidationErrorsKind::Struct(errors)) => Some((i, errors)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>(); // Collect into a BTreeMap (index -> errors).

        if errors.is_empty() {
            parent // If no child errors, return the parent's result.
        } else {
            // If there are child errors, ensure parent is an Err, then add child errors as a List.
            parent
                .and_then(|_| Err(ValidationErrors::new()))
                .map_err(|mut parent_errors| {
                    parent_errors.add_nested(field, ValidationErrorsKind::List(errors));
                    parent_errors
                })
        }
    }

    /// Returns a map of field-level validation errors found for the struct that was validated and
    /// any of it's nested structs that are tagged for validation.
    /// Provides an immutable reference to the internal error map.
    pub fn errors(&self) -> &HashMap<Cow<'static, str>, ValidationErrorsKind> {
        &self.0
    }

    /// Returns a mutable map of field-level validation errors found for the struct that was validated and
    /// any of it's nested structs that are tagged for validation.
    /// Provides a mutable reference to the internal error map.
    pub fn errors_mut(&mut self) -> &mut HashMap<Cow<'static, str>, ValidationErrorsKind> {
        &mut self.0
    }

    /// Consume the struct, returning the validation errors found.
    /// Takes ownership of `self` and returns the internal error map.
    pub fn into_errors(self) -> HashMap<Cow<'static, str>, ValidationErrorsKind> {
        self.0
    }

    /// Returns a map of only field-level validation errors found for the struct that was validated.
    /// Filters the main error map to include only `ValidationErrorsKind::Field` entries.
    pub fn field_errors(&self) -> HashMap<Cow<'static, str>, &Vec<ValidationError>> {
        self.0
            .iter()
            .filter_map(|(k, v)| {
                // Iterate over errors.
                if let ValidationErrorsKind::Field(errors) = v {
                    // Check if the error kind is Field.
                    Some((k.clone(), errors)) // If so, include it in the result.
                } else {
                    None // Otherwise, filter it out.
                }
            })
            .collect::<HashMap<_, _>>() // Collect into a new HashMap.
    }

    /// Adds a specific `ValidationError` to a given `field`, appending to any
    /// existing field-level errors. If the field currently holds *structural*
    /// (nested struct/list) errors, those are kept and the scalar error is
    /// dropped — the two kinds cannot share a key in this model, and structural
    /// errors are the richer signal. Never panics.
    pub fn add(&mut self, field: &'static str, error: ValidationError) {
        self.add_kind(
            Cow::Borrowed(field),
            ValidationErrorsKind::Field(vec![error]),
        );
    }

    /// Strips the `value` param from every field-level error recorded under
    /// `field`, so the submitted value of a credential never leaves the process.
    pub fn redact_value(&mut self, field: &str) {
        if let Some(ValidationErrorsKind::Field(errors)) = self.0.get_mut(field) {
            for error in errors {
                error.params.remove("value");
            }
        }
    }

    /// Checks if the `ValidationErrors` collection is empty (i.e., no errors).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds a `ValidationErrorsKind` (nested struct/list errors) for a given
    /// `field`, merging with any existing entry rather than panicking. This is
    /// what makes a DTO field that carries *both* a field-level validator (e.g.
    /// `length`) and `nested` safe: the derive emits the scalar error first and
    /// the nested errors second, so this merge point must tolerate an occupied
    /// key. See [`Self::add_kind`] for the merge policy.
    fn add_nested(&mut self, field: &'static str, errors: ValidationErrorsKind) {
        self.add_kind(Cow::Borrowed(field), errors);
    }

    /// Insert `incoming` under `field`, merging into any existing entry.
    ///
    /// Merge policy: `Field`+`Field` concatenates, `List`+`List` merges by index
    /// (recursing per element), `Struct`+`Struct` merges recursively. A scalar
    /// `Field` colliding with structural errors yields the structural errors
    /// (richer); two mismatched structural kinds keep the existing one. Never
    /// panics.
    fn add_kind(&mut self, field: Cow<'static, str>, incoming: ValidationErrorsKind) {
        let merged = match self.0.remove(&field) {
            None => incoming,
            Some(existing) => merge_kinds(existing, incoming),
        };
        self.0.insert(field, merged);
    }

    /// Fold every entry of `other` into `self`, merging per key.
    fn absorb(&mut self, other: ValidationErrors) {
        for (key, kind) in other.0 {
            self.add_kind(key, kind);
        }
    }

    /// Checks if the `ValidationErrors` collection contains any errors for the specified `field`.
    #[must_use]
    fn has_key(&self, field: &'static str) -> bool {
        self.0.contains_key(field)
    }

    /// Removes and returns the `ValidationErrorsKind` for a given `field`, if it exists.
    fn remove(&mut self, field: &'static str) -> Option<ValidationErrorsKind> {
        self.0.remove(field)
    }
}

/// Combine two error kinds recorded under the same field key. See
/// [`ValidationErrors::add_kind`] for the policy this implements.
fn merge_kinds(
    existing: ValidationErrorsKind,
    incoming: ValidationErrorsKind,
) -> ValidationErrorsKind {
    use ValidationErrorsKind::{Field, List, Struct};
    match (existing, incoming) {
        (Field(mut a), Field(b)) => {
            a.extend(b);
            Field(a)
        }
        (List(mut a), List(b)) => {
            for (idx, item) in b {
                match a.entry(idx) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(item);
                    }
                    std::collections::btree_map::Entry::Occupied(mut o) => {
                        o.get_mut().absorb(*item);
                    }
                }
            }
            List(a)
        }
        (Struct(mut a), Struct(b)) => {
            a.absorb(*b);
            Struct(a)
        }
        // A scalar field-level error and structural errors cannot share a key;
        // keep whichever side is structural.
        (Field(_), other) => other,
        (existing, Field(_)) => existing,
        // Two different structural kinds (List vs Struct) under one key is not
        // representable; keep the existing.
        (existing, _incoming) => existing,
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (path, err)) in self.errors().iter().enumerate() {
            display_errors(fmt, err, path)?;
            if idx + 1 < self.errors().len() {
                writeln!(fmt)?;
            }
        }
        Ok(())
    }
}

/// Implements the standard `Error` trait for `ValidationErrors`.
impl std::error::Error for ValidationErrors {}

/// Recursively formats and writes validation errors to a `fmt::Formatter`.
///
/// This function is responsible for creating a human-readable string representation
/// of the validation errors, including their paths (e.g., "user.email" or "items[0].name").
///
/// # Arguments
///
/// * `fmt`: A mutable reference to a `fmt::Formatter` where the output will be written.
/// * `errs`: A reference to the `ValidationErrorsKind` to be displayed. This can be
///   a collection of field-specific errors, errors for a nested struct, or errors
///   for a list of items.
/// * `path`: A string slice representing the current path to the error(s) being processed.
///   This is built up recursively as the function traverses nested structures.
fn display_errors(
    fmt: &mut fmt::Formatter<'_>,
    errs: &ValidationErrorsKind,
    path: &str,
) -> fmt::Result {
    /// Inner helper function to display errors for a nested struct.
    /// It iterates over the errors within the struct and recursively calls `display_errors`
    /// for each, prepending the current `path` and the field name.
    fn display_struct(
        fmt: &mut fmt::Formatter<'_>,
        errs: &ValidationErrors, // The collection of errors for the nested struct.
        path: &str,              // The base path to this nested struct.
    ) -> fmt::Result {
        // Create a mutable string to build the full path for nested fields.
        let mut full_path = String::new();
        // Append the current path and a dot separator (e.g., "parent_struct.").
        write!(&mut full_path, "{}.", path)?;
        // Store the length of the base path to easily truncate later.
        let base_len = full_path.len();
        // Iterate over each error entry (field name and its errors) in the nested struct.
        for (key, err) in errs.errors() {
            // Append the current field's name to the full_path (e.g., "parent_struct.child_field").
            write!(&mut full_path, "{}", key)?;
            // Recursively call display_errors for the current field's errors.
            display_errors(fmt, err, &full_path)?;
            // Reset full_path to the base path for the next field in the struct.
            full_path.truncate(base_len);
        }
        Ok(())
    }

    // Match on the kind of validation errors.
    match errs {
        // Case 1: Errors for a simple field.
        ValidationErrorsKind::Field(field_errors) => {
            // Write the path to the field, followed by a colon and space (e.g., "fieldname: ").
            write!(fmt, "{}: ", path)?;
            let len = field_errors.len();
            // Iterate over each ValidationError for this field.
            for (idx, err) in field_errors.iter().enumerate() {
                // If it's the last error in the list, write it directly.
                if idx + 1 == len {
                    write!(fmt, "{}", err)?;
                } else {
                    // Otherwise, write the error followed by a comma and space for separation.
                    write!(fmt, "{}, ", err)?;
                }
            }
            Ok(())
        }
        // Case 2: Errors for a nested struct.
        ValidationErrorsKind::Struct(struct_errors) => {
            // Delegate to the `display_struct` helper function.
            display_struct(fmt, struct_errors, path)
        }
        // Case 3: Errors for a list/vector of items.
        ValidationErrorsKind::List(list_errors) => {
            // Create a mutable string to build the full path for list items.
            let mut full_path = String::new();
            // Write the base path of the list (e.g., "items").
            write!(&mut full_path, "{}", path)?;
            // Store the length of the base path.
            let base_len = full_path.len();
            // Iterate over each entry (index and its errors) in the list.
            for (idx, err) in list_errors.iter() {
                // Append the current item's index to the full_path (e.g., "items[0]").
                write!(&mut full_path, "[{}]", idx)?;
                // The errors for a list item are themselves `ValidationErrors` (like a struct),
                // so call `display_struct` to format them.
                display_struct(fmt, err, &full_path)?;
                // Reset full_path to the base path for the next item in the list.
                full_path.truncate(base_len);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidationError;
    use std::borrow::Cow;

    fn create_simple_error(code: &'static str) -> ValidationError {
        ValidationError::new(code)
    }

    #[test]
    fn test_new_and_is_empty() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty(), "A new ValidationErrors should be empty");
        errors.add("field1", create_simple_error("required"));
        assert!(
            !errors.is_empty(),
            "ValidationErrors should not be empty after adding an error"
        );
    }

    #[test]
    fn test_add_and_field_errors() {
        let mut errors = ValidationErrors::new();
        let err1 = create_simple_error("required");
        let err2 = create_simple_error("min_length");

        errors.add("name", err1.clone());
        let field_errs = errors.field_errors();

        let field_name = Cow::from("name");

        assert_eq!(field_errs.len(), 1, "Should have errors for one field");
        assert_eq!(
            field_errs.get(&field_name).unwrap().len(),
            1,
            "Field 'name' should have one error"
        );
        assert_eq!(
            field_errs.get(&field_name).unwrap()[0],
            err1,
            "The error for 'name' should be the one added"
        );

        errors.add("name", err2.clone());
        let field_errs_updated = errors.field_errors();
        assert_eq!(
            field_errs_updated.get(&field_name).unwrap().len(),
            2,
            "Field 'name' should now have two errors"
        );
        assert_eq!(
            field_errs_updated.get(&field_name).unwrap()[1],
            err2,
            "The second error for 'name' should be the one added"
        );

        errors.add("age", create_simple_error("numeric"));
        assert_eq!(
            errors.field_errors().len(),
            2,
            "Should have errors for two distinct fields now"
        );
    }

    #[test]
    fn test_add_field_over_struct_keeps_struct() {
        let mut errors = ValidationErrors::new();
        let mut inner = ValidationErrors::new();
        inner.add("email", create_simple_error("invalid"));
        errors.add_nested(
            "profile",
            ValidationErrorsKind::Struct(Box::new(inner.clone())),
        );
        // Adding a scalar error to a key that holds structural errors keeps the
        // structural errors rather than panicking.
        errors.add("profile", create_simple_error("unexpected"));
        match errors.errors().get(&Cow::from("profile")) {
            Some(ValidationErrorsKind::Struct(s)) => assert_eq!(**s, inner),
            other => panic!("expected Struct to be preserved, got {other:?}"),
        }
    }

    #[test]
    fn test_field_and_nested_merge_no_panic() {
        // The failure mode that produced a request-time panic: a field carrying
        // both a scalar validator and `nested`, both failing.
        let mut errors = ValidationErrors::new();
        errors.add("items", create_simple_error("length"));
        let mut item0 = ValidationErrors::new();
        item0.add("quantity", create_simple_error("range"));
        let mut list_map = BTreeMap::new();
        list_map.insert(0, Box::new(item0));
        errors.add_nested("items", ValidationErrorsKind::List(list_map));
        // Structural (per-item) errors win; no panic.
        assert!(matches!(
            errors.errors().get(&Cow::from("items")),
            Some(ValidationErrorsKind::List(_))
        ));
    }

    #[test]
    fn test_add_nested_struct_and_list() {
        let mut errors = ValidationErrors::new();
        let mut inner_struct_errors = ValidationErrors::new();
        inner_struct_errors.add("email", create_simple_error("invalid_format"));

        errors.add_nested(
            "user",
            ValidationErrorsKind::Struct(Box::new(inner_struct_errors.clone())),
        );

        let mut list_item_errors = ValidationErrors::new();
        list_item_errors.add("name", create_simple_error("required"));
        let mut list_map = BTreeMap::new();
        list_map.insert(0, Box::new(list_item_errors.clone()));
        errors.add_nested("items", ValidationErrorsKind::List(list_map.clone()));

        match errors.errors().get(&Cow::from("user")) {
            Some(ValidationErrorsKind::Struct(s_errs)) => assert_eq!(
                **s_errs, inner_struct_errors,
                "Nested struct errors for 'user' do not match"
            ),
            _ => panic!("Expected Struct errors for 'user'"),
        }

        match errors.errors().get(&Cow::from("items")) {
            Some(ValidationErrorsKind::List(l_errs)) => assert_eq!(
                *l_errs, list_map,
                "Nested list errors for 'items' do not match"
            ),
            _ => panic!("Expected List errors for 'items'"),
        }
    }

    #[test]
    fn test_add_nested_merges_on_existing_key() {
        // Two List entries under one key merge by index instead of panicking.
        let mut errors = ValidationErrors::new();
        let mut a_item = ValidationErrors::new();
        a_item.add("name", create_simple_error("required"));
        let mut a = BTreeMap::new();
        a.insert(0, Box::new(a_item));
        errors.add_nested("items", ValidationErrorsKind::List(a));

        let mut b_item = ValidationErrors::new();
        b_item.add("value", create_simple_error("range"));
        let mut b = BTreeMap::new();
        b.insert(1, Box::new(b_item));
        errors.add_nested("items", ValidationErrorsKind::List(b));

        match errors.errors().get(&Cow::from("items")) {
            Some(ValidationErrorsKind::List(map)) => assert_eq!(map.len(), 2),
            other => panic!("expected merged List, got {other:?}"),
        }
    }

    #[test]
    fn test_has_error() {
        let mut errors = ValidationErrors::new();
        errors.add("name", create_simple_error("required"));
        let err_result: Result<(), ValidationErrors> = Err(errors);
        let ok_result: Result<(), ValidationErrors> = Ok(());

        assert!(
            ValidationErrors::has_error(&err_result, "name"),
            "has_error should return true for 'name' when errors exist"
        );
        assert!(
            !ValidationErrors::has_error(&err_result, "age"),
            "has_error should return false for 'age' as no error was added for it"
        );
        assert!(
            !ValidationErrors::has_error(&ok_result, "name"),
            "has_error should return false when result is Ok"
        );
    }

    #[test]
    fn test_merge_self() {
        let mut parent_errors = ValidationErrors::new();
        parent_errors.add("parent_field", create_simple_error("p_err"));

        // Case 1: Child is Ok
        parent_errors.merge_self("child_struct", Ok(()));
        assert!(
            parent_errors
                .errors()
                .get(&Cow::from("child_struct"))
                .is_none(),
            "No errors should be added for 'child_struct' if child result is Ok"
        );

        // Case 2: Child is Err (normal struct)
        let mut child_errs1 = ValidationErrors::new();
        child_errs1.add("child_field1", create_simple_error("c_err1"));
        parent_errors.merge_self("child_struct1", Err(child_errs1.clone()));
        match parent_errors.errors().get(&Cow::from("child_struct1")) {
            Some(ValidationErrorsKind::Struct(s_errs)) => assert_eq!(
                **s_errs, child_errs1,
                "Merged child errors for 'child_struct1' do not match"
            ),
            _ => panic!("Expected Struct errors for 'child_struct1'"),
        }

        // Case 3: Child is Err with _tmp_validator
        let mut child_errs2_container = ValidationErrors::new();
        let mut actual_child_errs2 = ValidationErrors::new();
        actual_child_errs2.add("item_name", create_simple_error("item_err"));
        child_errs2_container.add_nested(
            "_tmp_validator",
            ValidationErrorsKind::Struct(Box::new(actual_child_errs2.clone())),
        );

        parent_errors.merge_self("child_struct2", Err(child_errs2_container));
        match parent_errors.errors().get(&Cow::from("child_struct2")) {
            Some(ValidationErrorsKind::Struct(s_errs)) => assert_eq!(
                **s_errs, actual_child_errs2,
                "Merged child errors for 'child_struct2' after _tmp_validator processing do not match"
            ),
            _ => {
                panic!("Expected Struct errors for 'child_struct2' after _tmp_validator processing")
            }
        }
    }

    #[test]
    fn test_merge() {
        let parent_ok: Result<(), ValidationErrors> = Ok(());
        let mut parent_err = ValidationErrors::new();
        parent_err.add("p_field", create_simple_error("p_err"));
        let parent_err_res: Result<(), ValidationErrors> = Err(parent_err.clone());

        let mut child_err = ValidationErrors::new();
        child_err.add("c_field", create_simple_error("c_err"));
        let child_err_res: Result<(), ValidationErrors> = Err(child_err.clone());
        let child_ok_res: Result<(), ValidationErrors> = Ok(());

        // Parent Ok, Child Ok
        assert!(
            ValidationErrors::merge(parent_ok.clone(), "child", child_ok_res.clone()).is_ok(),
            "Merge (Parent Ok, Child Ok) should result in Ok"
        );

        // Parent Ok, Child Err
        let res1 = ValidationErrors::merge(parent_ok.clone(), "child", child_err_res.clone());
        assert!(
            res1.is_err(),
            "Merge (Parent Ok, Child Err) should result in Err"
        );
        if let Err(e) = res1 {
            match e.errors().get(&Cow::from("child")) {
                Some(ValidationErrorsKind::Struct(s)) => assert_eq!(
                    **s, child_err,
                    "Merged child errors in (Parent Ok, Child Err) case do not match"
                ),
                _ => panic!("Merge (Parent Ok, Child Err) failed"),
            }
        }

        // Parent Err, Child Ok
        let res2 = ValidationErrors::merge(parent_err_res.clone(), "child", child_ok_res.clone());
        assert!(
            res2.is_err(),
            "Merge (Parent Err, Child Ok) should result in Err"
        );
        assert_eq!(
            res2.unwrap_err(),
            parent_err,
            "Merge (Parent Err, Child Ok) should return original parent errors"
        ); // Child Ok, so parent errors unchanged

        // Parent Err, Child Err
        let res3 = ValidationErrors::merge(parent_err_res.clone(), "child", child_err_res.clone());
        assert!(
            res3.is_err(),
            "Merge (Parent Err, Child Err) should result in Err"
        );
        if let Err(e) = res3 {
            assert!(
                e.errors().contains_key(&Cow::from("p_field")),
                "Merged errors in (Parent Err, Child Err) should contain original parent field error"
            );
            match e.errors().get(&Cow::from("child")) {
                Some(ValidationErrorsKind::Struct(s)) => assert_eq!(
                    **s, child_err,
                    "Merged child errors in (Parent Err, Child Err) case do not match"
                ),
                _ => panic!("Merge (Parent Err, Child Err) failed for child part"),
            }
        }
    }

    #[test]
    fn test_merge_all() {
        let parent_ok: Result<(), ValidationErrors> = Ok(());
        let mut child1_errs_container = ValidationErrors::new();
        let mut child1_actual_errs = ValidationErrors::new();
        child1_actual_errs.add("name", create_simple_error("c1_err"));
        child1_errs_container.add_nested(
            "items",
            ValidationErrorsKind::Struct(Box::new(child1_actual_errs.clone())),
        );

        let mut child2_errs_container = ValidationErrors::new();
        let mut child2_actual_errs = ValidationErrors::new();
        child2_actual_errs.add("value", create_simple_error("c2_err"));
        child2_errs_container.add_nested(
            "items",
            ValidationErrorsKind::Struct(Box::new(child2_actual_errs.clone())),
        );

        let children = vec![
            Ok(()),
            Err(child1_errs_container),
            Err(child2_errs_container),
        ];

        let result = ValidationErrors::merge_all(parent_ok.clone(), "items", children);
        assert!(
            result.is_err(),
            "merge_all with child errors should result in Err"
        );
        if let Err(merged_errors) = result {
            match merged_errors.errors().get(&Cow::from("items")) {
                Some(ValidationErrorsKind::List(list_map)) => {
                    assert_eq!(
                        list_map.len(),
                        2,
                        "List map should contain errors from two children"
                    ); // Only two children had errors
                    assert_eq!(
                        *list_map.get(&1).unwrap(),
                        Box::new(child1_actual_errs),
                        "Errors for child at index 1 do not match"
                    );
                    assert_eq!(
                        *list_map.get(&2).unwrap(),
                        Box::new(child2_actual_errs),
                        "Errors for child at index 2 do not match"
                    );
                }
                _ => panic!("merge_all did not produce List errors correctly"),
            }
        }

        // Test with no child errors
        let children_all_ok = vec![Ok(()), Ok(())];
        let result_all_ok = ValidationErrors::merge_all(parent_ok, "items", children_all_ok);
        assert!(
            result_all_ok.is_ok(),
            "merge_all with no child errors should result in Ok"
        );
    }

    #[test]
    fn test_into_errors() {
        let mut errors = ValidationErrors::new();
        errors.add("field", create_simple_error("test"));
        let map = errors.clone().into_errors(); // Clone because into_errors consumes
        assert!(
            map.contains_key(&Cow::from("field")),
            "The resulting map from into_errors should contain the added field"
        );
        assert_eq!(
            map.len(),
            1,
            "The resulting map from into_errors should have one entry"
        );
    }

    #[test]
    fn test_display_validation_errors() {
        let mut errors = ValidationErrors::new();
        errors.add("name", ValidationError::new("required"));
        let mut addr_errors = ValidationErrors::new();
        addr_errors.add("street", ValidationError::new("too_short"));
        errors.add_nested(
            "address",
            ValidationErrorsKind::Struct(Box::new(addr_errors)),
        );

        let mut item0_errors = ValidationErrors::new();
        item0_errors.add("id", ValidationError::new("invalid_chars"));
        let mut list_map = BTreeMap::new();
        list_map.insert(0, Box::new(item0_errors));
        errors.add_nested("items", ValidationErrorsKind::List(list_map));

        let display_str = format!("{}", errors);

        // Order of top-level fields in HashMap is not guaranteed, so check for substrings
        assert!(
            display_str.contains("name: Validation error: required [{}]"),
            "Display output missing 'name' field error"
        );
        assert!(
            display_str.contains("address.street: Validation error: too_short [{}]"),
            "Display output missing nested 'address.street' field error"
        );
        assert!(
            display_str.contains("items[0].id: Validation error: invalid_chars [{}]"),
            "Display output missing list item 'items[0].id' field error"
        );
    }

    #[test]
    fn redact_value_strips_only_the_value_param() {
        let mut errors = ValidationErrors::new();
        let mut error = ValidationError::new("length");
        error.add_param(Cow::from("value"), &"hunter7");
        error.add_param(Cow::from("min"), &8);
        errors.add("password", error);

        errors.redact_value("password");

        let Some(ValidationErrorsKind::Field(field)) = errors.0.get("password") else {
            panic!("password errors missing");
        };
        assert!(!field[0].params.contains_key("value"));
        assert_eq!(field[0].params["min"], 8);
    }
}
