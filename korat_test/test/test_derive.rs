#[macro_use] extern crate korat_derive;
extern crate korat;

#[derive(DynamoDBItem)]
struct ItemWithAllTypes {
    #[hash] id: i32,
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
    #[hash] hash: i32,
    #[sort] sort: String,
}


#[cfg(test)]
mod tests {

    use std::collections::HashSet;
    use rusoto_dynamodb::AttributeMap;

    use errors::KoratError;
    use super::DynamoDBItem;

    impl DynamoDBItem for TestItem {
        fn deserialize(attributes: AttributeMap) -> Result<Self, KoratError> {
            Ok(TestItem {
                number_attribute: 1,
                string_attribute: String::from("hello"),
                list_attribute: vec![],
                set_attribute: HashSet::new()
            })
        }

        fn serialize(&self) -> AttributeMap {
            AttributeMap::new()
        }
    }
    
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
