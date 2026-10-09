use valid8r as valid8r_renamed;

mod inner {
    use super::valid8r_renamed;

    mod valid8r {}

    #[derive(valid8r_renamed::Validate)]
    #[validate(crate = "validator_other")]
    struct Test {
        #[validate(url)]
        val: String,
    }
}

fn main() {}
