use quote::Tokens;
use syn::{Ident, Field, MacroInput};
use syn::Body::Struct;
use syn::VariantData::Struct as StructData;


pub fn expand(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;
    match ast.body {
        Struct(StructData(ref fields)) => make_dynamodb_item(name, fields),
        _ => panic!("DynamoDB Items can only be generated for structs")
    }
}

fn make_dynamodb_item(name: &Ident, fields: &[Field]) -> Tokens {

    let dynamodb_traits = get_dynamodb_traits(name, fields);
    let to_attribute_map = get_from_attribute_map_trait(name, fields);
    let from_attribute_map = get_to_attribute_map_trait(name, fields);

    quote! {
        #from_attribute_map
        #to_attribute_map
        #dynamodb_traits
    }
}

fn get_to_attribute_map_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);
    let from = quote!(::std::convert::From);
    let to_attribute_map = get_to_attribute_map_function(name, fields);

    quote! {
        impl #from<#name> for #attribute_map {
            #to_attribute_map
        }
    }
}

fn get_to_attribute_map_function(name: &Ident, fields: &[Field]) -> Tokens {
    let to_attribute_value = quote!(
        ::korat::AttributeValueConverter::to_attribute_value
    );

    let field_conversions = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            values.insert(
                stringify!(#field_name).to_string(),
                #to_attribute_value(item.#field_name)
            );
        }
    });

    quote! {
        fn from(item: #name) -> Self {
            let mut values = Self::new();
            #(#field_conversions)*
            values
        }
    }
}

fn get_from_attribute_map_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);
    let conversion_error = quote!(::korat::errors::ConversionError); 
    let try_from = quote!(::std::convert::TryFrom);
    let from_attribute_map = get_from_attribute_map_function(fields);

    quote! {
        impl #try_from<#attribute_map> for #name {
            type Error = #conversion_error;
            #from_attribute_map
        }
    }
}

fn get_from_attribute_map_function(fields: &[Field]) -> Tokens {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);
    let from_attribute_value = quote!(
        ::korat::AttributeValueConverter::from_attribute_value
    );
    let conversion_error = quote!(::korat::errors::ConversionError); 

    let field_conversions = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: #from_attribute_value(
                item.remove(stringify!(#field_name))
                    .ok_or(#conversion_error::MissingField)?
            )?
        }
    });

    quote! {
        fn try_from(mut item: #attribute_map) -> Result<Self, Self::Error> {
            Ok(Self {
                #(#field_conversions),*
            })
        }
    }
}

fn get_dynamodb_traits(name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_item_trait = get_dynamodb_item_trait(name, fields);
    let dynamodb_insertables = get_dynamodb_insertables(name, fields);

    quote! {
        #dynamodb_item_trait
        #dynamodb_insertables
    }
}

fn get_dynamodb_item_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_item = quote!(::korat::DynamoDBItem);
    let attribute_name = quote!(::rusoto_dynamodb::AttributeName);
    let field_names: Vec<String> = fields.iter().cloned()
        .map(|f| f.ident.expect("DynamoDBItem fields should have identifiers")
             .to_string())
        .collect();

    quote!{
        impl #dynamodb_item for #name {
            fn get_attribute_names() -> Vec<#attribute_name> {
                vec![#(String::from(#field_names)),*]
            }
        }
    }
}

fn get_dynamodb_insertables(name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_insertable_trait = get_dynamodb_insertable_trait(name, fields);
    let dynamodb_key_struct = get_dynamodb_key_struct(name, fields);

    quote! {
        #dynamodb_insertable_trait
        #dynamodb_key_struct
    }
}

fn get_dynamodb_insertable_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_insertable = quote!(::korat::DynamoDBInsertable);
    let key = quote!(::rusoto_dynamodb::Key);
    let hash_key_name = get_field_name_with_attribute(&fields, "hash");
    let range_key_name = get_field_name_with_attribute(&fields, "range");

    let hash_key_inserter = get_key_inserter(&hash_key_name);
    let range_key_inserter = get_key_inserter(&range_key_name);

    hash_key_name.map(|_| quote!{
        impl #dynamodb_insertable for #name {
            fn get_key(&self) -> #key {
                let mut keys = #key::new();
                #hash_key_inserter
                #range_key_inserter
                keys
            }
        }
    }).unwrap_or(quote!{})
}

fn get_field_name_with_attribute(
    fields: &[Field], attribute_name: &str
) -> Option<Ident> {
    get_field_with_attribute(fields, attribute_name)
        .map(|field| field.ident.expect(
            &format!("{} should have an identifier", attribute_name)
        ))
}

fn get_field_with_attribute(
    fields: &[Field], attribute_name: &str
) -> Option<Field> {
    let mut fields = fields.iter().cloned().filter(
        |field| field.attrs.iter().any(|attr| attr.name() == attribute_name)
    );

    let field = fields.next();
    if let Some(_) = fields.next() {
        panic!("Can't set more than one {} key", attribute_name);
    }
    field
}

fn get_key_inserter(field_name: &Option<Ident>) -> Tokens {    
    let to_attribute_value = quote!(
        ::korat::AttributeValueConverter::to_attribute_value
    );
    field_name.as_ref().map(|field_name| quote!{
        keys.insert(
            stringify!(#field_name).to_string(),
            #to_attribute_value(self.#field_name.clone())
        );
    }).unwrap_or(quote!())
}

fn get_dynamodb_key_struct(name: &Ident, fields: &[Field]) -> Tokens {
    let name = Ident::from(format!("{}Key", name));

    let hash_key = get_field_with_attribute(&fields, "hash");
    let range_key = get_field_with_attribute(&fields, "range")
        .map(|mut range_key| {
            range_key.attrs = vec![];
            quote! {#range_key}
        }).unwrap_or(quote!());

    hash_key.map(|mut hash_key| {
        hash_key.attrs = vec![];
        quote!{
            #[derive(DynamoDBItem, Debug, Clone, PartialEq)]
            struct #name {
                #hash_key,
                #range_key
            }
        }}).unwrap_or(quote!())
}
