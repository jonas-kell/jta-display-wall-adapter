use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TypescriptSerializable)]
pub fn derive_typescript_serializable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (body_serialize, body_all_types) = match &input.data {
        syn::Data::Struct(data) => {
            let identifiers = data.fields.iter().map(|f| {
                let field = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                (field, ty)
            });

            let format_string = format!(
                "{{{{\n{}}}}}",
                std::iter::repeat("{}\n")
                    .take(data.fields.len())
                    .fold(String::new(), |a, b| a + b)
            );

            let calls = identifiers.clone().map(|(ident, ty)| {
                let ident_as_string = ident.to_string();
                quote! { format!("    {}: {};", #ident_as_string, <#ty as TypescriptSerializable>::type_name()) }
            });
            let all_types = identifiers.map(|(_, ty)| {
                quote! { collector.append(&mut <#ty as TypescriptSerializable>::all_types_output()); }
            });
            (
                quote! { format!(#format_string, #(#calls),*)  },
                quote! {
                    let mut collector: Vec<String> = Vec::new();

                    #(#all_types)*
                    collector.push(format!("export type {} = {};\n", <Self as TypescriptSerializable>::type_name(), <Self as TypescriptSerializable>::serialize_to_type()));

                    collector
                },
            )
        }
        syn::Data::Enum(data) => {
            let arms = data.variants.iter().map(|v| {
                let vname = &v.ident;
                match &v.fields {
                    syn::Fields::Unnamed(fields) => {
                        let bindings: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("f{i}"), v.ident.span()))
                            .collect();
                        quote! {
                            #name::#vname(#(#bindings),*) => { #(#bindings.serialize_to_type();)* }
                        }
                    }
                    syn::Fields::Named(fields) => {
                        let bindings: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();

                        quote! {
                            #name::#vname { #(#bindings),* } => {
                                #(#bindings.serialize_to_type();)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #name::#vname => {}
                        }
                    }
                }
            });
            let arms_cloned = arms.clone();
            (
                quote! {
                    match self {
                        #(#arms),*
                    }
                },
                quote! {
                    match self {
                        #(#arms_cloned),*
                    }
                },
            )
        }
        syn::Data::Union(_) => {
            panic!("TypescriptSerializable cannot be derived for unions");
        }
    };

    let name_as_string = name.to_string();
    let expanded = quote! {
        impl #impl_generics TypescriptSerializable for #name #ty_generics #where_clause {
            fn type_name() -> String {
                #name_as_string.into()
            }

            fn serialize_to_type() -> String {
                #body_serialize
            }

            fn all_types_output() -> Vec<String> {
                #body_all_types
            }
        }
    };

    expanded.into()
}
