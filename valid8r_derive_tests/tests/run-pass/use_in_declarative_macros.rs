macro_rules! generate_struct {
    ($name:ident,$variant:ident,$typ:ty) => {
        #[derive(Default, valid8r::Validate)]
        pub struct $name {
            $variant: $typ,
        }
    };
}
generate_struct!(TestStruct, val, String);

fn main() {}
