use quote2::{Quote, proc_macro2::TokenStream, quote};
use syn::{Data, DataStruct, DeriveInput, Expr, Field, Meta};

pub fn expand(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        ref data,
        ..
    } = input;

    let body = quote(|t| match data {
        Data::Struct(DataStruct { fields, .. }) => {
            for field in fields {
                if let Some(key) = get_key(field) {
                    let ident = &field.ident;
                    quote!(t, {
                        ::cor::FieldEncoder::encode(&self.#ident, w, #key)?;
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
        Meta::NameValue(kv) => kv.path.is_ident("key").then(|| &kv.value),
        _ => None,
    })
}
