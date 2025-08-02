#![allow(warnings)]
use quote2::{Quote, ToTokens, proc_macro2::TokenStream, quote};
use syn::DeriveInput;

pub fn expand(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut t = TokenStream::new();
    quote!(t, {
        impl #impl_generics ::cor::Decoder for #ident #ty_generics #where_clause {
            fn decode(_: &mut &'de [u8]) -> ::std::io::Result<Self> {
                todo!()
            }
        }
    });
    t
}
