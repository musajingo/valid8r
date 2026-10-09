use valid8r::Validate;

#[derive(Validate)]
struct NotAValidator {
    #[validate(my_custom_validator)]
    field: String,
}

fn main() {}
