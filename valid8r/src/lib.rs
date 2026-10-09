#![doc = include_str!("../../valid8r.md")]

mod traits;
mod types;
mod validators;

pub use traits::{Validate, ValidateArgs};
pub use types::{ValidationError, ValidationErrors, ValidationErrorsKind};
pub use valid8r_derive::Validate;
#[cfg(feature = "cards")]
pub use validators::cards::ValidateCreditCard;
pub use validators::consent::ValidateConsent;
pub use validators::contains::ValidateContains;
pub use validators::does_not_contain::ValidateDoesNotContain;
pub use validators::email::ValidateEmail;
pub use validators::ip::ValidateIp;
pub use validators::length::ValidateLength;
pub use validators::must_match::validate_must_match;
pub use validators::non_control_character::ValidateNonControlCharacter;
#[cfg(feature = "phone_number")]
pub use validators::phone_number::ValidatePhoneNumber;
pub use validators::prohibited::ValidateProhibited;
pub use validators::range::ValidateRange;
pub use validators::regex::{AsRegex, ValidateRegex};
pub use validators::required::ValidateRequired;
#[cfg(feature = "url")]
pub use validators::urls::ValidateUrl;
