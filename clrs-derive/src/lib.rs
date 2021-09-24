extern crate proc_macro;

use quote::quote;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
