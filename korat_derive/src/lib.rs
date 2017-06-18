//! Korat provides rusoto implementations for using an structs as dynamodb items
use rusoto_dynamodb::AttributeMap;

use korat::errors::KoratError;
use korat::DynamoDBItem;


mod dynamodb_item;

use dynamodb_item::expand;


#[proc_macro_derive(DynamoDBItem)]
pub fn dynamodb_item(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = expand(&ast);
    gen.parse().unwrap()
}
