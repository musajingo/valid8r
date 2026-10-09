use valid8r::Validate;

#[derive(Validate)]
struct DefaultParameters<T = ()> {
    a: T,
}

fn main() {}
