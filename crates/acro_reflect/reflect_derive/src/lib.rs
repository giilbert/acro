use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, token::Struct, Data, DataStruct, DeriveInput, Ident};

#[proc_macro_derive(Reflect)]
pub fn reflect_derive(input: TokenStream) -> TokenStream {
    reflect_derive_impl(input)
}

fn reflect_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(struct_data) => reflect_derive_struct(input.ident, struct_data),
        _ => todo!(),
    }
}

fn reflect_derive_struct(name: Ident, struct_data: DataStruct) -> TokenStream {
    let field_idents = struct_data
        .fields
        .iter()
        .map(|field| field.ident.as_ref().expect("fields should have names"));
    let field_names_string = field_idents.clone().map(|name| name.to_string());

    let field_idents_2 = field_idents.clone();
    let field_names_string_2 = field_names_string.clone();
    let field_idents_3 = field_idents.clone();
    let field_names_string_3 = field_names_string.clone();

    quote! {
        impl Reflect for #name {
            fn get_field_names(&self) -> &'static [&'static str] {
                &[
                    #( #field_names_string ),*
                ]
            }

            fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any> {
                match path {
                    #(
                        ReflectPath::Property(#field_names_string_2, rest) => self.#field_idents_2.get_opt(rest),
                    )*
                    _ => None,
                }
            }

            fn set_any(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
                match path {
                    #(
                        ReflectPath::Property(#field_names_string_3, rest) => self.#field_idents_3.set_any(rest, data),
                    )*
                    _ => Err(ReflectSetError::PathNotFound),
                }
            }
        }
    }
    .into()
}