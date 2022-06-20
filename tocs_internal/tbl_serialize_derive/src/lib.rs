extern crate ioutil;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Meta};

#[proc_macro_derive(Serialize, attributes(serialize))]
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
            }
        }
    }
    .into()
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
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let token_serialize = Ident::new("serialize", Span::call_site());
                let token_ignore = Ident::new("ignore", Span::call_site());
                let ignore = |attr: &syn::Attribute| {
                    let meta = attr.parse_meta().expect("attribute");
                    match meta {
                        Meta::List(list) => {
                            list.path.get_ident() == Some(&token_serialize)
                                && list
                                    .nested
                                    .iter()
                                    .any(|nested| matches!(nested, syn::NestedMeta::Meta(m) if m.path().get_ident() == Some(&token_ignore)))
                        }
                        Meta::Path(_) => unimplemented!(),
                        Meta::NameValue(_) => unimplemented!(),
                    }
                };
                let names_with_span = fields.named.iter().filter_map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let span = f.span();
                    if f.attrs.iter().any(ignore) {
                        None
                    } else {
                        Some(quote_spanned! {span=> self.#fieldname.serialize(out)?;})
                    }
                });
                quote! { #(#names_with_span);* Ok(()) }
            }
            _ => unimplemented!(),
        },
        Data::Enum(ref data) => {
            // Note: we don't try to serialize general enums.
            // We only care about the general "Entry" enum which is a tagged union of all entry types (e.g. "item", "magic", "voice", "char_revise").
            // The tbl serialization code must add a header (header name and entry length), which is outside the scope of this macro.
            // The macro only "peeks" into each variant and calls .serialize() on the variant field (singular).
            let names_with_spans = data.variants.iter().map(|f| {
                let variantname = &f.ident;
                let span = f.span();
                quote_spanned! {span=> #variantname}
            });
            quote! {
                match self {
                    #(Self::#names_with_spans(data) => data.serialize(out)),*
                }
            }
        }
        _ => unimplemented!(),
    }
}
