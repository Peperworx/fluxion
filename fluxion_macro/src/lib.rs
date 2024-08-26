use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse::Parse, punctuated::Punctuated, token::Comma, DeriveInput, LitStr, Token, Type};


struct MessageParams {
    pub result_type: Type,
    pub name: Option<LitStr>
}

impl Parse for MessageParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse the result type
        let result_type = input.parse()?;

        // If there is a comma, parse it
        let name = if input.peek(Token![,]) {
            input.parse::<Comma>()?;

            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            result_type, name
        })
    }
}

#[proc_macro_attribute]
pub fn message(attr: TokenStream, item: TokenStream) -> TokenStream {

    // Get the parameters
    let params = if attr.is_empty() {
        MessageParams {
            result_type: Type::Tuple(syn::TypeTuple {
                paren_token: syn::token::Paren(Span::call_site()),
                elems: Punctuated::new()
            }),
            name: None,
        }
    } else {
        syn::parse_macro_input!(attr as MessageParams)
    };


    // Get the item's name
    let item_name = item.clone();
    let item_name = syn::parse_macro_input!(item_name as DeriveInput).ident;

    // Default the id to the path of the item
    // if no id is provided.
    let id: TokenStream2 = match params.name {
        Some(name) => name.to_token_stream(),
        None => {
            // Get the name of the item
            let name: TokenStream2 = format!("\"{}\"", item_name).parse().expect("this should always succeed parsing as a string");


            quote! {
                fluxion::concatcp!(module_path!(), "::", #name)
            }
        }
    };

    
    // Convert to tokenstream 2 for quote.
    let item: TokenStream2 = item.into();

    // Extract the result type
    let result_type = params.result_type;
    
    quote! {
        #item

        impl fluxion::MessageID for #item_name {
            const ID: &'static str = #id;
        }

        impl fluxion::Message for #item_name {
            type Result = #result_type;
        }
    }.into()
}


#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Get the item's name
    let item_name = item.clone();
    let item_name = syn::parse_macro_input!(item_name as DeriveInput).ident;

    // Get the optional error type, defaulting to ()
    let error_type = if attr.is_empty() {
        Type::Tuple(syn::TypeTuple {
            paren_token: syn::token::Paren(Span::call_site()),
            elems: Punctuated::new()
        })
    } else {
        syn::parse_macro_input!(attr as Type)
    };

    let item: TokenStream2 = item.into();

    quote! {
        #item

        impl fluxion::Actor for #item_name {
            type Error = #error_type;
        }
    }.into()
}