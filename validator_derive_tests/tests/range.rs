use validator::{Validate, ValidationErrors};

const MAX_CONST: usize = 10;
const MIN_CONST: usize = 0;

// Loose floating point comparison using EPSILON error bound
macro_rules! assert_float {
    ($e1:expr, $e2:expr) => {
        assert!(($e2 - $e1).abs() < f64::EPSILON);
    };
}

#[test]
fn can_validate_range_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5, max = 10))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_range_exclusive_min_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = 5, max = 10))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_range_exclusive_max_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5, exclusive_max = 10))]
        val: usize,
    }

    let s = TestStruct { val: 9 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_exclusive_min_and_max_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = 5, exclusive_max = 10))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_min_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_max_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(max = 50))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_exclusive_min_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = 5))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_only_exclusive_max_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_max = 50))]
        val: usize,
    }

    let s = TestStruct { val: 49 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_range_value_crate_path_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "MIN_CONST", max = "MAX_CONST"))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_exclusive_range_value_crate_path_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = "MIN_CONST", max = "MAX_CONST"))]
        val: usize,
    }

    let s = TestStruct { val: 6 };

    assert!(s.validate().is_ok());
}

#[test]
fn value_out_of_range_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5, max = 10))]
        val: usize,
    }

    let s = TestStruct { val: 11 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].code, "range");
}

#[test]
fn value_out_of_range_fails_validation_with_crate_path() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "MIN_CONST", max = "MAX_CONST"))]
        val: usize,
    }

    let s = TestStruct { val: 16 };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    println!("{}", err);
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].code, "range");
}

#[test]
fn can_specify_code_for_range() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5, max = 10, code = "oops"))]
        val: usize,
    }
    let s = TestStruct { val: 11 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].code, "oops");
    assert_eq!(errs["val"][0].params["value"], 11);
    assert_float!(errs["val"][0].params["min"].as_f64().unwrap(), 5.0);
    assert_float!(errs["val"][0].params["max"].as_f64().unwrap(), 10.0);
}

#[test]
fn can_specify_message_for_range() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = 5, max = 10, message = "oops"))]
        val: usize,
    }
    let s = TestStruct { val: 1 };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("val"));
    assert_eq!(errs["val"].len(), 1);
    assert_eq!(errs["val"][0].clone().message.unwrap(), "oops");
}

#[test]
fn can_pass_reference_as_validate() {
    // This tests that the blanket Validate implementation on
    // `&T where T:Validate` works properly

    #[derive(Validate)]
    struct TestStruct {
        #[validate(range(min = 100))]
        num_field: u32,
    }

    fn validate<T: Validate>(value: T) -> Result<(), ValidationErrors> {
        value.validate()
    }

    let val = TestStruct { num_field: 10 };
    validate(&val).unwrap_err();
    assert_eq!(val.num_field, 10);
}

#[test]
fn can_validate_option() {
    #[derive(Validate)]
    struct TestStruct {
        #[validate(range(min = 100))]
        num_field: Option<u32>,
        #[validate(range(min = 5, exclusive_max = 10))]
        exl_field: Option<Option<u8>>,
    }

    let t = TestStruct {
        num_field: Some(101),
        exl_field: Some(Some(9)),
    };
    assert!(t.validate().is_ok());
}

#[test]
fn can_validate_none_values() {
    #[derive(Validate)]
    struct TestStruct {
        #[validate(range(min = 100))]
        num_field: Option<u32>,
        #[validate(range(min = 5, exclusive_max = 10))]
        exl_field: Option<Option<u8>>,
    }

    let t = TestStruct {
        num_field: None,
        exl_field: None,
    };
    assert!(t.validate().is_ok());
}

// Tests for rust_decimal::Decimal support

const DECIMAL_ZERO: rust_decimal::Decimal = rust_decimal::Decimal::ZERO;
const DECIMAL_MIN: rust_decimal::Decimal = rust_decimal::Decimal::ZERO;
const DECIMAL_MAX: rust_decimal::Decimal = rust_decimal::Decimal::from_parts(10000, 0, 0, false, 0);
const DECIMAL_HUNDRED: rust_decimal::Decimal =
    rust_decimal::Decimal::from_parts(100, 0, 0, false, 0);

#[test]
fn can_validate_decimal_range_ok() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_MIN", max = "DECIMAL_MAX"))]
        salary: Decimal,
    }

    let s = TestStruct {
        salary: Decimal::from_str("1250.50").unwrap(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_decimal_min_only_ok() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_ZERO", message = "salary must be non-negative"))]
        salary: Decimal,
    }

    let s = TestStruct {
        salary: Decimal::from_str("1250.50").unwrap(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn decimal_below_min_fails_validation() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_ZERO", message = "salary must be non-negative"))]
        salary: Decimal,
    }

    let s = TestStruct {
        salary: Decimal::from_str("-100.00").unwrap(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("salary"));
    assert_eq!(errs["salary"].len(), 1);
    assert_eq!(errs["salary"][0].code, "range");
    assert_eq!(
        errs["salary"][0].clone().message.unwrap(),
        "salary must be non-negative"
    );
}

#[test]
fn can_validate_decimal_option_ok() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_ZERO"))]
        salary: Option<Decimal>,
    }

    let s = TestStruct {
        salary: Some(Decimal::from_str("1250.50").unwrap()),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_decimal_option_none() {
    use rust_decimal::Decimal;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_ZERO"))]
        salary: Option<Decimal>,
    }

    let s = TestStruct { salary: None };

    assert!(s.validate().is_ok());
}

#[test]
fn decimal_option_below_min_fails() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_ZERO"))]
        salary: Option<Decimal>,
    }

    let s = TestStruct {
        salary: Some(Decimal::from_str("-50.00").unwrap()),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert!(errs.contains_key("salary"));
}

#[test]
fn can_validate_decimal_exclusive_range() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = "DECIMAL_ZERO", exclusive_max = "DECIMAL_HUNDRED"))]
        percentage: Decimal,
    }

    let s = TestStruct {
        percentage: Decimal::from_str("50.5").unwrap(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn decimal_exclusive_range_boundary_fails() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(exclusive_min = "DECIMAL_ZERO", exclusive_max = "DECIMAL_HUNDRED"))]
        percentage: Decimal,
    }

    // Test exclusive_min boundary
    let s = TestStruct {
        percentage: Decimal::from_str("0.0").unwrap(),
    };

    let res = s.validate();
    assert!(res.is_err());

    // Test exclusive_max boundary
    let s2 = TestStruct {
        percentage: Decimal::from_str("100.0").unwrap(),
    };

    let res2 = s2.validate();
    assert!(res2.is_err());
}

#[test]
fn can_validate_decimal_with_precision() {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    const DECIMAL_MIN_PRICE: rust_decimal::Decimal =
        rust_decimal::Decimal::from_parts(1, 0, 0, false, 2); // 0.01
    const DECIMAL_MAX_PRICE: rust_decimal::Decimal =
        rust_decimal::Decimal::from_parts(99999999, 0, 0, false, 2); // 999999.99

    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(range(min = "DECIMAL_MIN_PRICE", max = "DECIMAL_MAX_PRICE"))]
        price: Decimal,
    }

    // Test with precise decimal value
    let s = TestStruct {
        price: Decimal::from_str("123.45").unwrap(),
    };

    assert!(s.validate().is_ok());

    // Test edge case with many decimal places
    let s2 = TestStruct {
        price: Decimal::from_str("99.999999").unwrap(),
    };

    assert!(s2.validate().is_ok());
}
