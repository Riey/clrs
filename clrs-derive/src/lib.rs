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
            syn::Ident,
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

    let impls = lines.iter().map(|(field, _, ty, ..)| {
        let index_ty_name = syn::Ident::new(&format!("{}Index", ty), ty.span());

        quote! {
            #[derive(Copy, Clone, Debug)]
            pub struct #index_ty_name(pub u32);

            impl From<u32> for #index_ty_name {
                fn from(n: u32) -> Self {
                    Self(n)
                }
            }

            impl<'a> TryFromCtx<'a, PeCtx> for #index_ty_name {
                type Error = scroll::Error;

                fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                    let n: u16 = src.pread_with(0, ctx)?;
                    Ok((Self(n as _), 2))
                }
            }

            impl TableIndex<#ty> for #index_ty_name {
                fn resolve_table(self, table: &MetadataTable) -> Option<&#ty> {
                    // row index is one based zero means `NULL`
                    table.#field.get((self.0 as usize).checked_sub(1)?)
                }
            }
        }
    });

    let token_variants = lines.iter().map(|(_, _, ty, _, _)| {
        let index_ty_name = syn::Ident::new(&format!("{}Index", ty), ty.span());

        quote! {
            #ty(#index_ty_name),
        }
    });

    let token_arms = lines.iter().map(|(_, _, ty, _, expr)| {
        quote! {
            x if x == #expr << 24 => Ok((Self::#ty(rid.into()), 4)),
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

        #[derive(Clone, Copy, Debug)]
        pub enum MetadataToken {
            #(#token_variants)*
            Document(u32),
            MethodDebugInformation(u32),
            LocalScope(u32),
            LocalVariable(u32),
            LocalConstant(u32),
            ImportScope(u32),
            StateMachineMethod(u32),
            CustomDebugInformation(u32),

            String(StringIndex),
        }

        impl<'a> TryFromCtx<'a, ::scroll::Endian> for MetadataToken {
            type Error = ::scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: ::scroll::Endian) -> Result<(Self, usize), Self::Error> {
                let token: u32 = src.pread_with(0, ctx)?;
                let rid = token & 0x00FFFFFF;

                let ty = token & 0xFF000000;

                match ty {
                    #(#token_arms)*
                    0x30000000 => Ok((Self::Document(rid), 4)),
                    0x31000000 => Ok((Self::MethodDebugInformation(rid), 4)),
                    0x32000000 => Ok((Self::LocalScope(rid), 4)),
                    0x33000000 => Ok((Self::LocalVariable(rid), 4)),
                    0x34000000 => Ok((Self::LocalConstant(rid), 4)),
                    0x35000000 => Ok((Self::ImportScope(rid), 4)),
                    0x36000000 => Ok((Self::StateMachineMethod(rid), 4)),
                    0x37000000 => Ok((Self::CustomDebugInformation(rid), 4)),

                    0x70000000 => Ok((Self::String(StringIndex(rid)), 4)),
                    _ => Err(::scroll::Error::BadInput { size: 4, msg: "Bad TokenType" }),
                }
            }
        }

        pub trait TableIndex<T> {
            fn resolve_table(self, table: &MetadataTable) -> Option<&T>;
        }

        #(#impls)*
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
