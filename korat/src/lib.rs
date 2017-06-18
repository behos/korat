#[macro_use] extern crate quick_error;

extern crate rusoto_dynamodb;

mod errors;

use rusoto_dynamodb::AttributeValue;

use errors::ConversionError;


type ConversionResult<T> = Result<T, ConversionError>;

trait ValueExtractor: Sized {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<Self>;
}

impl ValueExtractor for String {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<String> {
        attribute_value.s.ok_or(ConversionError::Missing)
    }
}

macro_rules! numeric_extractor {
    ($type:ty) => {
        impl ValueExtractor for $type {
            fn extract(
                attribute_value: AttributeValue
            ) -> ConversionResult<$type> {
                attribute_value.n
                    .ok_or(ConversionError::Missing)
                    .and_then(|number_string| {
                        number_string.parse()
                            .map_err(|_| ConversionError::Invalid)
                    })
            }
        }
    }
}

numeric_extractor!(u8);
numeric_extractor!(i32);
numeric_extractor!(i64);
numeric_extractor!(f32);
numeric_extractor!(f64);


#[cfg(test)]
mod test {
    use std::default::Default;

    use rusoto_dynamodb::AttributeValue;
    
    use super::ValueExtractor;


    macro_rules! test_for_numeric_types {
        ($($mod_name:ident($type:ty, $valid:expr)),*) => {
            $(mod $mod_name {

                
                #[test]
                fn fails_on_missing_value() {
                    let default = ::AttributeValue::default();
                    let result: Result<$type, _> = ::ValueExtractor::extract(
                        default
                    );
                    assert!(result.is_err());
                }

                #[test]
                fn can_extract_valid() {
                    let av = ::AttributeValue {
                        n: Some(format!("{}", $valid)),
                        .. ::AttributeValue::default()
                    };

                    let extracted: $type = ::ValueExtractor::extract(av)
                        .unwrap();
                    assert_eq!($valid, extracted);
                }

                #[test]
                fn fails_on_incompatible_number_type() {
                    let av = ::AttributeValue {
                        n: Some(String::from("non-numeric")),
                        .. ::AttributeValue::default()
                    };
                    
                    let res: Result<$type, _> = ::ValueExtractor::extract(av);
                    assert!(res.is_err());
                }
            })*
        }
    }

    test_for_numeric_types![
        i32_tests(i32, 1234),
        i64_tests(i64, 1234),
        f32_tests(f32, 123.4),
        f64_tests(f64, 123.4),
        u8_tests(u8, 123)
    ];

    #[test]
    fn can_extract_string() {
        let av = AttributeValue {
            s: Some(String::from("value")),
            .. AttributeValue::default()
        };

        let extracted: String = ValueExtractor::extract(av).unwrap();
        assert_eq!("value", &extracted);
    }

    #[test]
    fn fails_on_missing_string() {
        let av = AttributeValue {
            s: Some(String::from("value")),
            .. AttributeValue::default()
        };

        let extracted: String = ValueExtractor::extract(av).unwrap();
        assert_eq!("value", &extracted);
    }

}
