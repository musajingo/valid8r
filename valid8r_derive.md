# Validator Derive - Procedural Macros for Validation

This crate provides the `#[derive(Validate)]` procedural macro that generates
validation implementations for Rust structs. It is part of the validation
framework and works in conjunction with the `valid8r` crate.

## Overview

The derive macro parses `#[validate(...)]` attributes on struct fields and
generates the `impl Validate for T` block that performs runtime validation.

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Your Struct Definition                       │
│  #[derive(Validate)]                                           │
│  struct User {                                                  │
│      #[validate(email)]                                        │
│      email: String,                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼ (compile time)
┌─────────────────────────────────────────────────────────────────┐
│                    valid8r_derive (This Crate)                │
│                                                                 │
│  1. Parse #[validate(...)] attributes using darling            │
│  2. Generate validation code for each field                     │
│  3. Output impl Validate for User { ... }                      │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼ (runtime)
┌─────────────────────────────────────────────────────────────────┐
│  user.validate()  →  Runs generated validation code            │
│                   →  Returns Ok(()) or Err(ValidationErrors)   │
└─────────────────────────────────────────────────────────────────┘
```

## Basic Usage

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8))]
    password: String,
}

let user = CreateUser {
    email: "test@example.com".to_string(),
    password: "securepass".to_string(),
};

assert!(user.validate().is_ok());
```

## Struct-Level Attributes

These attributes are placed on the struct itself:

### `context`

Pass a validation context to custom validators:

```rust,ignore
use valid8r::{Validate, ValidateArgs, ValidationError};

struct MyContext {
    max_length: usize,
}

fn validate_with_ctx(val: &str, ctx: &MyContext) -> Result<(), ValidationError> {
    if val.len() > ctx.max_length {
        return Err(ValidationError::new("too_long"));
    }
    Ok(())
}

#[derive(Validate)]
#[validate(context = MyContext)]
struct Form {
    #[validate(custom(function = validate_with_ctx, use_context))]
    value: String,
}

let form = Form { value: "test".to_string() };
let ctx = MyContext { max_length: 10 };
form.validate_with_args(&ctx);
```

### `mutable`

Allow mutable access to the context:

```rust,ignore
#[derive(Validate)]
#[validate(context = MyContext, mutable)]
struct Form { /* ... */ }
```

### `nest_all_fields`

Automatically validate all nested structs:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct Address {
    #[validate(length(min = 1))]
    street: String,
}

#[derive(Validate)]
#[validate(nest_all_fields)]
struct User {
    home: Address,     // Automatically validated
    work: Address,     // Automatically validated
}
```

### `schema`

Add struct-level validation functions:

```rust,ignore
use valid8r::{Validate, ValidationError};

fn validate_range(form: &DateRange) -> Result<(), ValidationError> {
    if form.start > form.end {
        return Err(ValidationError::new("invalid_range"));
    }
    Ok(())
}

#[derive(Validate)]
#[validate(schema(function = validate_range))]
struct DateRange {
    start: u32,
    end: u32,
}
```

Schema options:

- `function = path::to::fn` - The validation function
- `skip_on_field_errors = false` - Run even if field validation failed (default: true)

---

## Field-Level Attributes

These attributes are placed on struct fields using `#[validate(...)]`:

### Format Validators

| Attribute               | Description              | Options                                 |
| ----------------------- | ------------------------ | --------------------------------------- |
| `email`                 | Valid email address      | `message`, `code`                       |
| `url`                   | Valid URL                | `message`, `code`                       |
| `phone_number`          | Valid phone number       | `country`, `message`, `code`            |
| `credit_card`           | Valid credit card (Luhn) | `message`, `code`                       |
| `ip`                    | Valid IP address         | `format` ("v4"/"v6"), `message`, `code` |
| `non_control_character` | No control characters    | `message`, `code`                       |
| `regex`                 | Matches regex pattern    | `path`, `message`, `code`               |

### Numeric Validators

| Attribute | Description              | Options                                                           |
| --------- | ------------------------ | ----------------------------------------------------------------- |
| `range`   | Numeric range            | `min`, `max`, `exclusive_min`, `exclusive_max`, `message`, `code` |
| `length`  | String/collection length | `min`, `max`, `equal`, `message`, `code`                          |

### Presence Validators

| Attribute              | Description                    | Options                                |
| ---------------------- | ------------------------------ | -------------------------------------- |
| `required`             | Field must be Some             | `message`, `code`                      |
| `required_if`          | Required if condition          | `func`, `message`, `code`              |
| `required_with`        | Required if any field present  | `other_fields(...)`, `message`, `code` |
| `required_with_all`    | Required if all fields present | `other_fields(...)`, `message`, `code` |
| `required_without`     | Required if any field absent   | `other_fields(...)`, `message`, `code` |
| `required_without_all` | Required if all fields absent  | `other_fields(...)`, `message`, `code` |

### Prohibition Validators

| Attribute                | Description                        | Options                                |
| ------------------------ | ---------------------------------- | -------------------------------------- |
| `prohibited_if`          | Must be None if condition          | `func`, `message`, `code`              |
| `prohibited_with`        | Must be None if any field present  | `other_fields(...)`, `message`, `code` |
| `prohibited_with_all`    | Must be None if all fields present | `other_fields(...)`, `message`, `code` |
| `prohibited_without`     | Must be None if any field absent   | `other_fields(...)`, `message`, `code` |
| `prohibited_without_all` | Must be None if all fields absent  | `other_fields(...)`, `message`, `code` |

### Container Validators

| Attribute          | Description           | Options                      |
| ------------------ | --------------------- | ---------------------------- |
| `contains`         | Contains value        | `pattern`, `message`, `code` |
| `does_not_contain` | Doesn't contain value | `pattern`, `message`, `code` |

### Comparison Validators

| Attribute    | Description          | Options                    |
| ------------ | -------------------- | -------------------------- |
| `must_match` | Equals another field | `other`, `message`, `code` |

### Consent Validators

| Attribute     | Description                      | Options                       |
| ------------- | -------------------------------- | ----------------------------- |
| `accepted`    | Boolean true or accepted string  | `message`, `code`             |
| `declined`    | Boolean false or declined string | `message`, `code`             |
| `accepted_if` | Accepted if condition            | `function`, `message`, `code` |
| `declined_if` | Declined if condition            | `function`, `message`, `code` |

### Other Validators

| Attribute | Description                | Options                                      |
| --------- | -------------------------- | -------------------------------------------- |
| `custom`  | Custom validation function | `function`, `use_context`, `message`, `code` |
| `nested`  | Validate nested struct     | -                                            |
| `skip`    | Skip this field            | -                                            |

---

## Multiple Validators

Apply multiple validators to a single field:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct User {
    // All validators must pass
    #[validate(length(min = 3, max = 20))]
    #[validate(regex(path = *USERNAME_RE))]
    username: String,
}

# use std::sync::LazyLock;
# use regex::Regex;
# static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z]+$").unwrap());
```

Or combine in a single attribute:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct User {
    #[validate(email, required)]
    email: Option<String>,
}
```

---

## Option, `Option<Option<T>>`, and `Delta<T>` Handling

The derive macro automatically handles optional fields:

### `Option<T>`

- `None` → Validation skipped (unless `required`)
- `Some(value)` → Validate the inner value

### `Option<Option<T>>` (PATCH semantics)

For partial update operations, the macro auto-detects `Option<Option<T>>`:

- `None` → Field omitted, validation skipped
- `Some(None)` → Clearing field, validation skipped
- `Some(Some(value))` → Validate the inner value

This affects cross-field validators like `required_with`:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct UpdateAddress {
    // country_id required only when area_id has a value (Some(Some(_)))
    // NOT required when area_id is being cleared (Some(None))
    #[validate(required_with(other_fields("area_id")))]
    country_id: Option<Option<i32>>,

    area_id: Option<Option<i32>>,
}

// Clearing area_id - country_id NOT required
let update = UpdateAddress {
    country_id: None,
    area_id: Some(None),  // Clearing
};
assert!(update.validate().is_ok());

// Setting area_id - country_id IS required
let update = UpdateAddress {
    country_id: None,
    area_id: Some(Some(123)),  // Has value
};
assert!(update.validate().is_err());
```

### `Delta<T>` (Recommended for PATCH)

`Delta<T>` from the `delta` crate is a three-state enum (`Unchanged` / `Clear` /
`Set(value)`) that provides the same semantics as `Option<Option<T>>` with less
boilerplate. The derive macro automatically recognizes `Delta<T>` and treats
only `Delta::Set(_)` as a value to validate.

```rust,ignore
use delta::Delta;
use serde::Deserialize;
use valid8r::Validate;

#[derive(Deserialize, Validate)]
struct UpdateVendor {
    // Only needs #[serde(default)] - no custom deserializer required
    #[serde(default)]
    #[validate(length(min = 1, max = 100))]
    name: Delta<String>,

    #[serde(default)]
    #[validate(email)]
    email: Delta<String>,

    #[serde(default)]
    #[validate(url(nullable))]
    website: Delta<String>,

    // Cross-field validation works with Delta
    #[serde(default)]
    #[validate(required_with(other_fields("area_id")))]
    country_id: Delta<i32>,

    #[serde(default)]
    area_id: Delta<i32>,
}
```

**Why use `Delta<T>` over `Option<Option<T>>`?**

| `Option<Option<T>>`                                    | `Delta<T>`                                            |
| ------------------------------------------------------ | ----------------------------------------------------- |
| Requires `#[serde(default, deserialize_with = "...")]` | Only `#[serde(default)]`                              |
| No helper methods                                      | `is_unchanged()`, `is_clear()`, `is_set()`, `value()` |
| Nested `Some(Some(_))` patterns                        | `Delta::Set(_)` patterns or helper methods            |

**Semantics (identical for both types):**

| State  | `Option<Option<T>>` | `Delta<T>`         | Validators |
| ------ | ------------------- | ------------------ | ---------- |
| Absent | `None`              | `Delta::Unchanged` | Skipped    |
| Null   | `Some(None)`        | `Delta::Clear`     | Skipped    |
| Value  | `Some(Some(v))`     | `Delta::Set(v)`    | Run        |

---

## Custom Validators

### Basic Custom Validator

```rust,ignore
use valid8r::{Validate, ValidationError};

fn validate_username(name: &str) -> Result<(), ValidationError> {
    if name.starts_with("admin") {
        return Err(ValidationError::new("reserved_prefix")
            .with_message("Username cannot start with 'admin'".into()));
    }
    Ok(())
}

#[derive(Validate)]
struct User {
    #[validate(custom(function = validate_username))]
    username: String,
}
```

### Custom Validator with Context

```rust,ignore
use valid8r::{Validate, ValidateArgs, ValidationError};

struct Config {
    forbidden_words: Vec<String>,
}

fn validate_no_forbidden(text: &str, ctx: &Config) -> Result<(), ValidationError> {
    for word in &ctx.forbidden_words {
        if text.contains(word) {
            return Err(ValidationError::new("forbidden_word"));
        }
    }
    Ok(())
}

#[derive(Validate)]
#[validate(context = Config)]
struct Post {
    #[validate(custom(function = validate_no_forbidden, use_context))]
    content: String,
}
```

---

## Generated Code

For a struct like:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct User {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8))]
    password: String,
}
```

The macro generates approximately:

```rust,ignore
impl valid8r::Validate for User {
    fn validate(&self) -> Result<(), valid8r::ValidationErrors> {
        use valid8r::ValidateEmail;
        use valid8r::ValidateLength;

        let mut errors = valid8r::ValidationErrors::new();

        // Email validation
        if !self.email.validate_email() {
            let mut err = valid8r::ValidationError::new("email");
            err.add_param(std::borrow::Cow::from("value"), &self.email);
            errors.add("email", err);
        }

        // Length validation
        if !self.password.validate_length(Some(8), None, None) {
            let mut err = valid8r::ValidationError::new("length");
            err.add_param(std::borrow::Cow::from("value"), &self.password);
            err.add_param(std::borrow::Cow::from("min"), &8);
            errors.add("password", err);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## Error Messages and Codes

All validators accept `message` and `code` parameters:

```rust,ignore
use valid8r::Validate;

#[derive(Validate)]
struct Form {
    #[validate(email(
        code = "INVALID_EMAIL",
        message = "Please enter a valid email address"
    ))]
    email: String,

    #[validate(length(
        min = 8,
        max = 100,
        code = "PASSWORD_LENGTH",
        message = "Password must be 8-100 characters"
    ))]
    password: String,
}
```

---

## Feature Flags

The derive macro respects the feature flags from the `valid8r` crate:

- `email` - Email validation with IDN support
- `url` - URL validation
- `phone_number` - Phone number validation
- `cards` - Credit card validation

---

## Implementation Details

This crate uses:

- [`darling`](https://docs.rs/darling) for attribute parsing
- [`syn`](https://docs.rs/syn) for Rust syntax parsing
- [`quote`](https://docs.rs/quote) for code generation
- [`proc-macro2`](https://docs.rs/proc-macro2) for token manipulation

The macro generates both `impl Validate` and `impl ValidateArgs` to support
validation with and without context.
