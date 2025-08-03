use quote2::{
    Quote,
    proc_macro2::{Span, TokenStream},
    quote,
};
use syn::*;

pub fn expand(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            for field in fields {
                match crate::utils::get_key(field) {
                    Some(key) => {
                        let key_name = &field.ident;
                        quote!(t, { #key_name: e.get_and_convert(#key)?, });
                    }
                    None => {
                        let key_name = &field.ident;
                        quote!(t, { #key_name: ::std::default::Default::default(), });
                    }
                }
            }
        }
        Data::Enum(_) => {}
        Data::Union(_) => todo!(),
    });

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    // Add a bound `T: Decode<'de>` to every type parameter of `T`.
    let bound: TypeParamBound = parse_quote!(::cor::Decode<'decode>);
    let mut params = generics.params.clone();
    let mut lifetime = LifetimeParam::new(Lifetime::new("'decode", Span::call_site()));

    for param in params.iter_mut() {
        match param {
            GenericParam::Type(ty) => ty.bounds.push(bound.clone()),
            GenericParam::Lifetime(lt) => lifetime.bounds.push(lt.lifetime.clone()),
            _ => {}
        }
    }

    let mut t = TokenStream::new();
    quote!(t, {
        impl <#lifetime, #params> ::cor::Decoder<'decode> for #ident #ty_generics #where_clause {
            fn decode(e: &::cor::Entries<'decode>) -> ::std::io::Result<Self> {
                Ok(Self { #body })
            }
        }

        impl <#lifetime, #params> TryFrom<&::cor::Value<'decode>> for #ident #ty_generics #where_clause {
            type Error = std::io::Error;
            fn try_from(v: &::cor::Value<'decode>) -> Result<Self, Self::Error> {
                ::cor::__private::convert_into_struct(v)
            }
        }

    });
    t
}
