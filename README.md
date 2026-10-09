# valid8r

Struct validation for Rust via `#[derive(Validate)]`.

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
valid8r = { version = "0.1", features = ["full"] }
```

The derive macro is included. Enable individual features instead of `full`
when you only need some optional validators. Requires Rust 1.94 or later.

```rust
use valid8r::Validate;

#[derive(Validate)]
struct SignUp {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8), sensitive)]
    password: String,

    #[validate(must_match(other = "password"), sensitive)]
    password_confirmation: String,

    #[validate(phone_number(country = "US"))]
    phone: Option<String>,
}
```

Calling `.validate()` returns `Ok(())` or a structured `ValidationErrors` tree
suitable for serializing into API error responses.

## Attribution

This workspace is derived from the excellent
[validator](https://github.com/Keats/validator) crate by Vincent Prouillet
and its contributors (MIT licensed, Copyright © 2016 Vincent Prouillet). The
core design; the `Validate` trait, the derive macro approach, the error
model, and the built-in validators all comes from that project, and this fork
would not exist without it. If you just need struct validation, use the
original crate from crates.io; this fork exists to carry extensions its
projects needed.

## What this fork adds

On top of upstream's validators (`email`, `url`, `length`, `range`, `ip`,
`credit_card`, `regex`, `contains`, `does_not_contain`, `must_match`,
`non_control_character`, `required`, `nested`, `custom`, `schema`):

- **`phone_number`** — phone validation via the `phonenumber` crate, with an
  optional ISO 3166-1 country hint: `#[validate(phone_number(country = "US"))]`.
- **Conditional presence** — `required_if`, `required_with`,
  `required_with_all`, `required_without`, `required_without_all`.
- **Prohibition** — `prohibited_if`, `prohibited_with`, `prohibited_with_all`,
  `prohibited_without`, `prohibited_without_all`: the field must be absent
  when the condition holds.
- **Consent** — `accepted`, `declined`, `accepted_if`, `declined_if` for
  terms-of-service and opt-in/opt-out fields (bools or strings like
  `"yes"`/`"on"`/`"1"`).
- **PATCH semantics** — `Option<Option<T>>` and
  [`Delta<T>`](https://github.com/musajingo/delta) fields distinguish
  _absent_ / _explicit null_ / _value_; validators run only on actual values,
  and cross-field presence checks treat a cleared field as absent.
- **`sensitive`** — `#[validate(sensitive)]` strips a field's submitted value
  from every error it produces (including the `other` param of a `must_match`
  on a different field), so credentials never reach a response body or log.
- **`nullable`** on `email`/`url` — accept empty strings for clear-on-empty
  form fields.
- **`rust_decimal::Decimal`** support in `range`.
- **Non-panicking error merging** — field, nested-struct and list errors for
  the same key merge instead of panicking.

## Workspace layout

| Crate                    | Purpose                                                                                   |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| `valid8r`                | Core traits (`Validate`, `ValidateArgs`), error types, built-in validator implementations    |
| `valid8r_derive`         | The `#[derive(Validate)]` procedural macro                                                 |
| `valid8r_derive_tests`   | Integration and compile-fail tests for the derive macro (not published)                    |

## Cargo features

No features are enabled by default.

| Feature        | Enables                                               |
| -------------- | ----------------------------------------------------- |
| `email`        | Domain/host-part email validation (via `idna`)        |
| `phone_number` | `phone_number` validator (via `phonenumber`)          |
| `cards`        | `credit_card` validator (via `card-validate`)         |
| `url`          | `url` validator (via `url`)                           |
| `indexmap`     | `length`/`contains` support for `IndexMap`/`IndexSet` |
| `full`         | All of the above                                      |

## Development

```sh
./scripts/check.sh
```

runs `cargo fmt --check`, clippy with warnings denied,
the full test suite, documentation with warnings denied, a per-feature test
sweep of the `valid8r` crate, and an MSRV (`1.94`) check.
CI also builds both crates from their packaged archives.

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

Portions of this codebase are copied from or derived from
[Keats/validator](https://github.com/Keats/validator),
Copyright © 2016 Vincent Prouillet, and are used under the terms of the MIT
license. The original copyright notice is preserved in [LICENSE-MIT](LICENSE-MIT).
