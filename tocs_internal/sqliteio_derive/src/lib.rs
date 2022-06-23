extern crate sqliteio;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Lit, Meta, NestedMeta};

#[proc_macro_derive(SqliteIO, attributes(sqlite))]
pub fn derive_sqliteio(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Adapted from the heapsize_derive example of the syn crate.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let table_name_str = get_attr_table_name(&input.attrs);
    let add_column_names_statements = generate_column_names_statements(&input.data);
    let add_values_to_tup_statements = generate_value_statements(&input.data);
    let read_values_statements = generate_read_statements(&input.data);
    let fields = get_field_names(&input.data);

    quote! {
        impl #impl_generics sqliteio::SqliteIO for #name #ty_generics #where_clause {
            fn sqlite_table() -> &'static str { #table_name_str }

            fn sqlite_columns() -> std::vec::Vec<String> {
                let mut columns = std::vec::Vec::<String>::new();
                #add_column_names_statements
                columns
            }

            fn sqlite_to_tup(&self) -> std::vec::Vec<&dyn rusqlite::ToSql> {
                let mut tup = std::vec::Vec::<&dyn rusqlite::ToSql>::new();
                #add_values_to_tup_statements
                tup
            }

            fn sqlite_row_read(row: &rusqlite::Row<'_>, start_idx: usize) -> rusqlite::Result<(Self, usize)> {
                #read_values_statements
                Ok(( Self { #(#fields),* }, start_idx ))
            }
        }
    }
    .into()
}

fn get_attr_table_name(attrs: &[syn::Attribute]) -> Lit {
    let token_sqlite = Ident::new("sqlite", Span::call_site());
    let token_table = Ident::new("table", Span::call_site());
    for attr in attrs {
        let meta = attr.parse_meta().expect("attribute meta");
        match meta {
            Meta::List(list) => {
                if list.path.get_ident() == Some(&token_sqlite) {
                    for nested in list.nested.iter() {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(m)) => {
                                if m.path.get_ident() == Some(&token_table) {
                                    return m.lit.clone();
                                }
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
    panic!("attribute sqlite(table = \"table name\") not specified");
}

fn generate_column_names_statements(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let column_statements = fields.named.iter().map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let t = &f.ty;
                    let span = f.span();
                    let fieldname_string = fieldname.to_string().escape_default().to_string();
                    quote_spanned! {span=>
                        for column in <#t>::sqlite_columns().iter() {
                            let s = #fieldname_string;
                            if column.is_empty() {
                                columns.push(format!("{}", s))
                            } else {
                                columns.push(format!("{} {}", s, column))
                            }
                        }
                    }
                });
                quote! { #(#column_statements)* }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn generate_value_statements(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let value_statements = fields.named.iter().map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let span = f.span();
                    quote_spanned! {span=> tup.extend_from_slice(&self.#fieldname.sqlite_to_tup()); }
                });
                quote! { #(#value_statements)* }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn generate_read_statements(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let value_statements = fields.named.iter().map(|f| {
                    let fieldname = f.ident.as_ref().expect("named identifier");
                    let t = &f.ty;
                    let span = f.span();
                    quote_spanned! {span=> let (#fieldname, start_idx) = <#t>::sqlite_row_read(row, start_idx)?; }
                });
                quote! { #(#value_statements)* }
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
