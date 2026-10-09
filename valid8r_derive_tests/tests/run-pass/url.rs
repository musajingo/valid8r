use valid8r::Validate;

#[derive(Validate)]
struct Test {
    #[validate(url)]
    s: String,
}

fn main() {}
