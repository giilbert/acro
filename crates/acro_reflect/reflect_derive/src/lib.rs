use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, token::Struct, Attribute, Data, DataStruct, DeriveInput, Ident, Meta,
};

#[proc_macro_derive(Reflect, attributes(derive))]
pub fn reflect_derive(input: TokenStream) -> TokenStream {
    reflect_derive_impl(input)
}

fn reflect_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(struct_data) => reflect_derive_struct(input.ident, input.attrs, struct_data),
        _ => todo!(),
    }
}

fn reflect_derive_struct(
    name: Ident,
    attrs: Vec<Attribute>,
    struct_data: DataStruct,
) -> TokenStream {
    let field_idents = struct_data
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| match &attr.meta {
                Meta::List(list) => {
                    if list.path.is_ident("reflect") {
                        let token = match list.tokens.clone().into_iter().next().unwrap() {
                            proc_macro2::TokenTree::Ident(ident) => ident.to_string(),
                            _ => panic!("expected ident"),
                        };

                        panic!("token: {}", token);

                        token != "skip"
                    } else {
                        true
                    }
                }
                _ => true,
            })
        })
        .map(|field| field.ident.as_ref().expect("fields should have names"));
    let field_names_string = field_idents.clone().map(|name| name.to_string());

    let field_idents_2 = field_idents.clone();
    let field_names_string_2 = field_names_string.clone();
    let field_idents_3 = field_idents.clone();
    let field_names_string_3 = field_names_string.clone();

    quote! {
        impl acro_reflect::Reflect for #name {
            fn get_field_names(&self) -> &'static [&'static str] {
                &[
                    #( #field_names_string ),*
                ]
            }

            fn get_opt(&self, path: &acro_reflect::ReflectPath) -> Option<&dyn std::any::Any> {
                match path {
                    #(
                        acro_reflect::ReflectPath::Property(#field_names_string_2, rest)
                            => self.#field_idents_2.get_opt(rest),
                    )*
                    _ => None,
                }
            }

            fn set_any(
                &mut self,
                path: &acro_reflect::ReflectPath,
                data: Box<dyn std::any::Any>
            ) -> Result<(), acro_reflect::ReflectSetError> {
                match path {
                    #(
                        acro_reflect::ReflectPath::Property(#field_names_string_3, rest)
                            => self.#field_idents_3.set_any(rest, data),
                    )*
                    _ => Err(acro_reflect::ReflectSetError::PathNotFound),
                }
            }
        }
    }
    .into()
}
