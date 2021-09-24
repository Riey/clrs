extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated};

fn impl_struct(
    name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let fields = fields.iter().map(|f| f.ident.as_ref().unwrap());

    quote! {
        impl<'a> ::scroll::ctx::TryFromCtx<'a, PeCtx> for #name {
            type Error = ::scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                use ::scroll::Pread;

                let offset = &mut 0;

                let s = Self {
                    #( #fields: src.gread_with(offset, ctx)?, )*
                };

                Ok((s, *offset))
            }
        }
    }
}

fn impl_try_from_ctx(
    syn::DeriveInput {
        ident,
        attrs,
        data,
        generics,
        vis,
    }: &syn::DeriveInput,
) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(ref fields) => impl_struct(ident, &fields.named),
            syn::Fields::Unnamed(ref _fields) => todo!("Only named struct supported"),
            _ => panic!(),
        },
        _ => panic!("Only struct supported"),
    }
}

#[proc_macro_derive(ClrPread)]
pub fn derive_clr_pread(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_try_from_ctx(&syn::parse_macro_input!(input as syn::DeriveInput)).into()
}

struct SortLinesInput {
    callback: syn::Ident,
    lines: Punctuated<(syn::Ident, syn::Token![:], syn::Type, syn::Token![=>], syn::LitInt), syn::token::Comma>,
}

impl syn::parse::Parse for SortLinesInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let callback = input.parse()?;
        let lines = input.parse_terminated(|input| {
            Ok((input.parse()?, input.parse()?, input.parse()?, input.parse()?, input.parse()?))
        })?;

        Ok(Self {
            callback,
            lines,
        })
    }
}

#[proc_macro]
pub fn sort_lines(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SortLinesInput);

    let mut lines = input.lines.into_iter().collect::<Vec<_>>();

    lines.sort_by_key(|a| {
        a.4.base10_parse::<u64>().unwrap()
    });

    let lines = lines.iter().map(|(field, _, ty, _, expr)| {
        quote! {
            #field: #ty => #expr,
        }
    });

    let cb = input.callback;

    (quote! {
        #cb!{
            #(#lines)*
        }
    }).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
