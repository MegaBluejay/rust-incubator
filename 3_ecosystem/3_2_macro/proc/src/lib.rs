use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Token,
};

struct KeyVal {
    key: Expr,
    val: Expr,
}

impl Parse for KeyVal {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let val = input.parse()?;

        Ok(Self { key, val })
    }
}

struct BTreeMapInput {
    pairs: Punctuated<KeyVal, Token![,]>,
}

impl Parse for BTreeMapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pairs: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro]
pub fn btreemap(input: TokenStream) -> TokenStream {
    let BTreeMapInput { pairs } = parse_macro_input!(input as BTreeMapInput);

    let inserts = pairs.into_iter().map(|KeyVal { key, val }| {
        quote! {
            __map.insert(#key, #val);
        }
    });

    let expanded = quote! {
        {
            #[allow(unused_mut)]
            let mut __map = ::std::collections::BTreeMap::new();
            #(#inserts)*
            __map
        }
    };

    expanded.into()
}
