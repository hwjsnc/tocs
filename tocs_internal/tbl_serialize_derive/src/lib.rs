extern crate ioutil;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use quote::ToTokens as _;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, };

#[proc_macro_derive(Serialize, attributes(expected_length))]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Adapted from the heapsize_derive example of the syn crate.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Add a bound `T: Serialize` to type parameters
    let generics = add_trait_bounds(input.generics);
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Add a new bound `W: std::io::Write` for the `impl<W> Serialize<W> for X`
    let impl_generics = add_write_trait(generics.clone()).into_token_stream();

    let serialize_statements = generate_serialize_statements(&input.data);

    quote! {
        impl #impl_generics ioutil::tbl::Serialize<W> for #name #ty_generics #where_clause {
            fn serialize(&self, out : &mut W) -> Result<(), std::io::Error> {
                #serialize_statements
                Ok(())
            }
        }
    }.into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(ioutil::tbl::Serialize));
        }
    }
    generics
}

fn add_write_trait(mut generics: Generics) -> Generics {
    generics.params.push(parse_quote!(W: std::io::Write));
    generics
}

fn generate_serialize_statements(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let names_with_span = fields.named.iter().map(|f| {
                        let fieldname = f.ident.as_ref().expect("named identifier");
                        let span = f.span();
                        quote_spanned! {span=> self.#fieldname.serialize(out)?;}
                    });
                    quote! { #(#names_with_span);* }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}
