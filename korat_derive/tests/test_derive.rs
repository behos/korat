#![feature(try_from)]

#[macro_use] extern crate korat_derive;
extern crate korat;

extern crate rusoto_dynamodb;


use std::collections::HashSet;


#[derive(DynamoDBItem)]
struct ItemWithAllTypes {
    id: i32,
    number_attribute: i32,
    string_attribute: String,
    string_set_attribute: HashSet<String>,
    number_set_attribute: HashSet<i32>,
    binary_set_attribute: HashSet<Vec<u8>>,
    boolean_attribute: bool,
    byte_attribute: Vec<u8>
}


#[derive(DynamoDBItem)]
struct ItemWithHashAndSort {
    hash: i32,
    sort: String,
}


#[cfg(test)]
mod tests {

    use std::collections::HashSet;
    use rusoto_dynamodb::AttributeMap;

    use korat::errors::ConversionError;
    
    #[test]
    fn can_deserialize_valid_input() {
    }

    #[test]
    fn fails_to_deserialize_invalid_input() {
    }

    #[test]
    fn can_deserialize_serialized() {
    }
}
