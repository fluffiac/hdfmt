use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Variant};

static MISSING: &str = "enum field must be assigned a discriminant value";
static UNION: &str = "union is not supported";
macro_rules! err {
	($span:expr, $msg:expr) => {
		return syn::Error::new($span, $msg).to_compile_error().into()
	};
}

#[proc_macro_derive(Checked)]
pub fn check_proc(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	quote! {
		impl Checked for #name {}
	}.into()
}

#[proc_macro_derive(ReadOut)]
pub fn read_out_proc(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	return match input.data {
		syn::Data::Struct(data) => do_struct(input.ident, data),
		syn::Data::Enum(data) => do_enum(input.ident, data),
		syn::Data::Union(_) => err!(input.ident.span(), UNION),
	};
}

fn do_struct(name: syn::Ident, data: syn::DataStruct) -> TokenStream {
	let ts = match data.fields {
		Fields::Named(fields) => {
			let fields = fields.named.iter().map(|f| &f.ident);

			quote! { Ok( #name {
				#(#fields: Checked::check(buf)?),*
			}) }
		},
		Fields::Unnamed(fields) => {
			let fields = fields
				.unnamed
				.iter()
				.map(|_| quote! { Checked::check(buf)? });

			quote! { Ok( #name (#( #fields ),*)) }
		},
		Fields::Unit => {
			quote! { Ok( #name ) }
		},
	};

	quote! {
		impl ReadOut for #name {
			#[inline(always)]
			fn read_out(buf: &mut impl Read) -> std::io::Result<Self> {
				#ts
			}
		}
	}.into()
}

fn do_enum(name: syn::Ident, data: syn::DataEnum) -> TokenStream {
	let mut ts = Vec::new();
	
	for Variant{ ident, fields, discriminant, .. } in data.variants {
		let tag = match discriminant {
			Some((_, tag)) => tag,
			None => err!(ident.span(), MISSING),
		};

		ts.push(match fields {
			Fields::Named(fields) => {
				let fields = fields.named.iter().map(|f| &f.ident);

				quote! { #tag => Ok( #name :: #ident {
						#(#fields: Checked::check(buf)?),*
				},) }
			}
			Fields::Unnamed(fields) => {
				let fields = fields
					.unnamed
					.iter()
					.map(|_| quote! { Checked::check(buf)? });

				quote! { #tag => Ok( Self::#ident(#( #fields ),*)) }
			}
			Fields::Unit => {
				quote! { #tag => Ok( Self::#ident ) }
			},
		})
	}

	quote! {
		impl ReadOut for #name {
			fn read_out(buf: &mut impl Read) -> std::io::Result<Self> {
				match u8::read_out(buf)? {
					#(#ts),*
					, i => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
				}
			}
		}
	}.into()
}

