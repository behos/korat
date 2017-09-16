#![feature(try_from)]

#[macro_use] extern crate korat_derive;
extern crate korat;

extern crate rusoto_dynamodb;


use std::collections::HashSet;


#[derive(DynamoDBItem, PartialEq, Debug, Clone)]
struct SingleFieldItem {
    number_attribute: i32
}


#[derive(DynamoDBItem, PartialEq, Debug, Clone)]
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


#[derive(DynamoDBItem, PartialEq, Debug, Clone)]
struct ItemWithHashAndRange {
    #[hash] hash: String,
    #[range] range: String,
    other: String
}


mod internal {
    #[derive(DynamoDBItem, PartialEq, Debug, Clone)]
    pub struct InternalItem {
        #[hash] hash: String,
        #[range] range: String,
        other: String
    }
}


#[cfg(test)]
mod tests {

    use std::collections::{HashSet, HashMap};
    use std::convert::TryFrom;
    use std::default::Default;

    use rusoto_dynamodb::AttributeValue;

    use korat::{DynamoDBInsertable, DynamoDBItem};

    use super::{
        ItemWithAllTypes, SingleFieldItem,
        ItemWithHashAndRange, ItemWithHashAndRangeKey
    };

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
        let mut attributes = HashMap::new();

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

        let mut item_attributes = HashMap::new();
        insert!(item_attributes, "number_attribute", n,
                number_value.to_string());

        let item_list_value = vec![item_value.clone(), item_value.clone()];
        let item_list_attributes = vec![
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
        let attributes = HashMap::new();
        let res = ItemWithAllTypes::try_from(attributes);
        assert!(res.is_err())
    }

    #[test]
    fn can_deserialize_serialized() {
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

        let item_list_value = vec![item_value.clone(), item_value.clone()];

        let item = ItemWithAllTypes {
            number_attribute: number_value,
            string_attribute: string_value,
            string_set_attribute: string_set_value,
            number_set_attribute: number_set_value,
            binary_set_attribute: binary_set_value,
            boolean_attribute: boolean_value,
            binary_attribute: binary_value,
            item_attribute: item_value,
            item_list_attribute: item_list_value
        };

        let serialized: HashMap<String, AttributeValue> = item.clone().into();
        let deserialized = ItemWithAllTypes::try_from(serialized).unwrap();

        assert_eq!(item, deserialized);
    }

    #[test]
    fn can_get_hash_key_from_insertable() {
        let item = ItemWithHashAndRange {
            hash: "hash value".to_string(),
            range: "range value".to_string(),
            other: "other value".to_string()
        };

        let key = item.get_key();
        assert_eq!("hash value", &key["hash"].clone().s.unwrap());
        assert_eq!("range value", &key["range"].clone().s.unwrap())
    }

    #[test]
    fn can_create_key_structure_from_insertable() {
        let key = ItemWithHashAndRangeKey {
            hash: "something".to_string(),
            range: "something else".to_string()
        };

        let attr_map: HashMap<String, AttributeValue> = key.clone().into();
        let deserialized = ItemWithHashAndRangeKey::try_from(attr_map).unwrap();

        assert_eq!(key, deserialized);
        
    }

    #[test]
    fn can_get_attribute_names() {
        assert_eq!(
            vec!["number_attribute"], SingleFieldItem::get_attribute_names()
        )
    }

    #[test]
    fn parent_pub_visibility_is_tranfered_to_key() {
        #[allow(unused_imports)]
        use super::internal::{InternalItem, InternalItemKey};
    }
}
