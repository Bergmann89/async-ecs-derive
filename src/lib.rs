#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod component;
mod system_data;

use proc_macro::TokenStream;

#[proc_macro_derive(SystemData)]
pub fn system_data(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    let gen = system_data::execute(&ast);

    gen.into()
}

#[proc_macro_derive(Component, attributes(storage))]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    let gen = component::execute(&ast);

    gen.into()
}
