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

    let to_attribute_map = get_from_attribute_map_trait(name, fields);
    let from_attribute_map = get_to_attribute_map_trait(name, fields);
    let struct_implementation = get_struct_implementation(name, fields);
    
    quote! {
        #to_attribute_map
        #from_attribute_map
        #struct_implementation
    }
}

fn get_to_attribute_map_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);
    let from = quote!(::std::from::From);
    let to_attribute_map = get_to_attribute_map_function(fields);
    
    quote! {
        impl #from<#name> for #attribute_map {
            #to_attribute_map
        }
    }
}

fn get_to_attribute_map_function(name: &Ident, fields: &[Field]) -> Tokens {

    quote! {
        fn from(item: #name) -> Self {
            let attribute_map = Self::new();
            attribute_map
        }
    }
}

fn get_from_attribute_map_trait(name: &Ident, fields: &[Field]) -> Token {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);
    let korat_error = quote!(::korat::errors::KoratError); 
    let try_from = quote!(::std::from::TryFrom);
    let from_attribute_map = get_from_attribute_map_function(name, fields);

    quote! {
        impl #try_from<#attribute_map> for #name {
            type Error = #korat_error;
            #from_attribute_map
        }
    }
}

fn get_from_attribute_map_function(
    name: &Ident, fields: &[Field]
) -> Tokens {
    let attribute_map = quote!(::rusoto_dynamodb::AttributeMap);

    let field_conversions = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_accessor = get_field_accessor(field);
        quote! {
            #field_name: #field_accessor
        }
    });

    quote! {
        fn try_from(item: #attribute_map) -> Result<Self, Self::Error> {
            Self {
                #(#field_conversions),*
            }
        }
    }
}

fn get_field_accessor(field: &Field) -> Tokens {
    let field_name = &field.ident;
    match field.ty {
    }
    
    quote! {
        item.get(stringify!(#field_name))
    }
}


fn get_struct_implementation(name: &Ident, fields: &[Field]) -> Tokens {
    quote! {
        impl #name {
        }
    }
}
