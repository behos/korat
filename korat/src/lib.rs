#![feature(try_from)]

#[macro_use] extern crate quick_error;

extern crate rusoto_dynamodb;

pub mod errors;

use std::collections::HashSet;
use std::convert::TryFrom;

use rusoto_dynamodb::{AttributeMap, AttributeValue};

use errors::ConversionError;


type ConversionResult<T> = Result<T, ConversionError>;


pub trait DynamoDBItem: TryFrom<AttributeMap, Error=ConversionError> {}


pub trait ValueExtractor: Sized {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<Self>;
}

impl ValueExtractor for String {
    fn extract(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value.s.ok_or(ConversionError::MissingValue)
    }
}

impl ValueExtractor for HashSet<String> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.ss.ok_or(ConversionError::MissingValue)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }
}

impl ValueExtractor for HashSet<Vec<u8>> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.bs.ok_or(ConversionError::MissingValue)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }
}

impl ValueExtractor for Vec<u8> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.b.ok_or(ConversionError::MissingValue)
    }
}

impl ValueExtractor for bool {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.bool.ok_or(ConversionError::MissingValue)
    }
}

impl <T: DynamoDBItem> ValueExtractor for T {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        attribute_value.m.ok_or(ConversionError::MissingValue)
            .and_then(|attribute_map| T::try_from(attribute_map))
    }
}

impl <T: DynamoDBItem> ValueExtractor for Vec<T> {
    fn extract(
        attribute_value: AttributeValue
    ) -> ConversionResult<Self> {
        let mut convertable_vec = attribute_value.l
            .ok_or(ConversionError::MissingValue)?;
        let results = convertable_vec
            .drain(..)
            .map(|convertable| ValueExtractor::extract(convertable))
            .collect();
        results    
    }
}


macro_rules! numeric_extractor {
    ($type:ty) => {
        impl ValueExtractor for $type {
            fn extract(
                attribute_value: AttributeValue
            ) -> ConversionResult<Self> {
                attribute_value.n
                    .ok_or(ConversionError::MissingValue)
                    .and_then(|number_string| {
                        number_string.parse()
                            .map_err(|_| ConversionError::InvalidValue)
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
                    .ok_or(ConversionError::MissingValue)?;
                let mut results: Vec<ConversionResult<$type>>= number_string_vec
                    .drain(..).map(
                        |number_string| number_string.parse()
                            .map_err(|_| ConversionError::InvalidValue))
                    .collect();
                let aggregated_result = results.drain(..).collect();
                aggregated_result
            }
        }
    }
}

numeric_extractor!(i32);
numeric_extractor!(i64);
numeric_extractor!(f32);
numeric_extractor!(f64);

numeric_set_extractor!(i32 => HashSet<i32>);
numeric_set_extractor!(i32 => Vec<i32>);
numeric_set_extractor!(i64 => HashSet<i64>);
numeric_set_extractor!(i64 => Vec<i64>);
numeric_set_extractor!(f32 => Vec<f32>);
numeric_set_extractor!(f64 => Vec<f64>);


#[cfg(test)]
mod test {
    use std::default::Default;
    use std::convert::TryFrom;
    use std::collections::HashSet;

    use rusoto_dynamodb::{AttributeValue, AttributeMap};

    use errors::ConversionError;
    use super::{ValueExtractor, DynamoDBItem};


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
        f64_tests(f64, 123.4, [Vec<f64>])
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
    fn can_extract_binary() {
        let input = vec![1, 2, 3, 4];
        let expected = vec![1, 2, 3, 4];
        let av = AttributeValue {
            b: Some(input),
            .. AttributeValue::default()
        };

        let extracted: Vec<u8> = ValueExtractor::extract(av).unwrap();
        assert_eq!(expected, extracted);
    }

    #[test]
    fn can_extract_binary_set() {
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

    #[derive(PartialEq, Debug)]
    struct Example {
        key: i32
    }

    impl DynamoDBItem for Example {}

    impl TryFrom<AttributeMap> for Example {
        type Error = ConversionError;

        fn try_from(
            mut attribute_map: AttributeMap
        ) -> Result<Self, Self::Error> {
            Ok(Example {
                key: attribute_map.remove("key").unwrap().n.unwrap().parse()
                    .unwrap()
            })
        }
    }
 
    #[test]
    fn can_extract_dynamodb_item() {

        let mut attribute_map = AttributeMap::new();
        let value = AttributeValue {
            n: Some(123.to_string()),
            .. AttributeValue::default()
        };

        attribute_map.insert("key".to_string(), value);

        let av = AttributeValue {
            m: Some(attribute_map),
            .. AttributeValue::default()
        };

        let extracted: Example = ValueExtractor::extract(av).unwrap();
        assert_eq!(Example { key: 123 }, extracted);
    }
 
    #[test]
    fn can_extract_dynamodb_item_list() {
        let mut attribute_map = AttributeMap::new();
        let value = AttributeValue {
            n: Some(123.to_string()),
            .. AttributeValue::default()
        };
        attribute_map.insert("key".to_string(), value);

        let values = vec![
            AttributeValue {
                m: Some(attribute_map.clone()),
                .. AttributeValue::default()
            },
            AttributeValue {
                m: Some(attribute_map.clone()),
                .. AttributeValue::default()
            }
        ];

        let av = AttributeValue {
            l: Some(values),
            .. AttributeValue::default()
        };

        let extracted: Vec<Example> = ValueExtractor::extract(av).unwrap();
        assert_eq!(vec![Example { key: 123 }, Example { key: 123 }], extracted);
    }
}
