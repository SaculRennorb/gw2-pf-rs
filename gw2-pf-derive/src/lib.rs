use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned, DeriveInput, Fields};



#[proc_macro_derive(Parse, attributes(versioned_chunk, v))]
pub fn derive_parse(input : TokenStream) -> TokenStream {
	let DeriveInput { ident, data, attrs, .. } = parse_macro_input!(input);
	
	let output = match data {
		syn::Data::Struct(_struct) => {
			let Fields::Named(fields) = _struct.fields else { panic!() };
			let fields = fields.named.iter().map(|f| {
				let ident = f.ident.as_ref().unwrap();
				let span = f.ty.span();
				quote_spanned!(span => #ident: Parse::parse(input)?)
			});

			let mut err : Option<proc_macro2::TokenStream> = None;

			if let Some(versioned_chunk_attr) = attrs.iter().find(|attr| matches!(attr.meta, syn::Meta::Path(ref p) if p.is_ident("versioned_chunk"))) {
				err = Some(syn::Error::new(versioned_chunk_attr.meta.span(), "versioned_chunk is only valid for enums").into_compile_error());
			}

			quote! {
				#[automatically_derived]
				impl crate::parse::Parse for #ident {
					fn parse(input : &mut crate::parse::Input) -> Result<Self, crate::parse::Error> {
						use crate::parse::Parse;
						Ok(Self {
							#(#fields),*
						})
					}
				}

				#err
			}
		},
		syn::Data::Enum(_enum) => {
			if attrs.iter().any(|attr| matches!(attr.meta, syn::Meta::Path(ref p) if p.is_ident("versioned_chunk"))) {
 
				let fields = _enum.variants.iter().map(|f| {
					let field_ident = &f.ident;
					let Some(version_attr) = f.attrs.iter().find(|attr| matches!(attr.meta, syn::Meta::List(ref meta) if meta.path.is_ident("v"))) else { return syn::Error::new(field_ident.span(), "missing version attribute").to_compile_error(); };
					let version = version_attr.parse_args_with(syn::LitInt::parse).unwrap();

					let inner_type = match f.fields {
						Fields::Unnamed(ref field) => {
							field.unnamed.first().unwrap()
						},
						_ => todo!(),
					};
					let span = inner_type.span();
					quote_spanned!(span => #version => Parse::parse(input).map(Self::#field_ident))
				});

				let chunk_magic = syn::LitByteStr::new(ident.to_string().as_bytes(), ident.span());

				quote! {
					#[automatically_derived]
					impl crate::parse::ParseVersioned for #ident {
						fn parse(version : u16, input : &mut crate::parse::Input) -> Result<Self, crate::parse::Error> {
							use crate::parse::Parse;
							match version {
								#(#fields),*,
								_ => Err(crate::parse::Error::UnknownVersion { actual: version }),
							}
						}
					}

					#[automatically_derived]
					impl crate::pf::Chunk for #ident {
						const MAGIC : u32 = crate::fcc(#chunk_magic);
					}
			}
			}
			else {
				todo!()
			}

		},
		syn::Data::Union(_) => {
			todo!()
		},
	};
	//panic!("{}", output);
	output.into()
}
