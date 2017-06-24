//! Korat provides rusoto implementations for using an structs as dynamodb items

#[macro_use] extern crate quote;

extern crate proc_macro;
extern crate rusoto_dynamodb;
extern crate syn;

extern crate korat;

mod dynamodb_item;

use proc_macro::TokenStream;

use dynamodb_item::expand;


#[proc_macro_derive(DynamoDBItem)]
pub fn dynamodb_item(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = expand(&ast);
    gen.parse().unwrap()
}
