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
            // TODO use https://doc.rust-lang.org/reference/attributes.html to additionally switch if this happens
            let all_arms_unit = data.variants.iter().all(|v| match &v.fields {
                syn::Fields::Unit => true,
                _ => false,
            });

            let arms = data.variants.iter().map(|v| {
                let vname = &v.ident;
                let vname_as_string = vname.to_string();
                match &v.fields {
                    syn::Fields::Unnamed(fields) => {
                        let unnamed_types: Vec<_> = fields.unnamed
                            .iter()
                            .map(|field| &field.ty)
                            .collect();

                        if unnamed_types.len() != 1 {
                            panic!("For enum type conversion, only exaclty one unnamed enum entry is allowed!")
                        } // TODO support these also

                        let unnamed_type = &unnamed_types[0];

                        (
                            quote! {
                                format!("{}{}", <Self as TypescriptSerializable>::type_name(), #vname_as_string)
                            },
                            quote! {
                                format!("export type {}{} = {{ type: \"{}\"; value: {} }};\n", <Self as TypescriptSerializable>::type_name(), #vname_as_string, #vname_as_string, <#unnamed_type as TypescriptSerializable>::type_name())
                            },
                            [unnamed_types[0]].into()
                        )
                    }
                    syn::Fields::Named(fields) => {
                        let intermediate = fields
                            .named
                            .iter()
                            .map(|f| (f.ident.as_ref().unwrap(), &f.ty));
                        let lines: Vec<_> = intermediate.clone()
                            .map(|(i,t)| {
                                    let i_as_str = i.to_string();
                                    quote! {
                                        format!("        {}: {}", #i_as_str, <#t as TypescriptSerializable>::type_name())
                                    }
                                }
                            )
                            .collect();

                        let format_string = format!(
                            "{{{{\n{}    }}}}\n",
                            std::iter::repeat("{};\n")
                                .take(lines.len())
                                .fold(String::new(), |a, b| a + b)
                        );

                        (
                            quote! {
                                format!("{}{}", <Self as TypescriptSerializable>::type_name(), #vname_as_string)
                            },
                            quote! {
                                format!("export type {}{} = {{ type: \"{}\"; value: {} }};\n", <Self as TypescriptSerializable>::type_name(), #vname_as_string, #vname_as_string, format!(#format_string, #(#lines),*))
                            },
                            intermediate.map(|(_,t)| t).collect()
                        )
                    }
                    syn::Fields::Unit => {
                        if all_arms_unit {
                            (
                                quote! {
                                    format!("    {} = \"{}\",\n", #vname_as_string, #vname_as_string)
                                },
                                quote! {
                                    format!("") // ignored in this case not needed
                                },
                                Vec::new()
                            )
                        } else {
                            (
                                quote! {
                                    format!("{}{}", <Self as TypescriptSerializable>::type_name(), #vname_as_string)
                                },
                                quote! {
                                    format!("export type {}{} = {{ type: \"{}\" }};\n", <Self as TypescriptSerializable>::type_name(), #vname_as_string, #vname_as_string)
                                },
                                Vec::new()
                            )
                        }
                    }
                }
            });

            if all_arms_unit {
                let format_string = format!(
                    "{{{{\n{}}}}}",
                    std::iter::repeat("{}")
                        .take(arms.len())
                        .fold(String::new(), |a, b| a + b)
                );
                let arms_values = arms.map(|(v, _, _)| v);

                (
                    quote! {
                        format!(#format_string, #(#arms_values),*)
                    },
                    quote! {
                        let mut collector: Vec<String> = Vec::new();

                        collector.push(format!("export enum {} {}\n", <Self as TypescriptSerializable>::type_name(), <Self as TypescriptSerializable>::serialize_to_type()));

                        collector
                    },
                )
            } else {
                let format_string = format!(
                    "{}",
                    std::iter::repeat("\n    | {}")
                        .take(arms.len())
                        .fold(String::new(), |a, b| a + b)
                );
                let arms_names = arms.clone().map(|(n, _, _)| n);
                let arms_values = arms.clone().map(|(_, a, _)| a).map(|a| {
                    quote! {
                        collector.push(#a);
                    }
                });
                let arms_types = arms.flat_map(|(_, _, t)| t).map(|ty| {
                    quote! {
                        collector.append(&mut <#ty as TypescriptSerializable>::all_types_output());
                    }
                });

                (
                    quote! {
                        format!(#format_string, #(#arms_names),*)
                    },
                    quote! {
                        let mut collector: Vec<String> = Vec::new();

                        #(#arms_values)*

                        collector.push(format!("export type {} ={};\n", <Self as TypescriptSerializable>::type_name(), <Self as TypescriptSerializable>::serialize_to_type()));

                        #(#arms_types)*

                        collector
                    },
                )
            }
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
