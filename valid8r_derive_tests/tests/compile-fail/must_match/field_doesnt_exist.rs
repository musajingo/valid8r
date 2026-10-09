use valid8r::Validate;

#[derive(Validate)]
struct Test {
    #[validate(must_match(other = password2))]
    password: String,
}

fn main() {}
