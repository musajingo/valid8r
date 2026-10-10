# valid8r_derive

The procedural macro behind `#[derive(Validate)]`. It reads
`#[validate(...)]` attributes and generates calls to the validation traits
and error types in `valid8r`.

Use it through `valid8r`, which re-exports the macro alongside the runtime
traits. You do not need to add `valid8r_derive` as a separate dependency.
For validator behavior and error formats, see the
[runtime guide](https://docs.rs/valid8r).

## Contents

- [Quick Start](#quick-start)
- [Generated Traits](#generated-traits)
- [Struct Attributes](#struct-attributes)
- [Field Attributes](#field-attributes)
- [Custom Functions](#custom-functions)
- [Validation Context](#validation-context)
- [Schema Functions](#schema-functions)
- [Nested Validation](#nested-validation)
- [Optional and PATCH Fields](#optional-and-patch-fields)
- [Error Codes and Sensitive Values](#error-codes-and-sensitive-values)
- [Crate Aliases](#crate-aliases)
- [Features and Dependencies](#features-and-dependencies)
- [Supported Inputs](#supported-inputs)
- [License](#license)

## Quick Start

Requires Rust 1.94 or later. Add this to your `Cargo.toml`:

```toml
[dependencies]
valid8r = { version = "0.2", features = ["email"] }
```

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct Contact {
    #[validate(required, email)]
    email: Option<String>,
}

let missing = Contact { email: None };
assert_eq!(
    missing.validate().unwrap_err().field_errors()["email"][0].code,
    "required"
);

let invalid = Contact {
    email: Some("invalid".into()),
};
assert_eq!(
    invalid.validate().unwrap_err().field_errors()["email"][0].code,
    "email"
);

let valid = Contact {
    email: Some("person@example.com".into()),
};
assert!(valid.validate().is_ok());
```

Rules can share one attribute or use separate attributes on the same field.
The macro generates validation code at compile time; that code runs only
when you call `.validate()` or `.validate_with_args(...)` at runtime.

## Generated Traits

The generated implementations depend on the struct's context configuration:

| Struct configuration | Generated implementations | How to call |
| --- | --- | --- |
| No `context` | `Validate` and `ValidateArgs<'v_a, Args = ()>` | `.validate()` or `.validate_with_args(())` |
| `context = Context` | `ValidateArgs<'v_a, Args = &'v_a Context>` | `.validate_with_args(&ctx)` |
| `context = Context, mutable` | `ValidateArgs<'v_a, Args = &'v_a mut Context>` | `.validate_with_args(&mut ctx)` |

Without a context, `.validate()` delegates to `.validate_with_args(())`.
With a context, the macro does not generate an argument-free `Validate`
implementation. Import `ValidateArgs` to call `.validate_with_args(...)`.

```rust,ignore
use valid8r::{Validate, ValidateArgs};

#[derive(Validate)]
struct Score {
    #[validate(range(min = 0, max = 100))]
    value: i32,
}

let score = Score { value: 101 };
assert_eq!(score.validate(), score.validate_with_args(()));
```

## Struct Attributes

Place these attributes after `#[derive(Validate)]`, before the struct:

| Attribute | Meaning |
| --- | --- |
| `#[validate(context = Context)]` | Pass `&Context` to functions that request `use_context` |
| `#[validate(context = Context, mutable)]` | Pass `&mut Context` instead |
| `#[validate(schema(function = check))]` | Call a function with the whole struct |
| `#[validate(nest_all_fields)]` | Apply nested validation to every field except fields marked `skip` |
| `#[validate(crate = "checks")]` | Generate paths through an alias instead of the default `::valid8r` |

`schema` can be repeated. Its options are `function`, `use_context`,
`skip_on_field_errors`, `code`, and `message`. The default for
`skip_on_field_errors` is `true`.

## Field Attributes

Place field rules inside `#[validate(...)]`. Value, presence, prohibition,
consent, comparison, custom, and schema rules accept `code` and `message`.
The modifiers `nested`, `skip`, and `sensitive` do not accept those options.

### Value Rules

| Rule | Rule-specific options | Behavior |
| --- | --- | --- |
| `email` | `nullable` | Email local-part and length checks; the `email` feature additionally checks domain syntax |
| `url` | `nullable` | Absolute HTTP/HTTPS URLs; needs the `url` feature |
| `phone_number` | `country`, `nullable` | Phone parsing and validity, with an optional ISO 3166-1 alpha-2 country hint; needs `phone_number` |
| `credit_card` | None | Credit-card checks through `card-validate`; needs `cards` |
| `ip` | `v4`, `v6` | IP addresses; use `ip(v4)` or `ip(v6)` to select a version |
| `regex` | `path` | Match a pattern, such as `regex(path = *PATTERN)` |
| `non_control_character` | None | Reject control characters |
| `length` | `min`, `max`, `equal` | Inclusive size bounds or exact size; `equal` cannot be combined with `min`/`max` |
| `range` | `min`, `max`, `exclusive_min`, `exclusive_max` | Inclusive or exclusive numeric bounds |
| `contains` | `pattern` | Substring or supported map-key containment |
| `does_not_contain` | `pattern` | Reject a substring or supported map key |
| `must_match` | `other` | Compare with another field, such as `must_match(other = "password")` |
| `custom` | `function`, `use_context` | Call a custom function; this rule can be repeated |

`length` counts Unicode scalar values for strings and elements for
collections. `nullable` accepts empty strings; it does not modify them.
`must_match` runs only when its annotated field has a value; the other field
must then have an equal value.

### Presence and Prohibition Rules

| Rule | Condition |
| --- | --- |
| `required` | Always require a value |
| `required_if(func = predicate)` | Require a value when `predicate(&self)` is true |
| `required_with(other_fields("a", "b"))` | Require a value when any referenced field is present |
| `required_with_all(other_fields("a", "b"))` | Require a value when all referenced fields are present |
| `required_without(other_fields("a", "b"))` | Require a value when any referenced field is absent |
| `required_without_all(other_fields("a", "b"))` | Require a value when all referenced fields are absent |
| `prohibited_if(func = predicate)` | Require absence when `predicate(&self)` is true |
| `prohibited_with(other_fields("a", "b"))` | Require absence when any referenced field is present |
| `prohibited_with_all(other_fields("a", "b"))` | Require absence when all referenced fields are present |
| `prohibited_without(other_fields("a", "b"))` | Require absence when any referenced field is absent |
| `prohibited_without_all(other_fields("a", "b"))` | Require absence when all referenced fields are absent |

The `with`/`without` families require optional or PATCH fields on both sides.
Referenced names must identify fields on the same struct. A predicate takes
`&Struct` and returns `bool`. Presence means an actual value, not a nonempty
string: `Some("")` counts as present.

### Consent Rules

| Rule | Condition |
| --- | --- |
| `accepted` | Require `true`, or a string equal to `"true"`, `"1"`, `"on"`, or `"yes"` ignoring ASCII case |
| `declined` | Require `false`, or any string not accepted by `accepted`, including an empty string |
| `accepted_if(function = predicate)` | Apply `accepted` when `predicate(&self)` is true |
| `declined_if(function = predicate)` | Apply `declined` when `predicate(&self)` is true |

Consent rules are value rules: an absent `Option<bool>` skips them. Add a
`required*` rule if absence should fail. Notice the option name difference:
presence/prohibition predicates use `func`, while consent predicates use
`function`.

### Field Modifiers

| Modifier | Meaning |
| --- | --- |
| `nested` | Call `.validate()` on a child or collection of children and merge its errors |
| `skip` | Omit this field's validation, including implicit `nest_all_fields` validation |
| `sensitive` | Remove the field's `value` error parameter and suppress its value as `other` in `must_match` errors |

## Custom Functions

A custom function returns `Ok(())` or a `ValidationError`. Repeat the
`custom` attribute to apply multiple functions:

```rust,ignore
use valid8r::{Validate, ValidationError};

fn not_reserved(name: &str) -> Result<(), ValidationError> {
    if ["admin", "root"].contains(&name) {
        Err(ValidationError::new("reserved_name"))
    } else {
        Ok(())
    }
}

fn no_spaces(name: &str) -> Result<(), ValidationError> {
    if name.contains(' ') {
        Err(ValidationError::new("contains_space"))
    } else {
        Ok(())
    }
}

#[derive(Validate)]
struct User {
    #[validate(length(min = 3))]
    #[validate(custom(function = not_reserved))]
    #[validate(custom(function = no_spaces))]
    name: String,
}

assert!(User { name: "ada".into() }.validate().is_ok());
let errors = User { name: "a b".into() }.validate().unwrap_err();
assert_eq!(errors.field_errors()["name"][0].code, "contains_space");
assert!(
    User {
        name: "admin".into()
    }
    .validate()
    .is_err()
);
```

String and collection functions can borrow their arguments (`&str`, `&[T]`).
Numeric primitives written as `i32`, `f64`, etc. are passed by value. This
choice depends on the written type; aliases and qualified spellings can
instead require a reference. Optional fields pass their inner
value when present. Values recorded in error parameters need Serde
serialization support, including values passed to custom field validators.

## Validation Context

### Shared Context

Declare the context on the struct, and add `use_context` to each custom or
schema function that should receive it:

```rust,ignore
use valid8r::{Validate, ValidateArgs, ValidationError};

struct Limits {
    max_items: usize,
}

fn check_items(items: &[String], ctx: &Limits) -> Result<(), ValidationError> {
    if items.len() > ctx.max_items {
        Err(ValidationError::new("too_many_items"))
    } else {
        Ok(())
    }
}

#[derive(Validate)]
#[validate(context = Limits)]
struct Order {
    #[validate(custom(function = check_items, use_context))]
    items: Vec<String>,
}

let order = Order {
    items: vec!["first".into(), "second".into()],
};
let ctx = Limits { max_items: 1 };
assert_eq!(
    order.validate_with_args(&ctx).unwrap_err().field_errors()["items"][0].code,
    "too_many_items"
);
```

Field functions receive `(value, &Context)`; schema functions receive
`(&Struct, &Context)`. Functions without `use_context` keep their usual
single argument.

### Mutable Context

Add `mutable` on the struct to give functions mutable access to the context:

```rust,ignore
use valid8r::{Validate, ValidateArgs, ValidationError};

struct Checks {
    calls: usize,
}

fn record_check(_: &str, ctx: &mut Checks) -> Result<(), ValidationError> {
    ctx.calls += 1;
    Ok(())
}

#[derive(Validate)]
#[validate(context = Checks, mutable)]
struct Form {
    #[validate(custom(function = record_check, use_context))]
    name: String,
}

let form = Form { name: "Ada".into() };
let mut ctx = Checks { calls: 0 };
assert!(form.validate_with_args(&mut ctx).is_ok());
assert_eq!(ctx.calls, 1);
```

This mutates the context, not the struct being validated.

### Borrowed Context Data

When the context type contains references, its attribute path must use the
lifetime `'v_a`, the lifetime reserved by the generated `ValidateArgs` impl:

```rust,ignore
use valid8r::{Validate, ValidateArgs, ValidationError};

struct Policy<'a> {
    allowed_name: &'a str,
}

fn check_name(name: &str, ctx: &Policy<'_>) -> Result<(), ValidationError> {
    if name == ctx.allowed_name {
        Ok(())
    } else {
        Err(ValidationError::new("name_not_allowed"))
    }
}

#[derive(Validate)]
#[validate(context = "Policy<'v_a>")]
struct Request {
    #[validate(custom(function = check_name, use_context))]
    name: String,
}

let expected = String::from("Ada");
let ctx = Policy {
    allowed_name: &expected,
};
assert!(
    Request { name: "Ada".into() }
        .validate_with_args(&ctx)
        .is_ok()
);
assert!(
    Request { name: "Bob".into() }
        .validate_with_args(&ctx)
        .is_err()
);
```

## Schema Functions

A schema function checks the whole struct and returns
`Result<(), ValidationError>`. Repeat the attribute to run multiple schemas:

```rust,ignore
use valid8r::{Validate, ValidationError};

fn ordered(window: &Window) -> Result<(), ValidationError> {
    if window.start > window.end {
        Err(ValidationError::new("invalid_order"))
    } else {
        Ok(())
    }
}

fn starts_early(window: &Window) -> Result<(), ValidationError> {
    if window.start > 5 {
        Err(ValidationError::new("starts_too_late"))
    } else {
        Ok(())
    }
}

#[derive(Validate)]
#[validate(schema(function = ordered))]
#[validate(schema(function = starts_early))]
struct Window {
    #[validate(range(min = 0))]
    start: i32,
    end: i32,
}

let invalid = Window { start: 10, end: 2 };
let errors = invalid.validate().unwrap_err();
let codes: Vec<_> = errors.field_errors()["__all__"]
    .iter()
    .map(|error| error.code.as_ref())
    .collect();
assert_eq!(codes, ["invalid_order", "starts_too_late"]);

// Field errors skip both schemas by default.
let field_error = Window { start: -1, end: 2 };
let errors = field_error.validate().unwrap_err();
assert!(errors.field_errors().contains_key("start"));
assert!(!errors.errors().contains_key("__all__"));
```

Schema errors are collected under `__all__`. By default, field errors,
including nested errors, skip schemas. This decision is made before schemas
run; one schema failure does not skip its siblings when fields were valid.

Set `skip_on_field_errors = false` for a schema that must run despite field
errors:

```rust,ignore
use valid8r::{Validate, ValidationError};

fn check_form(_: &Form) -> Result<(), ValidationError> {
    Err(ValidationError::new("schema_error"))
}

#[derive(Validate)]
#[validate(schema(function = check_form, skip_on_field_errors = false))]
struct Form {
    #[validate(length(min = 1))]
    name: String,
}

let errors = Form {
    name: String::new(),
}
.validate()
.unwrap_err();
assert!(errors.field_errors().contains_key("name"));
assert_eq!(errors.field_errors()["__all__"][0].code, "schema_error");
```

A schema with `use_context` receives `(&Struct, &Context)`, or
`(&Struct, &mut Context)` when `mutable` is set.

## Nested Validation

`nest_all_fields` is useful when all included fields are validating children.
Use `skip` for metadata that should not be validated:

```rust,ignore
use valid8r::{Validate, ValidationErrorsKind};

#[derive(Validate)]
struct Address {
    #[validate(length(min = 1))]
    street: String,
}

#[derive(Validate)]
#[validate(nest_all_fields)]
struct User {
    home: Address,
    work: Option<Address>,
    #[validate(skip)]
    internal_id: String,
}

let user = User {
    home: Address {
        street: String::new(),
    },
    work: None,
    internal_id: String::new(),
};
let errors = user.validate().unwrap_err();
let ValidationErrorsKind::Struct(home) = &errors.errors()["home"] else {
    panic!("expected child errors");
};
assert_eq!(home.field_errors()["street"][0].code, "length");
assert!(!errors.errors().contains_key("work"));
assert!(!errors.errors().contains_key("internal_id"));
```

For a struct with a mix of child and scalar fields, put `#[validate(nested)]`
on individual children instead. The generated code calls `.validate()` on
them; it does not automatically forward a parent's validation context.
Collections such as `Vec<T>` retain child errors by element index.

## Optional and PATCH Fields

Value rules skip absent values. Presence and prohibition rules still check
absence:

| Field type | Present value | Absent or cleared states |
| --- | --- | --- |
| `Option<T>` | `Some(value)` | `None` |
| `Option<Option<T>>` | `Some(Some(value))` | `None`, `Some(None)` |
| `Delta<T>` | `Delta::Set(value)` | `Delta::Unchanged`, `Delta::Clear` |

`required` rejects every absent or cleared state above. Cross-field rules
use the same definition of presence, including when ordinary and PATCH
fields are mixed:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct UpdateAddress {
    #[validate(required_with(other_fields("area_id")))]
    country_id: Option<Option<i32>>,
    area_id: Option<Option<i32>>,
}

assert!(
    UpdateAddress {
        country_id: None,
        area_id: Some(None)
    }
    .validate()
    .is_ok()
);
assert!(
    UpdateAddress {
        country_id: None,
        area_id: Some(Some(123))
    }
    .validate()
    .is_err()
);
assert!(
    UpdateAddress {
        country_id: Some(Some(1)),
        area_id: Some(Some(123))
    }
    .validate()
    .is_ok()
);
```

The derive recognizes these wrappers from their type syntax; it does not
resolve arbitrary Rust type aliases. Serde's default nested-`Option`
handling does not preserve the difference between omitted and explicit
null fields. See the runtime guide for a custom deserializer.

### Delta

Use the Git dependency below, together with `serde`'s `derive` feature and
`serde_json = "1"` for this JSON example:

```toml
[dependencies]
delta = { git = "https://github.com/musajingo/delta", tag = "v0.1.0" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

The dependency's default features include Serde support. `#[serde(default)]`
turns omitted fields into `Delta::Unchanged`; explicit null becomes
`Delta::Clear`:

```rust,ignore
use delta::Delta;
use serde::Deserialize;
use valid8r::Validate;

#[derive(Deserialize, Validate)]
struct UpdateUser {
    #[serde(default)]
    #[validate(required_with(other_fields("area_id")))]
    country_id: Delta<i32>,
    #[serde(default)]
    area_id: Delta<i32>,
}

let omitted: UpdateUser = serde_json::from_str("{}").unwrap();
assert!(matches!(omitted.area_id, Delta::Unchanged));
assert!(omitted.validate().is_ok());

let cleared: UpdateUser = serde_json::from_str(r#"{"area_id": null}"#).unwrap();
assert!(matches!(cleared.area_id, Delta::Clear));
assert!(cleared.validate().is_ok());

let missing_country: UpdateUser = serde_json::from_str(r#"{"area_id": 123}"#).unwrap();
assert!(missing_country.validate().is_err());
```

## Error Codes and Sensitive Values

Use `code` for a stable machine-readable identifier and `message` for
explanatory text. No message is generated automatically. `sensitive`
removes the annotated field's `value` parameter, and `must_match` omits a
sensitive comparison target's `other` parameter:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct PasswordChange {
    #[validate(
        length(
            min = 8,
            code = "PASSWORD_TOO_SHORT",
            message = "Use at least eight characters"
        ),
        sensitive
    )]
    password: String,
    #[validate(must_match(other = "password"), sensitive)]
    confirmation: String,
}

let change = PasswordChange {
    password: "short".into(),
    confirmation: "different".into(),
};
let errors = change.validate().unwrap_err();
let password_error = &errors.field_errors()["password"][0];
assert_eq!(password_error.code, "PASSWORD_TOO_SHORT");
assert_eq!(
    password_error.message.as_deref(),
    Some("Use at least eight characters")
);
assert_eq!(password_error.params["min"], 8);
assert!(!password_error.params.contains_key("value"));
let confirmation_error = &errors.field_errors()["confirmation"][0];
assert!(!confirmation_error.params.contains_key("value"));
assert!(!confirmation_error.params.contains_key("other"));
```

Other custom parameters and messages are not sanitized. Mark sensitive
fields inside child structs individually; marking a parent does not
recursively redact child errors.

## Crate Aliases

Generated paths default to `::valid8r`. If you rename the dependency in
`Cargo.toml`, set the struct's `crate` attribute to that name:

```toml
[dependencies]
checks = { package = "valid8r", version = "0.2" }
```

With this dependency, import `checks::Validate` directly. A Rust import alias
also works; this executable example uses one with the same name:

```rust,ignore
use checks::Validate;
use valid8r as checks;

#[derive(Validate)]
#[validate(crate = "checks")]
struct Contact {
    #[validate(email)]
    email: String,
}

assert!(
    Contact {
        email: "person@example.com".into()
    }
    .validate()
    .is_ok()
);
assert!(
    Contact {
        email: "invalid".into()
    }
    .validate()
    .is_err()
);
```

The macro does not automatically discover a renamed dependency.

## Features and Dependencies

Enable features on `valid8r`, not on this macro crate:

| Runtime feature | Effect |
| --- | --- |
| `email` | Email domain checks, including internationalized domain handling |
| `url` | HTTP/HTTPS URL validation |
| `phone_number` | Phone validation |
| `cards` | Credit-card validation |
| `indexmap` | `length` support for `IndexMap` and `IndexSet` |
| `full` | All optional runtime features |

The macro parses rules regardless of which runtime features you enabled.
A rule requiring a disabled feature fails to compile because its runtime
trait is unavailable. Basic email validation remains available without the
`email` feature, but does not validate domain syntax.

Add direct dependencies when an example uses them: `regex = "1"` for regular
expressions, `serde = { version = "1", features = ["derive"] }` for
serialization derives, `serde_json = "1"` for JSON, and `rust_decimal = "1"`
when your field type is `Decimal`.

## Supported Inputs

The derive accepts structs with named fields, including generic structs.
Tuple structs and enums are rejected. Fields without validation attributes
are left alone unless `nest_all_fields` is set.

`length` requires at least one bound and rejects `equal` combined with
`min` or `max`. `range` requires a bound and rejects a corresponding
inclusive/exclusive pair, such as `min` with `exclusive_min`. `must_match`
and cross-field presence/prohibition rules require existing field names.

Custom functions, predicates, and schema functions use Rust paths.
`#[validate]` on its own is not a nested-validation rule; use
`#[validate(nested)]` explicitly.

The README examples need the runtime crate and, for Delta, a Git dependency,
so they are ignored by this procedural-macro crate's standalone doctests.
The workspace's private integration-test crate compiles and executes these
same examples with `--include-ignored` during `./scripts/check.sh`.

## License

Licensed under either [MIT](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
The MIT license includes the original upstream copyright notice.
