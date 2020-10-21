use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    DeriveInput, Path,
};

pub fn execute(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let storage = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "storage")
        .map(|attr| {
            syn::parse2::<StorageAttribute>(attr.tokens.clone())
                .unwrap()
                .storage
        })
        .unwrap_or_else(|| parse_quote!(DenseVecStorage));

    quote! {
        impl #impl_generics async_ecs::component::Component for #name #ty_generics #where_clause {
            type Storage = #storage<Self>;
        }
    }
}

struct StorageAttribute {
    storage: Path,
}

impl Parse for StorageAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _parenthesized_token = parenthesized!(content in input);

        Ok(StorageAttribute {
            storage: content.parse()?,
        })
    }
}
