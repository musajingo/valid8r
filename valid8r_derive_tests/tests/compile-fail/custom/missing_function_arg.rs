use valid8r::Validate;

#[derive(Validate)]
struct Test {
    #[validate(custom(use_context))]
    s: String,
}

fn main() {}
