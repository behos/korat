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

    let dynamodb_item = get_dynamodb_item_trait(name);
    let to_attribute_map = get_from_attribute_map_trait(name, fields);
    let from_attribute_map = get_to_attribute_map_trait(name, fields);
    let struct_implementation = get_struct_implementation(name);
    
    quote! {
        #from_attribute_map
        #to_attribute_map
        #struct_implementation
        #dynamodb_item
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


fn get_struct_implementation(name: &Ident) -> Tokens {
    quote! {
        impl #name {
        }
    }
}


fn get_dynamodb_item_trait(name: &Ident) -> Tokens {
    let dynamodb_trait = quote!(::korat::DynamoDBItem);
    quote! {
        impl #dynamodb_trait for #name {
        }
    }
}
