use std::collections::HashSet;

use quote2::{
    Quote, ToTokens,
    proc_macro2::{Punct, Spacing, TokenStream},
    quote,
};
use syn::{Data, DataStruct, DeriveInput, Error, Expr, Field, Meta, Type, spanned::Spanned};

pub fn expand(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        ref data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let mut seen: HashSet<&Expr> = HashSet::new();

            for field in fields {
                if let Some(key) = get_key(field) {
                    match seen.get(key) {
                        Some(key1) => {
                            let mut err = Error::new(key1.span(), "duplicate key");
                            err.combine(Error::new(
                                key.span(),
                                format!(
                                    "duplicate key `{}` later defined here",
                                    key1.to_token_stream()
                                ),
                            ));
                            let err = err.to_compile_error();
                            quote!(t, { #err });
                        }
                        None => {
                            seen.insert(key);
                        }
                    }

                    let ident = &field.ident;
                    let ref_symbol = match field.ty {
                        Type::Reference(_) => None,
                        _ => Some(Punct::new('&', Spacing::Alone)),
                    };

                    quote!(t, {
                        ::cor::FieldEncoder::encode(#ref_symbol self.#ident, w, #key)?;
                    });
                }
            }
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut t = TokenStream::new();
    quote!(t, {
        impl #impl_generics ::cor::Encoder for #ident #ty_generics #where_clause {
            fn encode(&self, w: &mut (impl ::std::io::Write + ?::std::marker::Sized)) -> ::std::io::Result<()> {
                #body
                ::std::io::Write::write_all(w, &[10])?;
                ::std::result::Result::Ok(())
            }
        }
    });
    t
}

fn get_key(field: &Field) -> Option<&Expr> {
    field.attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(kv) => kv.path.is_ident("key").then_some(&kv.value),
        _ => None,
    })
}
