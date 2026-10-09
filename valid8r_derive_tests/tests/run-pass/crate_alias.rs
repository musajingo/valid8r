use valid8r as valid8r_renamed;

mod inner {
    use super::valid8r_renamed;

    mod valid8r {}

    fn validate_fn(_: &str) -> Result<(), valid8r_renamed::ValidationError> {
        Ok(())
    }

    #[derive(valid8r_renamed::Validate)]
    #[validate(crate = "valid8r_renamed")]
    struct Test {
        #[validate(url)]
        url: String,
        #[validate(email)]
        email: String,
        #[validate(length(min = 1, max = 10))]
        length: String,
        #[validate(range(min = 1, max = 10))]
        range: i32,
        #[validate(custom(function = "validate_fn"))]
        custom: String,
    }
}

fn main() {}
