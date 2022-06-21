extern crate ioutil;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Meta};

#[proc_macro_derive(Deserialize, attributes(deserialize))]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Adapted from the heapsize_derive example of the syn crate.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Add a bound `T: Deserialize` to type parameters
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_names = get_field_names(&input.data);
    let deserialize_statements = generate_serialize_statements(&input.data);

    quote! {
        impl #impl_generics ioutil::tbl::Deserialize for #name #ty_generics #where_clause {
            fn deserialize(ctx: ioutil::ParseContext) -> Result<(Self, ioutil::ParseContext), ioutil::tbl::ParseErrorDetail> {
                #deserialize_statements
                Ok((Self { #(#field_names),* }, ctx))
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(ioutil::tbl::Deserialize));
        }
    }
    generics
}

fn attr_ignore(attr: &syn::Attribute) -> bool {
    let token_serialize = Ident::new("serialize", Span::call_site());
    let token_ignore = Ident::new("ignore", Span::call_site());

    let path_ignore = |nested_meta: &syn::NestedMeta| matches!(nested_meta, syn::NestedMeta::Meta(m) if m.path().get_ident() == Some(&token_ignore));

    let meta = attr.parse_meta().expect("attribute meta");
    match meta {
        Meta::List(list) => list.path.get_ident() == Some(&token_serialize) && list.nested.iter().any(path_ignore),
        Meta::Path(_) => unimplemented!(),
        Meta::NameValue(_) => unimplemented!(),
    }
}

fn generate_serialize_statements(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let deserialize_statements = fields.named.iter().filter_map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let t = &f.ty;
                    let span = f.span();
                    if f.attrs.iter().any(attr_ignore) {
                        None
                    } else {
                        Some(quote_spanned! {span=> let (#fieldname, ctx) = <#t>::deserialize(ctx)?;})
                    }
                });
                quote! { #(#deserialize_statements)* }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn get_field_names(data: &Data) -> std::vec::Vec<TokenStream> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let span = f.span();
                    quote_spanned! {span=> #fieldname}
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
