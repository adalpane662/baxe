use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, Fields};

#[proc_macro_attribute]
pub fn error(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_attrs = attr.to_string();

    let mut log_message_with = None;
    let mut hide_message = false;

    for part in input_attrs.split(',') {
        let part = part.trim();
        if part.starts_with("logMessageWith=") {
            if let Some((_, value)) = part.split_once('=') {
                log_message_with = Some(syn::parse_str::<Expr>(&value.trim().to_string()).unwrap());
            }
        } else if part == "hideMessage" {
            hide_message = true;
        }
    }

    let input = parse_macro_input!(item as DeriveInput);

    for attr in &input.attrs {
        if attr.path().is_ident("baxe") {
            let err = syn::Error::new(
                attr.span(),
                "The #[baxe(...)] attribute is only allowed on enum variants, not on the enum itself.",
            );
            return err.to_compile_error().into();
        }
    }

    let enum_name = input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("baxe::error can only be applied to enums"),
    };

    let variants_def = data
        .variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            match &v.fields {
                Fields::Unit => quote! { #variant_ident },
                Fields::Unnamed(fields) => {
                    let types = fields.unnamed.iter().map(|f| &f.ty);
                    quote! { #variant_ident(#(#types),*) }
                }
                Fields::Named(fields) => {
                    let field_defs = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote! { #name: #ty }
                    });
                    quote! { #variant_ident { #(#field_defs),* } }
                }
            }
        })
        .collect::<Vec<_>>();

    let matches = data
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let attrs = parse_baxe_attributes(variant);
            let (status, tag, code, message) = (attrs.status, attrs.tag, attrs.code, attrs.message);

            let pattern = match &variant.fields {
                Fields::Unit => quote! { 
                    #enum_name::#variant_ident
                },
                Fields::Unnamed(ref fields) => {
                    let field_patterns: Vec<_> = fields.unnamed.iter().enumerate().map(|(i, _)| {
                        syn::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site())
                    }).collect();
                    quote! {
                        #enum_name::#variant_ident(#(#field_patterns),*)
                    }
                }
                Fields::Named(ref fields) => {
                    let field_patterns: Vec<_> = fields.named.iter().map(|field| {
                        field.ident.clone().unwrap()
                    }).collect();
                    quote! {
                        #enum_name::#variant_ident { #(#field_patterns),* }
                    }
                }
            };

            let message = match &variant.fields {
                Fields::Unit => quote! { 
                    #enum_name::#variant_ident => { 
                        write!(f, #message) 
                    }
                },
                Fields::Unnamed(ref fields) => {
                    let field_patterns: Vec<_> = fields.unnamed.iter().enumerate().map(|(i, _)| {
                        syn::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site())
                    }).collect();
            
                    let field_names = &field_patterns;
                    quote! {
                        #enum_name::#variant_ident(#(#field_patterns),*) => {
                            let formatted_message = format!(#message, #(#field_names),*);
                            write!(f, "{formatted_message}")
                        }
                    }
                }
                Fields::Named(ref fields) => {
                    let field_patterns: Vec<_> = fields.named.iter().map(|field| {
                        field.ident.clone().unwrap()
                    }).collect();
            
                    let field_names = &field_patterns;
                    quote! {
                        #enum_name::#variant_ident { #(#field_patterns),* } => {
                            let formatted_message = format!(#message, #(#field_names),*);
                            write!(f, "{formatted_message}")
                        }
                    }
                }
            };

            (pattern, status, tag, code, message)
        })
        .collect::<Vec<_>>();

    let (patterns, statuses, tags, codes, messages): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
        matches.into_iter().unzip_n_vec();

    let log_statement = if let Some(log_fn) = log_message_with {
        quote! {
            #log_fn!("{}", error.to_string());
        }
    } else {
        quote! {}
    };

    let to_message = if hide_message {
        quote! {
            None
        }
    } else {
        quote! {
            error.to_string().into()
        }
    };

    let expanded = quote! {
        #[derive(Debug)]
        pub enum #enum_name {
            #(#variants_def,)*
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#messages,)*
                }
            }
        }

        impl std::error::Error for #enum_name {}
        
        impl BackendError for #enum_name {
            fn to_status_code(&self) -> axum::http::StatusCode {
                match self {
                    #(#patterns => #statuses,)*
                }
            }

            fn to_error_tag(&self) -> impl std::fmt::Display {
                match self {
                    #(#patterns => #tags,)*
                }
            }

            fn to_error_code(&self) -> u16 {
                match self {
                    #(#patterns => #codes,)*
                }
            }
        }

        impl From<#enum_name> for BaxeError {
            fn from(error: #enum_name) -> Self {
                #log_statement
                let status = error.to_status_code();
                let tag: String = error.to_error_tag().to_string();
                BaxeError::new(status, #to_message, error.to_error_code(), tag)
            }
        }

        impl IntoResponse for #enum_name {
            fn into_response(self) -> axum::response::Response {
                (self.to_status_code(), Json(BaxeError::from(self))).into_response()
            }
        }
    };

    TokenStream::from(expanded)
}

struct BaxeAttributes {
    status: proc_macro2::TokenStream,
    tag: proc_macro2::TokenStream,
    code: proc_macro2::TokenStream,
    message: proc_macro2::TokenStream,
}

fn parse_baxe_attributes(variant: &syn::Variant) -> BaxeAttributes {
    let mut attrs = BaxeAttributes {
        status: quote!(None),
        tag: quote!(None),
        code: quote!(None),
        message: quote!(None),
    };

    for attr in &variant.attrs {
        if attr.path().is_ident("baxe") {
            attr.parse_nested_meta(|meta| {
                let value = meta.value()?.parse::<Expr>()?;
                if let Some(ident) = meta.path.get_ident().map(|ident| ident.to_string()) {
                    match ident.as_str() {
                        "status" => attrs.status = quote!(#value),
                        "tag" => attrs.tag = quote!(#value),
                        "code" => attrs.code = quote!(#value),
                        "message" => attrs.message = quote!(#value),
                        _ => {}
                    }
                }
                Ok(())
            })
            .unwrap();
        }
    }

    attrs
}

trait UnzipN<T1, T2, T3, T4, T5> {
    fn unzip_n_vec(self) -> (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>);
}

impl<T1, T2, T3, T4, T5, I: Iterator<Item = (T1, T2, T3, T4, T5)>> UnzipN<T1, T2, T3, T4, T5>
    for I
{
    fn unzip_n_vec(self) -> (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>, Vec<T5>) {
        let mut t1 = Vec::new();
        let mut t2 = Vec::new();
        let mut t3 = Vec::new();
        let mut t4 = Vec::new();
        let mut t5 = Vec::new();

        for (x1, x2, x3, x4, x5) in self {
            t1.push(x1);
            t2.push(x2);
            t3.push(x3);
            t4.push(x4);
            t5.push(x5);
        }

        (t1, t2, t3, t4, t5)
    }
}