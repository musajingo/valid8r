# valid8r

Validate Rust structs with `#[derive(Validate)]`. Rules produce structured
errors for fields, nested values, and the whole struct. The crate supports
conditional presence, consent checks, and partial-update payloads.

`valid8r` is a fork of [`validator`](https://github.com/Keats/validator),
created by Vincent Prouillet and its contributors. The upstream copyright
notice is preserved in [LICENSE-MIT](LICENSE-MIT).

## Contents

- [Quick Start](#quick-start)
- [Feature Flags](#feature-flags)
- [Validation Rules](#validation-rules)
- [Working with Errors](#working-with-errors)
- [Documentation](#documentation)
- [Development](#development)
- [License](#license)

## Quick Start

Requires Rust 1.94 or later. Add this dependency to `Cargo.toml`:

```toml
[dependencies]
valid8r = { version = "0.2", features = ["email"] }
```

The derive macro is included. The `email` feature adds domain validation;
without it, the email rule checks the local part and lengths but does not
check domain syntax.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct SignUp {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8), sensitive)]
    password: String,

    #[validate(must_match(other = "password"), sensitive)]
    confirmation: String,
}

let mut signup = SignUp {
    email: "person@example.com".into(),
    password: "long-enough-password".into(),
    confirmation: "long-enough-password".into(),
};
assert!(signup.validate().is_ok());

signup.email = "invalid".into();
signup.password = "short".into();
let errors = signup.validate().unwrap_err();
assert_eq!(errors.field_errors()["email"][0].code, "email");
assert_eq!(errors.field_errors()["password"][0].code, "length");
assert!(
    !errors.field_errors()["password"][0]
        .params
        .contains_key("value")
);
assert!(
    !errors.field_errors()["confirmation"][0]
        .params
        .contains_key("other")
);
```

Call `.validate()` after constructing or deserializing a struct. It returns
`Ok(())` when the rules pass, or `Err(ValidationErrors)` when they fail.
Validation checks the values without changing them; deriving `Deserialize`
does not automatically run validation.

## Feature Flags

No optional features are enabled by default. The derive macro and basic
email, length, range, IP, presence, consent, nested, and custom validation
are available without enabling a feature.

| Feature | What it enables |
| --- | --- |
| `email` | Email domain syntax checks and internationalized domain handling through `idna` |
| `url` | Absolute HTTP/HTTPS URL validation |
| `phone_number` | Phone validation through `phonenumber`, with an optional country hint |
| `cards` | Credit-card validation through `card-validate` |
| `indexmap` | `length` validation for `IndexMap` and `IndexSet` |
| `full` | All optional features above |

For all optional validators:

```toml
[dependencies]
valid8r = { version = "0.2", features = ["full"] }
```

You can instead select individual features, such as
`features = ["email", "url", "phone_number"]`. Features belong to the
`valid8r` runtime crate; `valid8r_derive` has no feature flags of its own.

## Validation Rules

| Purpose | Rules and behavior |
| --- | --- |
| Strings and formats | `email`, `url`, `phone_number`, `credit_card`, `ip`, `regex`, `non_control_character` |
| Bounds | `length` for Unicode scalar counts or collection sizes; `range` for numeric types, including `rust_decimal::Decimal` |
| Presence | `required`, `required_if`, `required_with`, `required_with_all`, `required_without`, `required_without_all` |
| Prohibition | `prohibited_if`, `prohibited_with`, `prohibited_with_all`, `prohibited_without`, `prohibited_without_all` |
| Consent | `accepted`, `declined`, `accepted_if`, `declined_if` |
| Comparisons and custom rules | `must_match`, repeatable `custom` functions, and struct-level `schema` functions |
| Nested data | `nested` for child structs or collections of validating values; `nest_all_fields` for struct-wide nested validation |

Value rules check `Some(value)` and skip `None`. Presence and prohibition
rules still run on absent values. `required` checks presence; add
`length(min = 1)` when a present string must also be nonempty.

For PATCH payloads, `Option<Option<T>>` and
[`Delta<T>`](https://github.com/musajingo/delta) distinguish omitted, cleared,
and set fields. Only actual values count as present and reach value
validators. Nested `Option`s need a custom Serde deserializer to preserve
explicit JSON nulls. Delta comes from the Git repository linked above.
See the [PATCH guide](valid8r/README.md#patch-semantics) for dependencies and
working examples.

`email(nullable)`, `url(nullable)`, and `phone_number(nullable)` accept empty
strings. They do not convert those strings to `None` or database nulls.
Other rules on the same field still apply.

## Working with Errors

An error has a machine-readable `code`, an optional `message`, and a
`params` map. `.field_errors()` exposes direct field errors. `.errors()` and
serialization preserve nested errors too. Struct-level schema errors use
`__all__`.

Add `serde_json = "1"` to your dependencies to serialize the error tree:

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
let json = serde_json::to_value(&errors).unwrap();
assert_eq!(
    json,
    serde_json::json!({
        "email": [{
            "code": "email",
            "message": "Enter a valid email address",
            "params": { "value": "invalid" }
        }]
    })
);
```

`sensitive` removes a field's `value` parameter and suppresses its `other`
parameter when referenced by `must_match`. It does not sanitize arbitrary
custom messages or parameters, and a parent annotation does not redact
nested children's errors. Mark sensitive child fields individually.

Error merging handles field/nested collisions without panicking. When a
field has both scalar and nested errors, the nested errors take precedence;
the scalar errors are not preserved under that same key.

## Documentation

| Readme | Purpose |
| --- | --- |
| [valid8r](valid8r/README.md) | Runtime guide, error handling, PATCH examples, and the full validator reference |
| [valid8r_derive](valid8r_derive/README.md) | Derive attributes, context passing, schema rules, and crate aliases |

The workspace contains the runtime crate, the procedural macro crate, and
`valid8r_derive_tests`, a private crate for integration tests, compile-fail
tests, and executable README examples.

## Development

Install the stable toolchain with `rustfmt` and `clippy`, and the Rust 1.94
toolchain, then run:

```sh
./scripts/check.sh
```

The script checks formatting, clippy with warnings denied, the workspace
test suite, README examples (including those normally ignored by standalone
crate doctests), documentation with warnings denied, the runtime's individual
feature configurations, and a Rust 1.94 build check.

To run just the examples from all three READMEs:

```sh
cargo +stable test -p valid8r_derive_tests --all-features --doc -- --include-ignored
```

CI also builds both public crates from their package archives.

## License

Licensed under either [MIT](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

Portions of the code are derived from
[Keats/validator](https://github.com/Keats/validator),
Copyright © 2016 Vincent Prouillet and the validator crate contributors,
and are used under the MIT license. The original notice is preserved in
[LICENSE-MIT](LICENSE-MIT).
