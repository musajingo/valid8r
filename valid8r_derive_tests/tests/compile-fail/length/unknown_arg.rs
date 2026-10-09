use valid8r::Validate;

#[derive(Validate)]
struct Test {
    #[validate(length(eq = 2))]
    s: String,
}

fn main() {}
