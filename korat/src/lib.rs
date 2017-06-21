#[macro_use] extern crate quick_error;

extern crate rusoto_dynamodb;

pub mod errors;

use std::collections::HashSet;

use rusoto_dynamodb::AttributeValue;

use errors::ConversionError;


type ConversionResult<T> = Result<T, ConversionError>;

pub trait ValueExtractor: Sized {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<Self>;
}

impl ValueExtractor for String {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value.s.ok_or(ConversionError::Missing)
    }
}

impl ValueExtractor for HashSet<String> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.ss.ok_or(ConversionError::Missing)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }
}

impl ValueExtractor for HashSet<Vec<u8>> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.bs.ok_or(ConversionError::Missing)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }
}

impl ValueExtractor for bool {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.bool.ok_or(ConversionError::Missing)
    }
}

macro_rules! numeric_extractor {
    ($type:ty) => {
        impl ValueExtractor for $type {
            fn extract(
                attribute_value: AttributeValue
            ) -> ConversionResult<Self> {
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

macro_rules! numeric_set_extractor {
    ($type:ty => $collection:ty) => {
        impl ValueExtractor for $collection {
            fn extract(
                attribute_value: AttributeValue
            ) -> ConversionResult<Self> {
                let mut number_string_vec = attribute_value.ns
                    .ok_or(ConversionError::Missing)?;
                let mut results: Vec<ConversionResult<$type>>= number_string_vec
                    .drain(..).map(
                        |number_string| number_string.parse()
                            .map_err(|_| ConversionError::Invalid))
                    .collect();
                let aggregated_result = results.drain(..).collect();
                aggregated_result
            }
        }
    }
}

numeric_extractor!(u8);
numeric_extractor!(i32);
numeric_extractor!(i64);
numeric_extractor!(f32);
numeric_extractor!(f64);

numeric_set_extractor!(u8 => HashSet<u8>);
numeric_set_extractor!(u8 => Vec<u8>);
numeric_set_extractor!(i32 => HashSet<i32>);
numeric_set_extractor!(i32 => Vec<i32>);
numeric_set_extractor!(i64 => HashSet<i64>);
numeric_set_extractor!(i64 => Vec<i64>);
numeric_set_extractor!(f32 => Vec<f32>);
numeric_set_extractor!(f64 => Vec<f64>);


#[cfg(test)]
mod test {
    use std::default::Default;
    use std::collections::HashSet;

    use rusoto_dynamodb::AttributeValue;
    
    use super::ValueExtractor;


    macro_rules! test_for_numeric_types {
        ($(
            $mod_name:ident($type:ty, $valid:expr, [$($collection:ty),*])
        ),*) => {
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
                fn can_extract_set() {
                    let val_1 = $valid;
                    let val_2 = $valid * $valid;
                    $(
                        let av = ::AttributeValue {
                            ns: Some(vec![
                                format!("{}", val_1), format!("{}", val_2)
                            ]),
                            .. ::AttributeValue::default()
                        };

                        let extracted: $collection = ::ValueExtractor
                            ::extract(av).unwrap();
                        assert!(extracted.contains(&val_1));
                        assert!(extracted.contains(&val_2));
                    )*
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
        i32_tests(i32, 1234, [super::HashSet<i32>, Vec<i32>]),
        i64_tests(i64, 1234, [super::HashSet<i64>, Vec<i64>]),
        f32_tests(f32, 123.4, [Vec<f32>]),
        f64_tests(f64, 123.4, [Vec<f64>]),
        u8_tests(u8, 10, [super::HashSet<u8>, Vec<u8>])
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
    fn can_extract_string_set() {
        let input = vec!["one".to_string(), "two".to_string()];
        let expected_set: HashSet<String> = input.iter().cloned().collect();
        let av = AttributeValue {
            ss: Some(input),
            .. AttributeValue::default()
        };

        let extracted: HashSet<String> = ValueExtractor::extract(av).unwrap();
        assert_eq!(expected_set, extracted);
    }

    #[test]
    fn can_extract_byte_string_set() {
        let input = vec![
            "one".to_string().into_bytes(), "two".to_string().into_bytes()
        ];
        let expected_set: HashSet<Vec<u8>> = input.iter().cloned().collect();
        let av = AttributeValue {
            bs: Some(input),
            .. AttributeValue::default()
        };

        let extracted: HashSet<Vec<u8>> = ValueExtractor::extract(av).unwrap();
        assert_eq!(expected_set, extracted);
    }

    #[test]
    fn can_extract_bool() {
        let av = AttributeValue {
            bool: Some(true),
            .. AttributeValue::default()
        };

        let extracted: bool = ValueExtractor::extract(av).unwrap();
        assert_eq!(true, extracted);
    }
}
