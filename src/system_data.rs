use syn::{
    punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Lifetime, Type, WhereClause, WherePredicate,
};

pub fn execute(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let mut generics = ast.generics.clone();

    let (fetch_return, tys) = gen_from_body(&ast.data, name);
    let tys = &tys;
    let def_fetch_lt = ast
        .generics
        .lifetimes()
        .next()
        .expect("There has to be at least one lifetime");
    let impl_fetch_lt = &def_fetch_lt.lifetime;

    {
        let where_clause = generics.make_where_clause();
        constrain_system_data_types(where_clause, impl_fetch_lt, tys);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics
            async_ecs::system::SystemData< #impl_fetch_lt >
            for #name #ty_generics #where_clause
        {
            fn setup(resources: &mut async_ecs::resources::Resources) {
                #(
                    <#tys as async_ecs::system::SystemData> :: setup(world);
                )*
            }

            fn fetch(world: & #impl_fetch_lt async_ecs::resources::Resources) -> Self {
                #fetch_return
            }

            fn reads() -> Vec<async_ecs::resources::ResourceId> {
                let mut r = Vec::new();

                #( {
                        let mut reads = <#tys as async_ecs::system::SystemData> :: reads();
                        r.append(&mut reads);
                    } )*

                r
            }

            fn writes() -> Vec<async_ecs::resources::ResourceId> {
                let mut r = Vec::new();

                #( {
                        let mut writes = <#tys as async_ecs::system::SystemData> :: writes();
                        r.append(&mut writes);
                    } )*

                r
            }
        }
    }
}

fn collect_field_types(fields: &Punctuated<Field, Comma>) -> Vec<Type> {
    fields.iter().map(|x| x.ty.clone()).collect()
}

fn gen_identifiers(fields: &Punctuated<Field, Comma>) -> Vec<Ident> {
    fields.iter().map(|x| x.ident.clone().unwrap()).collect()
}

fn constrain_system_data_types(clause: &mut WhereClause, fetch_lt: &Lifetime, tys: &[Type]) {
    for ty in tys.iter() {
        let where_predicate: WherePredicate = parse_quote!(#ty : async_ecs::system::SystemData< #fetch_lt >);
        clause.predicates.push(where_predicate);
    }
}

fn gen_from_body(ast: &Data, name: &Ident) -> (proc_macro2::TokenStream, Vec<Type>) {
    enum DataType {
        Struct,
        Tuple,
    }

    let (body, fields) = match *ast {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named: ref x, .. }),
            ..
        }) => (DataType::Struct, x),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed: ref x, .. }),
            ..
        }) => (DataType::Tuple, x),
        _ => panic!("Enums are not supported"),
    };

    let tys = collect_field_types(fields);

    let fetch_return = match body {
        DataType::Struct => {
            let identifiers = gen_identifiers(fields);

            quote! {
                #name {
                    #( #identifiers: async_ecs::system::SystemData::fetch(world) ),*
                }
            }
        }
        DataType::Tuple => {
            let count = tys.len();
            let fetch = vec![quote! { async_ecs::system::SystemData::fetch(world) }; count];

            quote! {
                #name ( #( #fetch ),* )
            }
        }
    };

    (fetch_return, tys)
}
