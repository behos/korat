#![feature(try_from, decl_macro)]

#[macro_use]
extern crate quick_error;

extern crate rusoto_dynamodb;

pub mod errors;

use std::collections::{HashSet, HashMap};
use std::convert::TryFrom;

use rusoto_dynamodb::AttributeValue;

use errors::ConversionError;


type ConversionResult<T> = Result<T, ConversionError>;


mod macros {

    use super::{
        AttributeValue, ConversionError, ConversionResult, AttributeValueConverter
    };

    pub macro attribute_value($field:ident, $value:expr) {
        AttributeValue {
            $field: Some($value),
            .. AttributeValue::default()
        }
    }

    pub macro numeric_converter($type:ty) {
        impl AttributeValueConverter for $type {
            fn from_attribute_value(
                attribute_value: AttributeValue
            ) -> ConversionResult<Self> {
                attribute_value.n
                    .ok_or(ConversionError::MissingValue)
                    .and_then(|number_string| {
                        number_string.parse()
                            .map_err(|_| ConversionError::InvalidValue)
                    })
            }
            
            fn to_attribute_value(self) -> AttributeValue {
                attribute_value!(n, self.to_string())
            }
        }
    }

    pub macro numeric_set_converter($type:ty => $collection:ty) {
        impl AttributeValueConverter for $collection {
            fn from_attribute_value(
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
            
            fn to_attribute_value(self) -> AttributeValue {
                attribute_value!(
                    ns, self.iter().cloned()
                        .map(|item| item.to_string()).collect()
                )
            }
        }
    }

}


/// The DynamoDBItem trait gathers all the requirements expected from a struct
/// which is meant to interact with DynamoDB operations.
pub trait DynamoDBItem
    : TryFrom<HashMap<String, AttributeValue>, Error = ConversionError>
    + Into<HashMap<String, AttributeValue>> {
    fn get_attribute_names() -> Vec<String>;
}


/// The DynamoDBInsertable trait should implement the additional expectations
/// for structs which are meant to be stored in dynamodb.
pub trait DynamoDBInsertable: DynamoDBItem {
    fn get_key(&self) -> HashMap<String, AttributeValue>;
}

pub trait AttributeValueConverter: Sized {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self>;
    fn to_attribute_value(self) -> AttributeValue;
}

impl AttributeValueConverter for String {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value.s.ok_or(ConversionError::MissingValue)
    }

    fn to_attribute_value(self) -> AttributeValue {
        macros::attribute_value!(s, self)
    }
}

impl AttributeValueConverter for HashSet<String> {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value
            .ss
            .ok_or(ConversionError::MissingValue)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }

    fn to_attribute_value(mut self) -> AttributeValue {
        macros::attribute_value!(ss, self.drain().collect())
    }
}

impl AttributeValueConverter for HashSet<Vec<u8>> {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value
            .bs
            .ok_or(ConversionError::MissingValue)
            .and_then(|mut vec| Ok(vec.drain(..).collect()))
    }

    fn to_attribute_value(mut self) -> AttributeValue {
        macros::attribute_value!(bs, self.drain().collect())
    }
}

impl AttributeValueConverter for Vec<u8> {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value.b.ok_or(ConversionError::MissingValue)
    }

    fn to_attribute_value(self) -> AttributeValue {
        macros::attribute_value!(b, self)
    }
}

impl AttributeValueConverter for bool {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value.bool.ok_or(ConversionError::MissingValue)
    }

    fn to_attribute_value(self) -> AttributeValue {
        macros::attribute_value!(bool, self)
    }
}

impl<T: DynamoDBItem> AttributeValueConverter for T {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        attribute_value
            .m
            .ok_or(ConversionError::MissingValue)
            .and_then(|attribute_map| T::try_from(attribute_map))
    }

    fn to_attribute_value(self) -> AttributeValue {
        macros::attribute_value!(m, self.into())
    }
}

impl<T: DynamoDBItem> AttributeValueConverter for Vec<T> {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        let mut convertable_vec = attribute_value.l.ok_or(ConversionError::MissingValue)?;
        let results = convertable_vec
            .drain(..)
            .map(|convertable| {
                AttributeValueConverter::from_attribute_value(convertable)
            })
            .collect();
        results
    }

    fn to_attribute_value(mut self) -> AttributeValue {
        macros::attribute_value!(
            l,
            self.drain(..)
                .map(|item| macros::attribute_value!(m, item.into()))
                .collect()
        )
    }
}

impl<T: AttributeValueConverter> AttributeValueConverter for Option<T> {
    fn from_attribute_value(attribute_value: AttributeValue) -> ConversionResult<Self> {
        match AttributeValueConverter::from_attribute_value(attribute_value) {
            Ok(value) => Ok(Some(value)),
            Err(ConversionError::MissingValue) |
            Err(ConversionError::MissingField) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn to_attribute_value(self) -> AttributeValue {
        match self {
            Some(value) => value.to_attribute_value(),
            None => AttributeValue::default(),
        }
    }
}

macros::numeric_converter!(u16);
macros::numeric_converter!(u32);
macros::numeric_converter!(i32);
macros::numeric_converter!(i64);
macros::numeric_converter!(f32);
macros::numeric_converter!(f64);

macros::numeric_set_converter!(u16 => HashSet<u16>);
macros::numeric_set_converter!(u16 => Vec<u16>);
macros::numeric_set_converter!(u32 => HashSet<u32>);
macros::numeric_set_converter!(u32 => Vec<u32>);
macros::numeric_set_converter!(i32 => HashSet<i32>);
macros::numeric_set_converter!(i32 => Vec<i32>);
macros::numeric_set_converter!(i64 => HashSet<i64>);
macros::numeric_set_converter!(i64 => Vec<i64>);
macros::numeric_set_converter!(f32 => Vec<f32>);
macros::numeric_set_converter!(f64 => Vec<f64>);


#[cfg(test)]
mod test {
    use std::default::Default;
    use std::convert::TryFrom;
    use std::collections::{HashSet, HashMap};

    use rusoto_dynamodb::AttributeValue;

    use errors::ConversionError;
    use super::{AttributeValueConverter, DynamoDBItem};


    macro_rules! test_for_numeric_types {
        ($(
            $mod_name:ident($type:ty, $valid:expr, [$($collection:ty),*])
        ),*) => {
            $(mod $mod_name {

                #[test]
                fn fails_on_missing_value() {
                    let default = ::AttributeValue::default();
                    let result: Result<$type, _> = ::AttributeValueConverter
                        ::from_attribute_value(default);
                    assert!(result.is_err());
                }

                #[test]
                fn can_convert_from_valid() {
                    let av = ::AttributeValue {
                        n: Some(format!("{}", $valid)),
                        .. ::AttributeValue::default()
                    };

                    let converted: $type = ::AttributeValueConverter
                        ::from_attribute_value(av)
                        .unwrap();
                    assert_eq!($valid, converted);
                }

                #[test]
                fn can_convert_into_attribute_value() {
                    let converted = ::AttributeValueConverter
                        ::to_attribute_value($valid);
                    assert_eq!(format!("{}", $valid), converted.n.unwrap());
                }
                
                #[test]
                fn can_convert_from_attribute_value_set() {
                    let val_1 = $valid;
                    let val_2 = $valid * $valid;
                    $(
                        let av = ::AttributeValue {
                            ns: Some(vec![
                                format!("{}", val_1), format!("{}", val_2)
                            ]),
                            .. ::AttributeValue::default()
                        };

                        let converted: $collection = ::AttributeValueConverter
                            ::from_attribute_value(av).unwrap();
                        assert!(converted.contains(&val_1));
                        assert!(converted.contains(&val_2));
                    )*
                }

                #[test]
                fn fails_on_incompatible_number_type() {
                    let av = ::AttributeValue {
                        n: Some(String::from("non-numeric")),
                        .. ::AttributeValue::default()
                    };
                    
                    let res: Result<$type, _> = ::AttributeValueConverter
                        ::from_attribute_value(av);
                    assert!(res.is_err());
                }
            })*
        }
    }

    test_for_numeric_types![
        u16_tests(u16, 123, [super::HashSet<u16>, Vec<u16>]),
        u32_tests(u32, 1234, [super::HashSet<u32>, Vec<u32>]),
        i32_tests(i32, 1234, [super::HashSet<i32>, Vec<i32>]),
        i64_tests(i64, 1234, [super::HashSet<i64>, Vec<i64>]),
        f32_tests(f32, 123.4, [Vec<f32>]),
        f64_tests(f64, 123.4, [Vec<f64>])
    ];

    #[test]
    fn can_convert_from_string() {
        let av = AttributeValue {
            s: Some(String::from("value")),
            ..AttributeValue::default()
        };

        let converted: String = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!("value", &converted);
    }

    #[test]
    fn can_convert_string_into_attribute_value() {
        let value = String::from("value");
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        assert_eq!(value, converted.s.unwrap());
    }

    #[test]
    fn can_convert_from_attribute_value_string_set() {
        let input = vec!["one".to_string(), "two".to_string()];
        let expected_set: HashSet<String> = input.iter().cloned().collect();
        let av = AttributeValue {
            ss: Some(input),
            ..AttributeValue::default()
        };

        let converted: HashSet<String> = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(expected_set, converted);
    }

    #[test]
    fn can_convert_string_set_into_attribute_value() {
        let mut value = HashSet::new();
        value.insert(String::from("value"));
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        let retrieved = converted.ss.unwrap();
        for val in value {
            assert!(retrieved.contains(&val))
        }
    }

    #[test]
    fn can_convert_from_binary() {
        let input = vec![1, 2, 3, 4];
        let expected = vec![1, 2, 3, 4];
        let av = AttributeValue {
            b: Some(input),
            ..AttributeValue::default()
        };

        let converted: Vec<u8> = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(expected, converted);
    }

    #[test]
    fn can_convert_binary_into_attribute_value() {
        let value: Vec<u8> = vec![1, 2, 3, 4];
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        assert_eq!(value, converted.b.unwrap());
    }

    #[test]
    fn can_convert_from_binary_set() {
        let input = vec![
            "one".to_string().into_bytes(), "two".to_string().into_bytes()
        ];
        let expected_set: HashSet<Vec<u8>> = input.iter().cloned().collect();
        let av = AttributeValue {
            bs: Some(input),
            ..AttributeValue::default()
        };

        let converted: HashSet<Vec<u8>> = AttributeValueConverter::from_attribute_value(av)
            .unwrap();
        assert_eq!(expected_set, converted);
    }

    #[test]
    fn can_convert_binary_set_into_attribute_value() {
        let mut value = HashSet::new();
        value.insert("vec".to_string().into_bytes());
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        let retrieved = converted.bs.unwrap();
        for val in value {
            assert!(retrieved.contains(&val))
        }
    }

    #[test]
    fn can_convert_from_bool() {
        let av = AttributeValue {
            bool: Some(true),
            ..AttributeValue::default()
        };

        let converted: bool = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(true, converted);
    }

    #[test]
    fn can_convert_bool_into_attribute_value() {
        let value = true;
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        assert_eq!(value, converted.bool.unwrap());
    }

    #[derive(PartialEq, Debug, Clone)]
    struct Example {
        key: i32,
    }

    impl DynamoDBItem for Example {
        fn get_attribute_names() -> Vec<String> {
            vec!["key"].iter().map(|s| s.to_string()).collect()
        }
    }

    impl TryFrom<HashMap<String, AttributeValue>> for Example {
        type Error = ConversionError;

        fn try_from(
            mut attribute_map: HashMap<String, AttributeValue>,
        ) -> Result<Self, Self::Error> {
            Ok(Example {
                key: attribute_map
                    .remove("key")
                    .unwrap()
                    .n
                    .unwrap()
                    .parse()
                    .unwrap(),
            })
        }
    }

    impl From<Example> for HashMap<String, AttributeValue> {
        fn from(example: Example) -> HashMap<String, AttributeValue> {
            let mut attribute_map = HashMap::new();
            attribute_map.insert(
                "key".to_string(),
                AttributeValue {
                    n: Some(example.key.to_string()),
                    ..AttributeValue::default()
                },
            );
            attribute_map
        }
    }

    #[test]
    fn can_convert_from_dynamodb_item() {

        let mut attribute_map = HashMap::new();
        let value = AttributeValue {
            n: Some(123.to_string()),
            ..AttributeValue::default()
        };

        attribute_map.insert("key".to_string(), value);

        let av = AttributeValue {
            m: Some(attribute_map),
            ..AttributeValue::default()
        };

        let converted: Example = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(Example { key: 123 }, converted);
    }

    #[test]
    fn can_convert_dynamodb_item_into_attribute_value() {
        let value = Example { key: 123 };

        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());

        let retrieved = converted.m.unwrap();
        let key = retrieved.get("key").unwrap();
        assert_eq!(&value.key.to_string(), &key.clone().n.unwrap());
    }

    #[test]
    fn can_convert_from_dynamodb_item_list() {
        let mut attribute_map = HashMap::new();
        let value = AttributeValue {
            n: Some(123.to_string()),
            ..AttributeValue::default()
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
            ..AttributeValue::default()
        };

        let converted: Vec<Example> = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(vec![Example { key: 123 }, Example { key: 123 }], converted);
    }


    #[test]
    fn can_convert_dynamodb_item_list_into_attribute_value() {
        let value = vec![
            Example {
                key: 123
            },
            Example {
                key: 124
            }
        ];

        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());

        let retrieved = converted.l.unwrap();
        for (index, item) in retrieved.iter().cloned().enumerate() {
            let map = item.m.unwrap();
            let key = map.get("key").unwrap();
            assert_eq!(&value[index].key.to_string(), &key.clone().n.unwrap());
        }
    }

    #[test]
    fn can_convert_from_option_none() {
        let av = AttributeValue { ..AttributeValue::default() };

        let converted: Option<bool> = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert!(converted.is_none());
    }

    #[test]
    fn can_convert_from_option_with_value() {
        let av = AttributeValue {
            bool: Some(true),
            ..AttributeValue::default()
        };

        let converted: Option<bool> = AttributeValueConverter::from_attribute_value(av).unwrap();
        assert_eq!(Some(true), converted);
    }

    #[test]
    fn can_convert_option_none_into_attribute_value() {
        let value: Option<bool> = None;
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        assert!(converted.bool.is_none());
    }

    #[test]
    fn can_convert_option_with_value_into_attribute_value() {
        let value: Option<bool> = Some(true);
        let converted = ::AttributeValueConverter::to_attribute_value(value.clone());
        assert_eq!(value.unwrap(), converted.bool.unwrap());
    }
}
