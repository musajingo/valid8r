# valid8r

Validate Rust structs with `#[derive(Validate)]` and receive structured field,
nested, and schema errors. Use it for form input, API payloads, and partial
updates.

This crate builds on [`validator`](https://github.com/Keats/validator) by
Vincent Prouillet and its contributors. It adds conditional presence and
prohibition rules, phone and consent validation, PATCH semantics.

## Contents

- [Quick Start](#quick-start)
- [Feature Flags](#feature-flags)
- [Error Handling](#error-handling)
- [Optional Fields](#optional-fields)
- [Sensitive Values](#sensitive-values)
- [PATCH Semantics](#patch-semantics)
- [Nested Validation](#nested-validation)
- [Custom Validators](#custom-validators)
- [Schema Validation](#schema-validation)
- [Validation Context](#validation-context)
- [Custom Error Messages and Codes](#custom-error-messages-and-codes)
- [Skipping Fields](#skipping-fields)
- [Request Validation](#request-validation)
- [Validator Reference](#validator-reference)
- [License](#license)

## Quick Start

Requires Rust 1.94 or later. Add this to `Cargo.toml`:

```toml
[dependencies]
valid8r = "0.2"
```

The `Validate` import brings both the derive macro and the trait into scope.
Annotate fields with rules, then call `.validate()`:

```rust
use valid8r::Validate;

#[derive(Validate)]
struct SignUp {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8), sensitive)]
    password: String,

    #[validate(accepted)]
    accepted_terms: bool,
}

let mut signup = SignUp {
    email: "person@example.com".into(),
    password: "long-enough-password".into(),
    accepted_terms: true,
};
assert!(signup.validate().is_ok());

signup.email = "invalid".into();
let errors = signup.validate().unwrap_err();
assert_eq!(errors.field_errors()["email"][0].code, "email");
```

Validation returns `Ok(())` when all rules pass and `Err(ValidationErrors)`
when they fail. It checks the values without changing them. You must call
`.validate()` yourself after constructing or deserializing the struct.

## Feature Flags

The derive macro is always available. No optional features are enabled by
default. Basic email, length, range, IP, presence, consent, and custom rules
work with the dependency above.

Enable `email` when you need email domain syntax checks. Without this
feature, `email` checks the local part and lengths but can accept an empty
or malformed domain, such as `"person@"`.

| Feature        | What it adds                                                                        |
| -------------- | ----------------------------------------------------------------------------------- |
| `email`        | Email domain validation through `idna`; basic email validation is already available |
| `url`          | HTTP/HTTPS URL validation                                                           |
| `phone_number` | Phone validation through `phonenumber`                                              |
| `cards`        | Credit-card validation through `card-validate`                                      |
| `indexmap`     | `length` validation for `IndexMap` and `IndexSet`                                   |
| `full`         | All optional features above                                                         |

Enable the features your fields need:

```toml
[dependencies]
valid8r = { version = "0.2", features = ["url", "phone_number"] }
```

For all optional validators, use `features = ["full"]`. A rule such as
`#[validate(url)]` needs its feature enabled in `Cargo.toml`.

## Error Handling

Each `ValidationError` has a machine-readable `code`, an optional `message`,
and a `params` map with details such as the submitted value and rule bounds.
The default code is usually the validator name, for example `"email"` or
`"length"`. Messages are only present when supplied by an attribute or a
custom validator.

Use `.field_errors()` to inspect direct field errors:

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Contact {
    #[validate(email(message = "Enter a valid email address"))]
    email: String,
}

let contact = Contact {
    email: "invalid".into(),
};
let errors = contact.validate().unwrap_err();
let error = &errors.field_errors()["email"][0];
assert_eq!(error.code, "email");
assert_eq!(
    error.message.as_deref(),
    Some("Enter a valid email address")
);
assert_eq!(error.params["value"], "invalid");

// Serialize the whole error tree for an API response.
let json = serde_json::to_value(&errors).unwrap();
assert_eq!(json["email"][0]["code"], "email");
```

Add `serde_json = "1"` to your application's dependencies for serialization.
The JSON representation of the error above is:

```json
{
  "email": [
    {
      "code": "email",
      "message": "Enter a valid email address",
      "params": { "value": "invalid" }
    }
  ]
}
```

`.field_errors()` includes only direct field errors. Use `.errors()` or
serialize the whole `ValidationErrors` to retain nested errors. Entries are
`ValidationErrorsKind::Field` (a list of errors), `Struct` (child fields), or
`List` (child errors indexed by position). Schema errors use the `__all__`
key. See [Nested Validation](#nested-validation) and
[Schema Validation](#schema-validation).

When scalar field errors and nested errors share a key, the nested errors
take precedence and the scalar errors under that key are discarded.

## Optional Fields

Value rules such as `email`, `length`, `range`, and `custom` check the inner
value of `Some(value)` and skip `None`. Add `required` when the value must be
present (or just don't make it Option):

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Profile {
    #[validate(email)]
    backup_email: Option<String>,

    #[validate(required, length(min = 3))]
    display_name: Option<String>,
}

let missing = Profile {
    backup_email: None,
    display_name: None,
};
assert_eq!(
    missing.validate().unwrap_err().field_errors()["display_name"][0].code,
    "required"
);

let present = Profile {
    backup_email: None,
    display_name: Some("Ada".into()),
};
assert!(present.validate().is_ok());
```

`required` checks presence, rather than string length: `Some("")` satisfies
`required`, but fails `length(min = 1)`. Presence and prohibition rules still
check absent fields; they do not behave like value rules.

### Empty Strings

`email(nullable)`, `url(nullable)`, and `phone_number(nullable)` accept an
empty string as well as a valid value. They leave the empty string unchanged;
conversion to
`None` or a database null belongs to your application.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Contact {
    #[validate(email(nullable))]
    email: String,
}

let empty = Contact { email: String::new() };
let invalid = Contact { email: "invalid".into() };
assert!(empty.validate().is_ok());
assert!(invalid.validate().is_err());
```

Enable the corresponding `url` or `phone_number` feature for those rules.
Whitespace is not an empty string, and `nullable` does not bypass other
rules on the field.

## Sensitive Values

Value validators normally include the submitted value in error parameters.
Add `sensitive` to a field to remove its `value` parameter. When `must_match`
compares against a sensitive field, it also omits that field's `other`
parameter:

```rust
use valid8r::Validate;

#[derive(Validate)]
struct PasswordChange {
    #[validate(length(min = 8), sensitive)]
    password: String,

    #[validate(must_match(other = "password"), sensitive)]
    confirmation: String,
}

let change = PasswordChange {
    password: "short".into(),
    confirmation: "different".into(),
};
let errors = change.validate().unwrap_err();
assert!(
    !errors.field_errors()["password"][0]
        .params
        .contains_key("value")
);
assert!(
    !errors.field_errors()["confirmation"][0]
        .params
        .contains_key("value")
);
assert!(
    !errors.field_errors()["confirmation"][0]
        .params
        .contains_key("other")
);
```

Codes, messages, and non-value parameters such as `min` remain available.
Custom validators control their other parameters and messages, so keep
credentials out of those too. Annotate sensitive fields inside nested structs
as well; an annotation on a parent field does not redact its children's
errors.

## PATCH Semantics

Partial updates need three states: leave a field unchanged, clear it, or set
it to a value. `valid8r` recognizes both `Option<Option<T>>` and
[`Delta<T>`](https://github.com/musajingo/field-delta).

| Request state   | `Option<Option<T>>` | `Delta<T>`          | Value rules run? | Counts as present? |
| --------------- | ------------------- | ------------------- | ---------------- | ------------------ |
| Omitted         | `None`              | `Delta::Unchanged`  | No               | No                 |
| Explicit `null` | `Some(None)`        | `Delta::Clear`      | No               | No                 |
| Value           | `Some(Some(value))` | `Delta::Set(value)` | Yes              | Yes                |

Presence rules still run. For example, `required` rejects both omitted and
cleared fields, while `required_with` becomes active only when a referenced
field has an actual value. `valid8r` validates the incoming update; applying
it to stored data is your application's responsibility.

### Using `Option<Option<T>>`

Serde's default handling of nested `Option`s treats both a missing field and
JSON `null` as `None`. Use `default` together with a custom deserializer to
preserve explicit nulls. For the example below, add:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use serde::{Deserialize, Deserializer};
use valid8r::Validate;

fn deserialize_patch_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize, Validate)]
struct UpdateUser {
    #[serde(default, deserialize_with = "deserialize_patch_field")]
    #[validate(required_with(other_fields("area_id")))]
    country_id: Option<Option<i32>>,

    #[serde(default, deserialize_with = "deserialize_patch_field")]
    area_id: Option<Option<i32>>,

    #[serde(default, deserialize_with = "deserialize_patch_field")]
    #[validate(range(min = 0, max = 100))]
    score: Option<Option<i32>>,
}

let omitted: UpdateUser = serde_json::from_str("{}").unwrap();
assert_eq!(omitted.area_id, None);
assert!(omitted.validate().is_ok());

let cleared: UpdateUser = serde_json::from_str(r#"{"area_id": null}"#).unwrap();
assert_eq!(cleared.area_id, Some(None));
assert!(cleared.validate().is_ok());

// Setting area_id requires a value for country_id too.
let missing_country: UpdateUser = serde_json::from_str(r#"{"area_id": 123}"#).unwrap();
assert!(missing_country.validate().is_err());

let valid: UpdateUser =
    serde_json::from_str(r#"{"area_id": 123, "country_id": 1, "score": 50}"#).unwrap();
assert!(valid.validate().is_ok());
```

### Using `Delta<T>`

`Delta<T>` models the same states directly and requires only
`#[serde(default)]` for missing fields. It comes from the
[`field-delta`](https://crates.io/crates/field-delta) crate:

```toml
[dependencies]
field-delta = "0.1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust,ignore
use field_delta::Delta;
use serde::Deserialize;
use valid8r::Validate;

#[derive(Deserialize, Validate)]
struct UpdateUser {
    #[serde(default)]
    #[validate(email)]
    email: Delta<String>,

    #[serde(default)]
    #[validate(required_with(other_fields("area_id")))]
    country_id: Delta<i32>,

    #[serde(default)]
    area_id: Delta<i32>,
}

let cleared: UpdateUser = serde_json::from_str(r#"{"area_id": null}"#).unwrap();
assert!(matches!(cleared.area_id, Delta::Clear));
assert!(cleared.validate().is_ok());

let invalid: UpdateUser = serde_json::from_str(r#"{"area_id": 123}"#).unwrap();
assert!(invalid.validate().is_err());
```

## Nested Validation

Validate nested structs using the `nested` attribute:

```rust
use valid8r::Validate;
#[derive(Validate)]
struct Address {
    #[validate(length(min = 1))]
    street: String,

    #[validate(length(min = 1))]
    city: String,
}

#[derive(Validate)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(nested)]
    address: Address,

    // Optional nested validation
    #[validate(nested)]
    billing_address: Option<Address>,
}
```

Use `nest_all_fields` when every field should receive nested validation. Each
field must support `.validate()` after unwrapping any optional values:

```rust
use valid8r::Validate;
# #[derive(Validate)]
# struct Address { #[validate(length(min = 1))] street: String }
#[derive(Validate)]
#[validate(nest_all_fields)]
struct Company {
    address: Address,      // Automatically nested
    mailing: Address,      // Automatically nested
}
```

Use `#[validate(nested)]` on `Vec<T>` to validate its elements. Child errors
remain under the parent field; collection errors are indexed by element
position. Optional nested values are skipped when absent. For example:

```rust
use valid8r::{Validate, ValidationErrorsKind};

#[derive(Validate)]
struct Address {
    #[validate(length(min = 1))]
    street: String,
}

#[derive(Validate)]
struct Order {
    #[validate(nested)]
    addresses: Vec<Address>,
}

let order = Order {
    addresses: vec![Address {
        street: String::new(),
    }],
};
let errors = order.validate().unwrap_err();
let ValidationErrorsKind::List(children) = &errors.errors()["addresses"] else {
    panic!("expected indexed child errors");
};
assert_eq!(children[&0].field_errors()["street"][0].code, "length");
```

## Custom Validators

Use a custom function for validation. Multiple custom validators can be applied.

```rust
use valid8r::{Validate, ValidationError};

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.chars().next().is_some_and(|c| c.is_numeric()) {
        return Err(ValidationError::new("username_starts_with_number")
            .with_message("Username cannot start with a number".into()));
    }
    Ok(())
}

fn validate_not_reserved(username: &str) -> Result<(), ValidationError> {
    let reserved = ["admin", "root", "system"];
    if reserved.contains(&username.to_lowercase().as_str()) {
        return Err(ValidationError::new("reserved_username")
            .with_message("This username is reserved".into()));
    }
    Ok(())
}

#[derive(Validate)]
struct User {
    #[validate(custom(function = validate_username))]
    #[validate(custom(function = validate_not_reserved))]
    username: String,
}
```

The function returns `Ok(())` for a valid value or a `ValidationError` for an
invalid value. Optional fields pass their inner value when present. Numeric
primitives written as `i32`, `f64`, etc. are passed by value; aliases and
qualified spellings can instead require references. Strings and collections
can use borrowed arguments such as `&str` and `&[T]`. Values passed to custom
field validators must support Serde serialization so they can be recorded
in error parameters.

## Schema Validation

Use a schema function for a rule involving the whole struct. It receives
`&Struct` and returns `Result<(), ValidationError>`:

```rust
use valid8r::{Validate, ValidationError};

fn check_date_range(form: &DateForm) -> Result<(), ValidationError> {
    if form.start_day > form.end_day {
        return Err(ValidationError::new("invalid_date_range")
            .with_message("Start day must not be after end day".into()));
    }
    Ok(())
}

#[derive(Validate)]
#[validate(schema(function = check_date_range))]
struct DateForm {
    #[validate(range(min = 1))]
    start_day: i32,
    #[validate(range(min = 1))]
    end_day: i32,
}

let invalid = DateForm {
    start_day: 10,
    end_day: 5,
};
let errors = invalid.validate().unwrap_err();
assert_eq!(
    errors.field_errors()["__all__"][0].code,
    "invalid_date_range"
);

// Field errors skip this schema function by default.
let invalid_field = DateForm {
    start_day: 0,
    end_day: 5,
};
assert!(
    !invalid_field
        .validate()
        .unwrap_err()
        .errors()
        .contains_key("__all__")
);
```

Schema errors are stored under `__all__`. By default, schema functions run
only when field validation, including nested validation, passes. Add
`skip_on_field_errors = false` to the schema attribute to run it despite
field errors:
`#[validate(schema(function = check_date_range, skip_on_field_errors = false))]`.

You can repeat the `#[validate(schema(...))]` attribute to run multiple
functions. An earlier schema failure does not suppress later schemas when
all fields were valid. Use `use_context` to pass a
[validation context](#validation-context) along with the struct.

## Validation Context

Use `ValidateArgs` to pass application data into a custom field or schema
validator. Declare its type on the struct and add `use_context` to each
function that needs it:

```rust
use valid8r::{Validate, ValidateArgs, ValidationError};

struct ValidationContext {
    max_items: usize,
}

fn check_item_limit(items: &[String], ctx: &ValidationContext) -> Result<(), ValidationError> {
    if items.len() > ctx.max_items {
        Err(ValidationError::new("too_many_items"))
    } else {
        Ok(())
    }
}

#[derive(Validate)]
#[validate(context = ValidationContext)]
struct Order {
    #[validate(custom(function = check_item_limit, use_context))]
    items: Vec<String>,
}

let ctx = ValidationContext { max_items: 1 };
let order = Order {
    items: vec!["first".into(), "second".into()],
};
let errors = order.validate_with_args(&ctx).unwrap_err();
assert_eq!(errors.field_errors()["items"][0].code, "too_many_items");
```

A struct with `context` uses `.validate_with_args(&ctx)` instead of generating
an argument-free `.validate()`. Schema functions with `use_context` receive
`(&Struct, &Context)`. For a context type containing borrowed data, use the
lifetime `'v_a` in its attribute path, for example
`#[validate(context = "ValidationContext<'v_a>")]`. Add `mutable` at the struct
level when the context must be passed as `&mut Context`.
Nested validation calls each child's `.validate()`; it does not forward
the parent's context automatically.

## Custom Error Messages and Codes

Value, presence, consent, custom, and schema rules accept `code` and
`message`. Keep codes stable for callers; use messages for display:

```rust
use valid8r::Validate;
#[derive(Validate)]
struct Login {
    #[validate(email(code = "INVALID_EMAIL", message = "Please enter a valid email address"))]
    email: String,

    #[validate(length(
        min = 8,
        code = "PASSWORD_TOO_SHORT",
        message = "Password must be at least 8 characters"
    ))]
    password: String,
}
```

## Skipping Fields

Skip validation for specific fields:

```rust
use valid8r::Validate;
#[derive(Validate)]
struct Data {
    #[validate(skip)]
    internal_id: String, // Never validated

    #[validate(email)]
    email: String,
}
```

Fields without validation attributes are otherwise left alone. `skip` is
useful when opting out of struct-wide `nest_all_fields` validation.

## Request Validation

In a web handler, deserialize the payload, validate it, and then perform the
operation. Deserialization and validation report different failures: JSON
parsing checks types and syntax, while `.validate()` checks your rules.

The following function can be called from a handler in any web framework.
For this example, add `serde` with its `derive` feature and `serde_json` as
shown in [PATCH Semantics](#patch-semantics).

```rust
use serde::Deserialize;
use valid8r::{Validate, ValidationErrors};

#[derive(Debug, Deserialize, Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8), sensitive)]
    password: String,
}

#[derive(Debug)]
enum RequestError {
    InvalidJson(serde_json::Error),
    InvalidFields(ValidationErrors),
}

fn parse_user(body: &str) -> Result<CreateUser, RequestError> {
    let user: CreateUser = serde_json::from_str(body).map_err(RequestError::InvalidJson)?;
    user.validate().map_err(RequestError::InvalidFields)?;
    Ok(user)
}

assert!(parse_user(r#"{"email":"person@example.com","password":"long-enough"}"#).is_ok());
assert!(matches!(
    parse_user(r#"{"email":"invalid","password":"long-enough"}"#),
    Err(RequestError::InvalidFields(_)),
));
assert!(matches!(
    parse_user("not JSON"),
    Err(RequestError::InvalidJson(_))
));
```

Map `InvalidJson` and `InvalidFields` to your framework's error responses.
Serialize the `ValidationErrors` in `InvalidFields` when returning field
feedback to a client.

## Validator Reference

Rules can share a single attribute, such as
`#[validate(required, email)]`, or use separate attributes on the same field.
`custom` can be repeated to call multiple functions.

| Group                                                | Rules                                                                                                                                                                                                                          |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [String/format](#stringformat-validators)            | [`email`](#email), [`url`](#url), [`phone_number`](#phone_number), [`credit_card`](#credit_card), [`ip`](#ip), [`regex`](#regex), [`non_control_character`](#non_control_character)                                            |
| [Numeric and length](#numeric-and-length-validators) | [`range`](#range), [`length`](#length)                                                                                                                                                                                         |
| [Presence](#presence-validators)                     | [`required`](#required), [`required_if`](#required_if), [`required_with`](#required_with), [`required_with_all`](#required_with_all), [`required_without`](#required_without), [`required_without_all`](#required_without_all) |
| [Prohibition](#prohibition-validators)               | [`prohibited_if`](#prohibited_if), [`prohibited_with`](#prohibited_with), [`prohibited_with_all`](#prohibited_with_all), [`prohibited_without`](#prohibited_without), [`prohibited_without_all`](#prohibited_without_all)      |
| [Containers](#container-validators)                  | [`contains`](#contains), [`does_not_contain`](#does_not_contain)                                                                                                                                                               |
| [Comparison](#comparison-validators)                 | [`must_match`](#must_match)                                                                                                                                                                                                    |
| [Consent](#consent-validators)                       | [`accepted`](#accepted), [`declined`](#declined), `accepted_if`, `declined_if`                                                                                                                                                 |
| [Custom](#custom-validators)                         | `custom`                                                                                                                                                                                                                       |

Presence and prohibition rules interpret `Some(_)` as present for `Option<T>`,
and only actual inner values as present for PATCH types. With an empty
string, an `Option<String>` is still present. Fields named in `other_fields`
must be optional (`Option<T>`, nested `Option`, or `Delta<T>`). Predicate
functions for `required_if` and `prohibited_if` receive `&Struct` and return
`bool`.

### String/Format Validators

#### `email`

Checks for an `@`, a nonempty local part matching the supported character
pattern, and length limits of 64 Unicode scalar values for the local part
and 255 for the domain. The `email` feature additionally checks domain
syntax, including internationalized domains and supported IP literals.
It does not verify that a mailbox or domain exists.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Newsletter {
    #[validate(email)]
    email: String,

    // With custom error
    #[validate(email(message = "Please provide a valid email", code = "invalid_email"))]
    contact_email: String,
}

let newsletter = Newsletter {
    email: "person@example.com".into(),
    contact_email: "person@example.com".into(),
};
assert!(newsletter.validate().is_ok());

let invalid = Newsletter {
    email: "invalid".into(),
    ..newsletter
};
assert!(invalid.validate().is_err());
```

**Supported types:** `String`, `&str`, `Cow<str>`, `Option<T>` where T implements `ValidateEmail`

#### `url`

Validates an absolute HTTP or HTTPS URL. Other schemes, including `ftp:`,
`javascript:`, and `data:`, are rejected.

```rust
# #[cfg(feature = "url")]
# {
use valid8r::Validate;

#[derive(Validate)]
struct Website {
    #[validate(url)]
    homepage: String,

    #[validate(url(message = "Invalid website URL"))]
    blog: Option<String>,
}

let mut website = Website {
    homepage: "https://example.com".into(),
    blog: None,
};
assert!(website.validate().is_ok());
website.homepage = "ftp://example.com".into();
assert!(website.validate().is_err());
# }
```

**Requires:** the `url` cargo feature (not in the default set)

#### `phone_number`

Parses phone numbers and checks their validity using `phonenumber`. The
optional `country` is a parsing hint for national-format numbers, not a
restriction on international numbers with an explicit country prefix.

```rust
# #[cfg(feature = "phone_number")]
# {
use valid8r::Validate;

#[derive(Validate)]
struct Contact {
    // International format (auto-detect country)
    #[validate(phone_number)]
    phone: String,

    // Country hint for national-format numbers
    #[validate(phone_number(country = "US"))]
    us_phone: String,

    #[validate(phone_number(country = "GB", message = "Invalid phone number"))]
    uk_phone: Option<String>,
}

let contact = Contact {
    phone: "+447911123456".into(),
    // An explicit international prefix takes precedence over the US hint.
    us_phone: "+447911123456".into(),
    uk_phone: Some("07911123456".into()),
};
assert!(contact.validate().is_ok());

#[derive(Validate)]
struct OptionalPhone {
    #[validate(phone_number(nullable))]
    phone: String,
}
assert!(OptionalPhone { phone: String::new() }.validate().is_ok());
assert!(OptionalPhone { phone: "invalid".into() }.validate().is_err());
# }
```

**Requires:** the `phone_number` cargo feature (not in the default set)

**Country codes:** Use ISO 3166-1 alpha-2 codes (e.g., "US", "GB", "DE", "FR")

#### `credit_card`

Checks credit-card format and checksum through `card-validate`. Passing
validation does not establish that a card is active or can be charged.

```rust
# #[cfg(feature = "cards")]
# {
use valid8r::Validate;

#[derive(Validate)]
struct Payment {
    #[validate(credit_card)]
    card_number: String,
}

assert!(Payment { card_number: "4539571147647251".into() }.validate().is_ok());
assert!(Payment { card_number: "4539571147647252".into() }.validate().is_err());
# }
```

**Requires:** the `cards` cargo feature (not in the default set)

#### `ip`

Validates IP addresses (v4, v6, or both).

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Server {
    // Any IP (v4 or v6)
    #[validate(ip)]
    address: String,

    // IPv4 only
    #[validate(ip(v4))]
    ipv4_address: String,

    // IPv6 only
    #[validate(ip(v6))]
    ipv6_address: String,
}
```

#### `regex`

Validates against a regular expression pattern.

```rust
use regex::Regex;
use std::sync::LazyLock;
use valid8r::Validate;

// Define regex as a static for efficiency
static USERNAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]{2,29}$").unwrap());

#[derive(Validate)]
struct User {
    #[validate(regex(path = *USERNAME_RE))]
    username: String,
}
```

Add `regex = "1"` to your dependencies. A `LazyLock` compiles the pattern once
and reuses it for each validation.

#### `non_control_character`

Validates that a string contains no control characters.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Input {
    #[validate(non_control_character)]
    user_input: String,
}
```

### Numeric and Length Validators

#### `range`

Validates that a number falls within a specified range.

Inclusive bounds alone do not reject floating-point `NaN`. Use a custom
validator that checks `.is_finite()` when non-finite values must be rejected.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Product {
    // Inclusive min and max
    #[validate(range(min = 0, max = 100))]
    quantity: i32,

    // Min only
    #[validate(range(min = 0.01))]
    price: f64,

    // Max only
    #[validate(range(max = 1000000))]
    budget: Option<i64>,

    // Exclusive bounds
    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0))]
    probability: f64,
}

let mut product = Product {
    quantity: 50,
    price: 1.0,
    budget: None,
    probability: 0.5,
};
assert!(product.validate().is_ok());
product.probability = 1.0;
assert!(product.validate().is_err());
# product.probability = 0.5;
# product.price = f64::NAN;
# assert!(product.validate().is_ok());
```

**Supported types:** All numeric primitives (`u8`-`u128`, `i8`-`i128`, `usize`, `isize`, `f32`, `f64`),
`rust_decimal::Decimal`, `Option<T>`, `Option<Option<T>>`

#### `length`

Checks the number of Unicode scalar values in a string, or the number of
items in a collection or array. Bounds are inclusive. Use `equal` on its own;
it cannot be combined with `min` or `max` in a derive attribute.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Form {
    // Min and max length
    #[validate(length(min = 3, max = 50))]
    username: String,

    // Exact length
    #[validate(length(equal = 6))]
    pin_code: String,

    // Collection length
    #[validate(length(min = 1, max = 10))]
    tags: Vec<String>,

    // Array length
    #[validate(length(equal = 3))]
    rgb: [u8; 3],
}
```

**Supported types:** `String`, `&str`, `Vec<T>`, `HashMap`, `HashSet`, `BTreeMap`,
`BTreeSet`, `VecDeque`, `[T; N]`, `&[T]`, `Cow<str>`

For strings, this is the count from `.chars()`, rather than bytes or displayed
grapheme clusters. For example, a letter followed by a combining accent
counts as two scalar values.

### Presence Validators

#### `required`

Validates that an `Option` field contains a value.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Registration {
    #[validate(required)]
    email: Option<String>,

    #[validate(required(message = "Password is required"))]
    password: Option<String>,
}
```

#### `required_if`

Conditionally requires a field based on a function.

```rust
use valid8r::Validate;
fn needs_shipping(order: &Order) -> bool {
    order.is_physical
}

#[derive(Validate)]
struct Order {
    is_physical: bool,

    #[validate(required_if(func = needs_shipping))]
    shipping_address: Option<String>,
}
```

#### `required_with`

The field is required if any of the specified fields have a value.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Address {
    // Country is required if area_id OR city_id is provided
    #[validate(required_with(other_fields("area_id", "city_id")))]
    country_id: Option<i32>,

    area_id: Option<i32>,
    city_id: Option<i32>,
}
```

#### `required_with_all`

The field is required if all specified fields have values.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct DateRange {
    start_date: Option<String>,
    end_date: Option<String>,

    // Timezone required only if both start and end dates are provided
    #[validate(required_with_all(other_fields("start_date", "end_date")))]
    timezone: Option<String>,
}
```

#### `required_without`

The field is required if any of the specified fields are missing.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Contact {
    // Email required if phone is missing
    #[validate(required_without(other_fields("phone")))]
    email: Option<String>,

    // Phone required if email is missing
    #[validate(required_without(other_fields("email")))]
    phone: Option<String>,
}
```

#### `required_without_all`

The field is required if all specified fields are missing.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Auth {
    oauth_token: Option<String>,
    api_key: Option<String>,

    // Password required only if both oauth_token AND api_key are missing
    #[validate(required_without_all(other_fields("oauth_token", "api_key")))]
    password: Option<String>,
}
```

### Prohibition Validators

#### `prohibited_if`

The field must be absent if a condition is met.

```rust
use valid8r::Validate;
fn is_readonly(form: &Form) -> bool {
    form.readonly
}

#[derive(Validate)]
struct Form {
    readonly: bool,

    #[validate(prohibited_if(func = is_readonly))]
    new_value: Option<String>,
}
```

#### `prohibited_with`

The field must be absent if any of the specified fields have values.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Transfer {
    // Cannot specify both bank transfer AND credit card
    #[validate(prohibited_with(other_fields("credit_card_id")))]
    bank_account_id: Option<String>,

    #[validate(prohibited_with(other_fields("bank_account_id")))]
    credit_card_id: Option<String>,
}
```

#### `prohibited_with_all`

The field must be absent if all specified fields have values.

#### `prohibited_without`

The field must be absent if any of the specified fields are missing.

#### `prohibited_without_all`

The field must be absent if all specified fields are missing.

### Container Validators

#### `contains`

Validates that a string contains a specific substring. Works with `String`, `&str`,
`Cow<str>`, and `HashMap<String, V>` (checks key presence).

```rust
use std::collections::HashMap;
use valid8r::Validate;

#[derive(Validate)]
struct Config {
    #[validate(contains(pattern = "@company.com"))]
    corporate_email: String,

    // For HashMap, checks if key exists
    #[validate(contains(pattern = "admin"))]
    permissions: HashMap<String, bool>,
}
```

#### `does_not_contain`

Checks that a string does not contain a substring, or a supported map does
not contain the specified key.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Content {
    #[validate(does_not_contain(pattern = "<script>"))]
    html_content: String,
}
```

### Comparison Validators

#### `must_match`

Compares a field with another field of the same underlying type. The check
runs only when the annotated field has a value. If the other field is absent
or cleared, a present value fails the comparison.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct PasswordChange {
    new_password: String,

    #[validate(must_match(other = "new_password"))]
    confirm_password: String,
}
```

### Consent Validators

#### `accepted`

Accepts `true` for booleans. For strings, accepts `"true"`, `"1"`, `"on"`, or
`"yes"`, ignoring ASCII case.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Terms {
    #[validate(accepted)]
    accept_terms: bool,

    #[validate(accepted)]
    accept_privacy: String, // Accepts "yes", "on", "1", "true"
}
```

#### `declined`

Accepts `false` for booleans. For strings, accepts every value that `accepted`
rejects, including an empty string. It does not restrict input to a fixed
list of negative answers.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct Preferences {
    #[validate(declined)]
    marketing_enabled: bool,
}
```

#### `accepted_if` / `declined_if`

Checks acceptance or decline when the predicate returns `true`. For an
`Option<bool>`, `None` skips this value check; add a `required*` rule when
absence should also fail.

```rust
use valid8r::Validate;
fn is_premium(reg: &Registration) -> bool {
    reg.plan == "premium"
}

#[derive(Validate)]
struct Registration {
    plan: String,

    #[validate(accepted_if(function = is_premium))]
    accept_premium_terms: Option<bool>,
}
```

## License

Licensed under either [MIT](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
The MIT license includes the original upstream copyright notice.
