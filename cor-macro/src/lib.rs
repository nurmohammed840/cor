mod decoder;
mod encoder;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Encoder, attributes(key))]
pub fn encoder(input: TokenStream) -> TokenStream {
    encoder::expand(&syn::parse_macro_input!(input)).into()
}

#[proc_macro_derive(Decoder)]
pub fn decoder(input: TokenStream) -> TokenStream {
    decoder::expand(&syn::parse_macro_input!(input)).into()
}
