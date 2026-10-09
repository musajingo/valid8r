use valid8r::Validate;

#[derive(Validate)]
struct Test {
    #[validate(custom(function = "hello_world", arg = "(i64, i64)"))]
    s: String,
}

fn main() {}
