# Validator - A Comprehensive Validation Framework for Rust

This crate provides a powerful, extensible validation framework for Rust structs.
It consists of two crates that work together:

- **`valid8r`** (this crate): Core validation traits and built-in validators
- **`valid8r_derive`**: Procedural macros for `#[derive(Validate)]`

It is derived from the [`validator`](https://github.com/Keats/validator) crate
by Vincent Prouillet and extends it with additional validators (phone numbers,
consent, conditional presence/prohibition), PATCH semantics, and sensitive-value
redaction.

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        HTTP Request                                  │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│        Your web layer (e.g. Axum/Actix extractors)                   │
│                                                                      │
│  1. Deserializes request body/query                                  │
│  2. Calls T::validate()                                              │
│  3. Returns validated T or structured errors                         │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│           valid8r_derive crate (Procedural Macro)                  │
│                    #[derive(Validate)]                               │
│                                                                      │
│  Generates impl Validate for T by:                                   │
│  - Parsing #[validate(...)] attributes                               │
│  - Generating validation code for each field                         │
│  - Combining results into ValidationErrors                           │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│              valid8r crate (This Crate)                              │
│                                                                      │
│  Provides:                                                           │
│  - Validate and ValidateArgs traits                                  │
│  - ValidationError and ValidationErrors types                        │
│  - Built-in validator traits (ValidateEmail, ValidateLength, etc.)   │
└─────────────────────────────────────────────────────────────────────┘
```

## Table of Contents

- [Available Validators](#available-validators)
  - [String/Format Validators](#stringformat-validators)
  - [Numeric Validators](#numeric-validators)
  - [Presence Validators](#presence-validators)
  - [Conditional Validators](#conditional-validators)
  - [Container Validators](#container-validators)
  - [Comparison Validators](#comparison-validators)
  - [Custom Validators](#custom-validators)
- [Optional Fields](#optional-fields)
- [PATCH Semantics](#patch-semantics)
  - [Using Option\<Option\<T\>\>](#using-optionoptiont)
  - [Using Delta\<T\>](#using-deltat-recommended)
- [Nested Validation](#nested-validation)
- [Schema Validation](#schema-validation)
- [Custom Error Messages and Codes](#custom-error-messages-and-codes)
- [Validation Context](#validation-context)

---

# Available Validators

## String/Format Validators

### `email`

Validates that a string is a valid email address (HTML5 spec compliant).

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Newsletter {
    #[validate(email)]
    email: String,

    // With custom error
    #[validate(email(message = "Please provide a valid email", code = "invalid_email"))]
    contact_email: String,
}
```

**Supported types:** `String`, `&str`, `Cow<str>`, `Option<T>` where T implements `ValidateEmail`

### `url`

Validates that a string is a valid URL.

```rust
# #[cfg(feature = "url")]
# mod example {
use valid8r::Validate;
#[derive(Validate)]
struct Website {
    #[validate(url)]
    homepage: String,

    #[validate(url(message = "Invalid website URL"))]
    blog: Option<String>,
}
# }
```

**Requires:** the `url` cargo feature (not in the default set)

### `phone_number`

Validates phone numbers using the `phonenumber` crate.

```rust
# #[cfg(feature = "phone_number")]
# mod example {
use valid8r::Validate;
#[derive(Validate)]
struct Contact {
    // International format (auto-detect country)
    #[validate(phone_number)]
    phone: String,

    // Country-specific validation
    #[validate(phone_number(country = "US"))]
    us_phone: String,

    #[validate(phone_number(country = "GB", message = "Invalid UK phone number"))]
    uk_phone: Option<String>,
}
# }
```

**Requires:** the `phone_number` cargo feature (not in the default set)

**Country codes:** Use ISO 3166-1 alpha-2 codes (e.g., "US", "GB", "DE", "FR")

### `credit_card`

Validates credit card numbers using the Luhn algorithm.

```rust
# #[cfg(feature = "cards")]
# mod example {
use valid8r::Validate;
#[derive(Validate)]
struct Payment {
    #[validate(credit_card)]
    card_number: String,
}
# }
```

**Requires:** the `cards` cargo feature (not in the default set)

### `ip`

Validates IP addresses (v4, v6, or both).

```rust,ignore
# use valid8r::Validate;
#[derive(Validate)]
struct Server {
    // Any IP (v4 or v6)
    #[validate(ip)]
    address: String,

    // IPv4 only
    #[validate(ip(format = "v4"))]
    ipv4_address: String,

    // IPv6 only
    #[validate(ip(format = "v6"))]
    ipv6_address: String,
}
```

### `regex`

Validates against a regular expression pattern.

```rust
use std::sync::LazyLock;
use regex::Regex;
# use valid8r::Validate;

// Define regex as a static for efficiency
static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]{2,29}$").unwrap()
});

#[derive(Validate)]
struct User {
    #[validate(regex(path = *USERNAME_RE))]
    username: String,
}
```

**Note:** Always use static/lazy patterns for performance.

### `non_control_character`

Validates that a string contains no control characters.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Input {
    #[validate(non_control_character)]
    user_input: String,
}
```

---

## Numeric Validators

### `range`

Validates that a number falls within a specified range.

```rust
# use valid8r::Validate;

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
```

**Supported types:** All numeric primitives (`u8`-`u128`, `i8`-`i128`, `f32`, `f64`),
`rust_decimal::Decimal`, `Option<T>`, `Option<Option<T>>`

### `length`

Validates the length of strings, collections, and arrays.

```rust
# use valid8r::Validate;
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

**Note:** For strings, length is measured in Unicode characters, not bytes.

---

## Presence Validators

### `required`

Validates that an `Option` field contains a value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Registration {
    #[validate(required)]
    email: Option<String>,

    #[validate(required(message = "Password is required"))]
    password: Option<String>,
}
```

### `required_if`

Conditionally requires a field based on a function.

```rust
# use valid8r::Validate;
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

### `required_with`

Field is required if ANY of the specified fields have a value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Address {
    // Country is required if area_id OR city_id is provided
    #[validate(required_with(other_fields("area_id", "city_id")))]
    country_id: Option<i32>,

    area_id: Option<i32>,
    city_id: Option<i32>,
}
```

### `required_with_all`

Field is required if ALL specified fields have values.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct DateRange {
    start_date: Option<String>,
    end_date: Option<String>,

    // Timezone required only if both start and end dates are provided
    #[validate(required_with_all(other_fields("start_date", "end_date")))]
    timezone: Option<String>,
}
```

### `required_without`

Field is required if ANY of the specified fields are missing.

```rust
# use valid8r::Validate;
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

### `required_without_all`

Field is required if ALL specified fields are missing.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Auth {
    oauth_token: Option<String>,
    api_key: Option<String>,

    // Password required only if both oauth_token AND api_key are missing
    #[validate(required_without_all(other_fields("oauth_token", "api_key")))]
    password: Option<String>,
}
```

---

## Conditional Validators (Prohibited)

### `prohibited_if`

Field must be `None` if a condition is met.

```rust
# use valid8r::Validate;
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

### `prohibited_with`

Field must be `None` if ANY of the specified fields have values.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Transfer {
    // Cannot specify both bank transfer AND credit card
    #[validate(prohibited_with(other_fields("credit_card_id")))]
    bank_account_id: Option<String>,

    #[validate(prohibited_with(other_fields("bank_account_id")))]
    credit_card_id: Option<String>,
}
```

### `prohibited_with_all`

Field must be `None` if ALL specified fields have values.

### `prohibited_without`

Field must be `None` if ANY of the specified fields are missing.

### `prohibited_without_all`

Field must be `None` if ALL specified fields are missing.

---

## Container Validators

### `contains`

Validates that a string contains a specific substring. Works with `String`, `&str`,
`Cow<str>`, and `HashMap<String, V>` (checks key presence).

```rust
# use valid8r::Validate;
use std::collections::HashMap;

#[derive(Validate)]
struct Config {
    #[validate(contains(pattern = "@company.com"))]
    corporate_email: String,

    // For HashMap, checks if key exists
    #[validate(contains(pattern = "admin"))]
    permissions: HashMap<String, bool>,
}
```

### `does_not_contain`

Validates that a string or collection does NOT contain a specific value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Content {
    #[validate(does_not_contain(pattern = "<script>"))]
    html_content: String,
}
```

---

## Comparison Validators

### `must_match`

Validates that two fields have the same value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct PasswordChange {
    new_password: String,

    #[validate(must_match(other = "new_password"))]
    confirm_password: String,
}
```

---

## Consent Validators

### `accepted`

Validates that a boolean field is `true` or a string is an accepted value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Terms {
    #[validate(accepted)]
    accept_terms: bool,

    #[validate(accepted)]
    accept_privacy: String, // Accepts "yes", "on", "1", "true"
}
```

### `declined`

Validates that a boolean field is `false` or a string is a declined value.

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Preferences {
    #[validate(declined)]
    opt_out_marketing: bool,
}
```

### `accepted_if` / `declined_if`

Conditionally requires acceptance/decline based on a function.

```rust
# use valid8r::Validate;
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

---

## Custom Validators

### `custom`

Use a custom function for validation. Multiple custom validators can be applied.

```rust
use valid8r::{Validate, ValidationError};

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.chars().next().map_or(false, |c| c.is_numeric()) {
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

---

# Optional Fields

All validators automatically handle `Option<T>` fields. When a field is `None`,
validation is skipped (unless using `required` validators).

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Profile {
    // Only validated if Some
    #[validate(email)]
    backup_email: Option<String>,

    #[validate(length(min = 3))]
    display_name: Option<String>,

    #[validate(range(min = 0, max = 100))]
    score: Option<i32>,
}

let profile = Profile {
    backup_email: None,  // Skipped
    display_name: None,  // Skipped
    score: None,         // Skipped
};
assert!(profile.validate().is_ok());
```

---

# PATCH Semantics

For PATCH (partial update) operations, you need to distinguish between three states:

- **Absent** - Field not sent in the request (don't update)
- **Null** - Field explicitly set to `null` (clear/reset the field)
- **Value** - Field has a value (update to the new value)

The validation framework supports two approaches: `Option<Option<T>>` and `Delta<T>`.

## Using `Option<Option<T>>`

The traditional approach uses nested Options with a custom deserializer:

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct UpdateUser {
    // country_id is required when area_id has a value (Some(Some(_)))
    // but NOT required when area_id is being cleared (Some(None))
    #[validate(required_with(other_fields("area_id")))]
    country_id: Option<Option<i32>>,

    area_id: Option<Option<i32>>,

    // Range is only validated when the value is Some(Some(_))
    #[validate(range(min = 0, max = 100))]
    score: Option<Option<i32>>,
}

// Example: Clear area_id without providing country_id - VALID
let update = UpdateUser {
    country_id: None,           // Not updating
    area_id: Some(None),        // Clearing to NULL
    score: None,                // Not updating
};
assert!(update.validate().is_ok());

// Example: Set area_id but missing country_id - INVALID
let update = UpdateUser {
    country_id: None,           // Not provided
    area_id: Some(Some(123)),   // Has a value
    score: None,
};
assert!(update.validate().is_err());
```

## Using `Delta<T>` (Recommended)

`Delta<T>` (from the `delta` crate) is a three-state enum that models a PATCH
field directly, with less boilerplate than `Option<Option<T>>`:

- `Delta::Unchanged` — field omitted from the request (leave the stored value)
- `Delta::Clear` — field explicitly set to `null` (clear the stored value)
- `Delta::Set(value)` — field sent with a value (set the stored value)

```rust,ignore
use delta::Delta;
use serde::Deserialize;
use valid8r::Validate;

#[derive(Deserialize, Validate)]
struct UpdateUser {
    // Only needs #[serde(default)] - no custom deserializer required!
    #[serde(default)]
    #[validate(email)]
    email: Delta<String>,

    #[serde(default)]
    #[validate(url(nullable))]
    website: Delta<String>,

    #[serde(default)]
    #[validate(required_with(other_fields("area_id")))]
    country_id: Delta<i32>,

    #[serde(default)]
    area_id: Delta<i32>,
}
```

### `Delta<T>` Helper Methods

```rust,ignore
use delta::Delta;

let unchanged: Delta<String> = Delta::Unchanged;
let clear: Delta<String> = Delta::Clear;
let set: Delta<String> = Delta::Set("hello".to_string());

// Check the state
assert!(unchanged.is_unchanged()); // field was omitted
assert!(clear.is_clear());         // field was explicitly set to null
assert!(set.is_set());             // field has a value

// Access the inner value
assert_eq!(set.value(), Some(&"hello".to_string()));
let inner: Option<String> = set.into_value();
```

### Comparison: `Option<Option<T>>` vs `Delta<T>`

| Aspect                 | `Option<Option<T>>`                           | `Delta<T>`                                            |
| ---------------------- | --------------------------------------------- | ----------------------------------------------------- |
| Serde annotation       | `#[serde(default, deserialize_with = "...")]` | `#[serde(default)]`                                   |
| Validation support     | ✅ Full                                       | ✅ Full                                               |
| Cross-field validators | ✅ Works                                      | ✅ Works                                              |
| Pattern matching       | Nested `Some(Some(_))`                        | `Delta::Set(_)`                                       |
| Helper methods         | None                                          | `is_unchanged()`, `is_clear()`, `is_set()`, `value()` |

**Semantics Summary (applies to both types):**

| State  | `Option<Option<T>>` | `Delta<T>`         | Validators Run |
| ------ | ------------------- | ------------------ | -------------- |
| Absent | `None`              | `Delta::Unchanged` | Skipped        |
| Null   | `Some(None)`        | `Delta::Clear`     | Skipped        |
| Value  | `Some(Some(v))`     | `Delta::Set(v)`    | Validated      |

### Validators that work with `Delta<T>`

All validators work with `Delta<T>`:

- **Format validators:** `email`, `url`, `phone_number`, `credit_card`, `ip`, `regex`
- **Presence validators:** `required`, `required_if`, `required_with`, `required_without`
- **Prohibition validators:** `prohibited_if`, `prohibited_with`, `prohibited_without`
- **Numeric validators:** `range`
- **String validators:** `length`, `contains`, `does_not_contain`
- **Custom validators:** `custom`

---

# Nested Validation

Validate nested structs using the `nested` attribute:

```rust
# use valid8r::Validate;
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

Use `nest_all_fields` to automatically validate all nested structs:

```rust
# use valid8r::Validate;
# #[derive(Validate)]
# struct Address { #[validate(length(min = 1))] street: String }
#[derive(Validate)]
#[validate(nest_all_fields)]
struct Company {
    address: Address,      // Automatically nested
    mailing: Address,      // Automatically nested
}
```

---

# Schema Validation

For complex cross-field validation, use schema validation:

```rust
use valid8r::{Validate, ValidationError};

fn validate_date_range(form: &DateForm) -> Result<(), ValidationError> {
    if let (Some(start), Some(end)) = (&form.start_date, &form.end_date) {
        if start > end {
            return Err(ValidationError::new("invalid_date_range")
                .with_message("Start date must be before end date".into()));
        }
    }
    Ok(())
}

#[derive(Validate)]
#[validate(schema(function = validate_date_range))]
struct DateForm {
    start_date: Option<String>,
    end_date: Option<String>,
}
```

**Multiple schema validators:**

```rust
# use valid8r::{Validate, ValidationError};
# fn validate_dates(f: &Form) -> Result<(), ValidationError> { Ok(()) }
# fn validate_amounts(f: &Form) -> Result<(), ValidationError> { Ok(()) }
#[derive(Validate)]
#[validate(schema(function = validate_dates))]
#[validate(schema(function = validate_amounts))]
struct Form {
    // fields...
#   start: String,
}
```

**Note:** Schema validation only runs if all field validations pass (to avoid
operating on invalid data). Use `skip_on_field_errors = false` to override:

```rust
# use valid8r::{Validate, ValidationError};
# fn always_validate(f: &Form) -> Result<(), ValidationError> { Ok(()) }
#[derive(Validate)]
#[validate(schema(function = always_validate, skip_on_field_errors = false))]
struct Form {
    // fields...
#   name: String,
}
```

---

# Custom Error Messages and Codes

All validators support custom error messages and codes:

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Login {
    #[validate(email(
        code = "INVALID_EMAIL",
        message = "Please enter a valid email address"
    ))]
    email: String,

    #[validate(length(
        min = 8,
        code = "PASSWORD_TOO_SHORT",
        message = "Password must be at least 8 characters"
    ))]
    password: String,
}
```

---

# Validation Context

Pass additional context to validators using `ValidateArgs`:

```rust
use valid8r::{Validate, ValidateArgs, ValidationError};

struct ValidationContext {
    max_items: usize,
    allowed_domains: Vec<String>,
}

fn validate_with_context(
    items: &[String],
    ctx: &ValidationContext
) -> Result<(), ValidationError> {
    if items.len() > ctx.max_items {
        return Err(ValidationError::new("too_many_items"));
    }
    Ok(())
}

#[derive(Validate)]
#[validate(context = ValidationContext)]
struct Order {
    #[validate(custom(function = validate_with_context, use_context))]
    items: Vec<String>,
}

// Usage:
let ctx = ValidationContext {
    max_items: 10,
    allowed_domains: vec!["example.com".to_string()],
};
let order = Order { items: vec!["item1".to_string()] };
order.validate_with_args(&ctx);
```

---

# Skipping Fields

Skip validation for specific fields:

```rust
# use valid8r::Validate;
#[derive(Validate)]
struct Data {
    #[validate(skip)]
    internal_id: String,  // Never validated

    #[validate(email)]
    email: String,
}
```

---

# Feature Flags

No features are enabled by default.

- `email`: Enable validation of the host part of an email (via `idna`)
- `url`: Enable URL validation
- `phone_number`: Enable phone number validation
- `cards`: Enable credit card validation
- `indexmap`: Enable validation for `IndexMap` and `IndexSet`
- `full`: Enable all of the above

---

# Error Handling

Validation errors are collected in [`ValidationErrors`]:

```rust
# use valid8r::Validate;
# #[derive(Validate)]
# struct User { #[validate(email)] email: String }
let user = User { email: "invalid".to_string() };

match user.validate() {
    Ok(()) => println!("Valid!"),
    Err(errors) => {
        for (field, field_errors) in errors.field_errors() {
            for error in field_errors {
                println!(
                    "Field '{}': {} (code: {})",
                    field,
                    error.message.as_deref().unwrap_or("validation failed"),
                    error.code
                );
            }
        }
    }
}
```

---

# Integration with web frameworks

Validation composes naturally with extractors: deserialize the payload,
call `validate()`, and reject the request with the structured errors on
failure. A typical Axum extractor looks like:

```rust,ignore
use axum::{Router, routing::post};
use serde::Deserialize;
use valid8r::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8), sensitive)]
    password: String,
}

// A custom extractor that deserializes JSON and then calls `validate()`,
// turning `ValidationErrors` into a 422 response.
async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUser>
) -> &'static str {
    // payload is guaranteed to be valid here
    "User created!"
}

let app = Router::new().route("/users", post(create_user));
```
