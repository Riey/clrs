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
        attrs: _,
        data,
        generics: _,
        vis: _,
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

struct MakeTableInput {
    lines: Punctuated<
        (
            syn::Ident,
            syn::Token![:],
            syn::Type,
            syn::Token![=>],
            syn::LitInt,
        ),
        syn::token::Comma,
    >,
}

impl syn::parse::Parse for MakeTableInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lines = input.parse_terminated(|input| {
            Ok((
                input.parse()?,
                input.parse()?,
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        })?;

        Ok(Self { lines })
    }
}

#[proc_macro]
pub fn make_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MakeTableInput);

    let mut lines = input.lines.into_iter().collect::<Vec<_>>();

    lines.sort_by_key(|a| a.4.base10_parse::<u64>().unwrap());

    let fields = lines.iter().map(|(field, _, ty, ..)| {
        quote! {
            #field: Vec<#ty>,
        }
    });

    let init_field = lines.iter().map(|(field, ..)| {
        quote! {
            let mut #field = (Vec::new(), 0);
        }
    });

    let ifs = lines.iter().map(|(field, .., expr)| {
        quote! {
            if vaild_bitvec & (1 << #expr) != 0 {
                #field.1 = src.gread_with::<u32>(offset, ctx)?;
                vaild_bitvec &= (!(1 << #expr));
            }
        }
    });

    let pushs = lines.iter().map(|(field, ..)| {
        quote! {
            for _ in 0..#field.1 {
                #field.0.push(src.gread_with(offset, ctx)?);
            }

            let #field = #field.0;
        }
    });

    let ret = lines.iter().map(|(field, ..)| {
        quote! {
            #field,
        }
    });

    (quote! {
        #[derive(Clone, Debug)]
        pub struct MetadataTable {
            #(#fields)*
        }

        impl<'a> TryFromCtx<'a, PeCtx> for MetadataTable {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                let offset = &mut 0;

                let mut vaild_bitvec: u64 = src.gread_with(offset, ctx)?;
                let _sorted_table_bitvec: u64 = src.gread_with(offset, ctx)?;

                #(#init_field)*

                #(#ifs)*

                assert_eq!(vaild_bitvec, 0, "Unknown table bitvec presents {:X}", vaild_bitvec);

                #(#pushs)*

                Ok((Self {
                    #(#ret)*
                }, *offset))
            }
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
