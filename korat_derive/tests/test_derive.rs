#![feature(try_from)]

#[macro_use] extern crate korat_derive;
extern crate korat;

extern crate rusoto_dynamodb;


use std::collections::HashSet;


#[derive(DynamoDBItem, PartialEq, Debug, Clone)]
struct SingleFieldItem {
    number_attribute: i32
}


#[derive(DynamoDBItem, PartialEq, Debug)]
struct ItemWithAllTypes {
    number_attribute: i32,
    string_attribute: String,
    string_set_attribute: HashSet<String>,
    number_set_attribute: HashSet<i32>,
    binary_set_attribute: HashSet<Vec<u8>>,
    boolean_attribute: bool,
    binary_attribute: Vec<u8>,
    item_attribute: SingleFieldItem,
    item_list_attribute: Vec<SingleFieldItem>
}


#[cfg(test)]
mod tests {

    use std::collections::HashSet;
    use std::convert::TryFrom;
    use std::default::Default;

    use rusoto_dynamodb::AttributeMap;
    use rusoto_dynamodb::AttributeValue;

    use korat::errors::ConversionError;

    use super::{ItemWithAllTypes, SingleFieldItem};
    
    macro_rules! insert {
        ($attrs:ident, $name:expr, $field:ident, $value:expr) => {
            $attrs.insert(String::from($name), AttributeValue {
                $field: Some($value),
                .. AttributeValue::default()
            });
        }
    }

    #[test]
    fn can_deserialize_valid_input() {
        let mut attributes = AttributeMap::new();

        let number_value = 1231;
        let string_value = String::from("string");

        let mut string_set_value = HashSet::new();
        string_set_value.insert(String::from("string"));

        let mut number_set_value = HashSet::new();
        number_set_value.insert(1);
        number_set_value.insert(2);
        number_set_value.insert(3);

        let mut binary_set_value = HashSet::new();
        binary_set_value.insert(vec![12, 11, 28]);

        let boolean_value = true;
        let binary_value = vec![12, 44, 28, 11];
        let item_value = SingleFieldItem {
            number_attribute: number_value
        };

        let mut item_attributes = AttributeMap::new();
        insert!(item_attributes, "number_attribute", n,
                number_value.to_string());

        let item_list_value = vec![item_value.clone(), item_value.clone()];
        let mut item_list_attributes = vec![
            AttributeValue {
                m: Some(item_attributes.clone()),
                .. AttributeValue::default()
            },
            AttributeValue {
                m: Some(item_attributes.clone()),
                .. AttributeValue::default()
            }
        ];

        insert!(attributes, "number_attribute", n, number_value.to_string());
        insert!(attributes, "string_attribute", s, string_value.clone());

        insert!(attributes, "string_set_attribute", ss,
                string_set_value.iter().cloned().collect());

        insert!(attributes, "number_set_attribute", ns,
                number_set_value.iter().map(|v| v.to_string()).collect());

        insert!(attributes, "binary_set_attribute", bs,
                binary_set_value.iter().cloned().collect());

        insert!(attributes, "boolean_attribute", bool, boolean_value);
        insert!(attributes, "binary_attribute", b, binary_value.clone());
        insert!(attributes, "item_attribute", m, item_attributes);
        insert!(attributes, "item_list_attribute", l, item_list_attributes);

        let item = ItemWithAllTypes::try_from(attributes).unwrap();

        assert_eq!(ItemWithAllTypes {
            number_attribute: number_value,
            string_attribute: string_value,
            string_set_attribute: string_set_value,
            number_set_attribute: number_set_value,
            binary_set_attribute: binary_set_value,
            boolean_attribute: boolean_value,
            binary_attribute: binary_value,
            item_attribute: item_value,
            item_list_attribute: item_list_value
        }, item)
    }

    #[test]
    fn fails_to_deserialize_invalid_input() {
        let mut attributes = AttributeMap::new();
        let res = ItemWithAllTypes::try_from(attributes);
        assert!(res.is_err())
    }

    #[test]
    fn can_deserialize_serialized() {
        unimplemented!()
    }
}
