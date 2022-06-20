extern crate ioutil;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Meta};

#[proc_macro_derive(SerializeLength, attributes(serialize))]
pub fn derive_serialize_length(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Adapted from the heapsize_derive example of the syn crate.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Add a bound `T: SerializeLength` to type parameters
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let length_expr = generate_data_length_expr(&input.data);

    quote! {
        impl #impl_generics ioutil::tbl::SerializeLength for #name #ty_generics #where_clause {
            fn data_length(&self) -> Option<u16> {
                #length_expr
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(ioutil::tbl::SerializeLength));
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

fn generate_data_length_expr(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let add_statements = fields.named.iter().filter_map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let span = f.span();
                    if f.attrs.iter().any(attr_ignore) {
                        None
                    } else {
                        Some(quote_spanned! {span=> sum = sum?.checked_add(self.#fieldname.data_length()?);})
                    }
                });
                quote! { let mut sum : Option<u16> = Some(0); #(#add_statements);* sum }
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
                    #(Self::#names_with_spans(data) => data.data_length()),*
                }
            }
        }
        _ => unimplemented!(),
    }
}
