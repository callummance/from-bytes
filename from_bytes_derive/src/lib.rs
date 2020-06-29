#![recursion_limit = "128"]

use proc_macro::TokenStream;
use proc_macro2;

use quote::quote;

#[proc_macro_derive(FromBytes, attributes(size, offset))]
pub fn derive_frombytes(tokens: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    let fields = calculate_field_offsets(&ast);
    let name = ast.ident.clone();
    gen_impl(fields, &name).into()
}

#[derive(Debug)]
struct StructField {
    pub field_ident: syn::Ident,
    pub size_expr: proc_macro2::TokenStream,
    pub offset_expr: proc_macro2::TokenStream,
}

fn gen_impl(fields: Vec<StructField>, name: &syn::Ident) -> TokenStream {
    let bytes_read_impl = impl_bytes_read(&fields);
    let bytes_size_impl = impl_bytes_size(&fields);

    (quote! {
        impl FromBytes for #name {
            #bytes_read_impl
            #bytes_size_impl
        }
    })
    .into()
}

fn impl_bytes_read(fields: &Vec<StructField>) -> proc_macro2::TokenStream {
    let mut read_field_impls: Vec<proc_macro2::TokenStream> = Vec::with_capacity(fields.len());
    for field in fields {
        let start_idx_expr = field.offset_expr.clone();
        let field_size = field.size_expr.clone();
        let end_idx_expr = quote! {#start_idx_expr + #field_size};
        let field_ident = field.field_ident.clone();
        let field_copy_statement = quote! {
            start_idx = #start_idx_expr;
            end_idx = #end_idx_expr;
            self.#field_ident.load_from_bytes(&bytes[start_idx..end_idx])?;
        };
        read_field_impls.push(field_copy_statement);
    }
    quote! {
        fn load_from_bytes(&mut self, bytes: &[u8]) -> from_bytes::ReadFromBytesResult<()> {
            let mut start_idx: usize;
            let mut end_idx: usize;
            if bytes.len() < self.bytes_size() {
                return Err(from_bytes::ReadFromBytesError::BytesArrayTooSmall(self.bytes_size(), bytes.len()));
            } else {
                #(#read_field_impls)*
                Ok(())
            }
        }
    }
}

fn impl_bytes_size(fields: &Vec<StructField>) -> proc_macro2::TokenStream {
    let last_field = fields.last().unwrap();
    let last_field_offset_expr = last_field.offset_expr.clone();
    let last_field_size_expr = last_field.size_expr.clone();
    let total_size_expr = quote! {#last_field_offset_expr + #last_field_size_expr};
    quote! {
        fn bytes_size(&self) -> usize {
            #total_size_expr
        }
    }
}

fn calculate_field_offsets(ast: &syn::DeriveInput) -> Vec<StructField> {
    let mut field_byte_data: Vec<StructField> = Vec::new();
    match ast.data {
        syn::Data::Struct(ref struct_data) => match struct_data.fields {
            syn::Fields::Named(ref struct_fields) => {
                let mut last_field_offset_expr = quote! {0};
                let mut last_field_size_expr = quote! {0};
                for field in struct_fields.named.iter() {
                    //For each field in the struct
                    //Base size and offset expressions, to be overwritten if helper attributes are found
                    let field_ident = field.ident.clone().unwrap();
                    let mut field_size_expr = quote! {
                        self.#field_ident.bytes_size()
                    };
                    let mut field_offset_expr = quote! {
                        #last_field_offset_expr + #last_field_size_expr
                    };
                    // Check if we have custom offset or size annotations
                    for attr in field.attrs.iter() {
                        if attr.path.is_ident("size") {
                            let meta = attr.parse_meta().unwrap();
                            if let syn::Meta::NameValue(val) = meta {
                                if let syn::Lit::Int(size_lit) = val.lit {
                                    let size: usize = size_lit.base10_parse().unwrap();
                                    field_size_expr = quote! {#size};
                                }
                            }
                        } else if attr.path.is_ident("offset") {
                            let meta = attr.parse_meta().unwrap();
                            if let syn::Meta::NameValue(val) = meta {
                                if let syn::Lit::Int(offset_lit) = val.lit {
                                    let offset: usize = offset_lit.base10_parse().unwrap();
                                    field_offset_expr = quote! {#offset};
                                }
                            }
                        }
                    }
                    //Save the offset and size expressions for next field
                    last_field_offset_expr = field_size_expr.clone();
                    last_field_size_expr = field_offset_expr.clone();

                    let field_data = StructField {
                        field_ident: field.ident.clone().unwrap(),
                        size_expr: field_size_expr,
                        offset_expr: field_offset_expr,
                    };
                    field_byte_data.push(field_data);
                }
            }
            _ => panic!("Must be a named struct"),
        },
        _ => panic!("Must be a struct"),
    }
    return field_byte_data;
}
