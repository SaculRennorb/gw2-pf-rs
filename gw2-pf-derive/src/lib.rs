use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, DeriveInput, Fields};



#[proc_macro_derive(Parse, attributes(chunk, v, packfile, null_terminated))]
pub fn derive_parse(input : TokenStream) -> TokenStream {
	let DeriveInput { ident: root_ident, data, attrs, generics: root_generics, .. } = parse_macro_input!(input);

	let mut output = proc_macro2::TokenStream::new();

	let input_lt = {
		let lt = ({
			let mut candidate = None;
			for param in root_generics.params.iter() {
				let syn::GenericParam::Lifetime(ref _lt_param) = param else { continue };
				match candidate {
					None => {
						candidate = Some(param.clone())
					},
					Some(ref _old) => {
						// todo warning
					},
				}
			}
			candidate
		}).unwrap_or_else(|| {
			syn::GenericParam::Lifetime(syn::LifetimeParam::new(syn::Lifetime::new("'inp", root_ident.span())))
		});

		let mut params = Punctuated::new();
		params.push_value(lt.clone());
		syn::Generics { params,	..Default::default() }
	};
	
	let output2 = match data {
		syn::Data::Struct(_struct) => {
			let mut err = proc_macro2::TokenStream::new();
			for attr in attrs {
				if let syn::Meta::Path(meta) = attr.meta {
					if meta.is_ident("chunk") {
						err.extend(syn::Error::new(meta.span(), "chunk is only valid for enums").into_compile_error())
					}
					else if meta.is_ident("packfile") {
						err.extend(syn::Error::new(meta.span(), "packfile is only valid for enums").into_compile_error())
					}
					else if meta.is_ident("null_terminated") {
						err.extend(syn::Error::new(meta.span(), "null_terminated is only valid for struct members").into_compile_error())
					}
				}
			}

			let Fields::Named(fields) = _struct.fields else { panic!() };

			

			let mut sizes = Vec::with_capacity(fields.named.len());
			for field in &fields.named {
				let _type = &field.ty;
				let span = field.ty.span();
				// todo strip lifetimes instead of wraping in triangle brackets
				sizes.push(quote_spanned!(span => <#_type>::BINARY_SIZE));
			}
			let sizes = match sizes.len() {
				0 => proc_macro2::TokenStream::new(),
				1 => sizes.into_iter().next().unwrap(),
				_ => {
					let first = &sizes[0];
					let sizes = sizes.iter().skip(1).map(|t| quote!{ .add(&#t) });
					quote!{ #first #(#sizes)* }
				}
			};
			

			let fields = fields.named.iter().map(|f| {
				let ident = f.ident.as_ref().unwrap();
				let span = f.ty.span();

				if f.attrs.iter().any(|attr| attr.meta.path().is_ident("null_terminated")) {
					quote_spanned!(span => #ident: crate::parse::parse_null_terminated_vec(input)?)
				}
				else {
					quote_spanned!(span => #ident: Parse::parse(input)?)
				}
			});


			quote! {
				#[automatically_derived]
				impl #input_lt crate::parse::Parse #input_lt for #root_ident #root_generics {
					const BINARY_SIZE : crate::parse::BinarySize = #sizes;
					fn parse(input : &mut crate::parse::Input #input_lt) -> Result<Self, crate::parse::Error> {
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
			let mut result = proc_macro2::TokenStream::new();
			for attr in attrs {
				let syn::Meta::Path(ref meta) = attr.meta else { continue };

				if meta.is_ident("chunk") {
					let fields = _enum.variants.iter().filter_map(|f| {
						let field_ident = &f.ident;
						if let Some(version_attr) = f.attrs.iter().find(|attr| matches!(attr.meta, syn::Meta::List(ref meta) if meta.path.is_ident("v"))) {
							let version = version_attr.parse_args_with(syn::LitInt::parse).unwrap();

							let tuple_field = match f.fields {
								Fields::Unnamed(ref field) => { &field.unnamed[0] },
								_ => todo!(),
							};
							let span = tuple_field.span();
							Some(quote_spanned!(span => #version => Parse::parse(input).map(Self::#field_ident)))
						}
						else{
							result.extend(syn::Error::new(field_ident.span(), "missing version attribute, add `#[v(..)]`").to_compile_error());
							None
						}
					});

					let own_magic = syn::LitByteStr::new(root_ident.to_string().as_bytes(), root_ident.span());

					result = quote! {
						#[automatically_derived]
						impl #input_lt crate::parse::ParseVersioned #input_lt for #root_ident #root_generics {
							type Output = Self;
							fn parse(version : u16, input : &mut crate::parse::Input #input_lt) -> crate::parse::Result<Self::Output> {
								use crate::parse::Parse;
								match version {
									#(#fields),*,
									_ => Err(crate::parse::Error::UnknownVersion { r#type: std::any::type_name::<#root_ident>(), actual: version }),
								}
							}
						}

						#[automatically_derived]
						impl #input_lt crate::parse::ParseMagicVariant #input_lt for #root_ident #root_generics {
							fn parse(magic : u32, version : u16, input : &mut crate::parse::Input #input_lt) -> crate::parse::Result<Self> {
								match magic {
									<Self as crate::pf::Magic>::MAGIC => <Self as crate::parse::ParseVersioned>::parse(version, input),
									_ => Err(crate::parse::Error::UnknownMagic { r#type: std::any::type_name::<#root_ident>(), actual: magic }),
								}
							}
						}

						#[automatically_derived]
						impl #root_generics crate::pf::Magic for #root_ident #root_generics {
							const MAGIC : u32 = crate::fcc(#own_magic);
						}
					};

					derive_deref_if_only_one_variant(&mut result, &root_ident, &root_generics, &_enum);

					break;
				}
				else if meta.is_ident("packfile") {
					let fields = _enum.variants.iter().filter_map(|f| {
						let field_ident = &f.ident;
						let tuple_field = match f.fields {
							Fields::Unnamed(ref field) => { &field.unnamed[0] },
							_ => todo!(),
						};
						let span = tuple_field.span();
						let field_type = match &tuple_field.ty {
							syn::Type::Path(p) => p,
							_ => todo!(),
						};
						let field_type_no_lt = strip_lifetimes_from_path(&field_type.path.segments);
						Some(quote_spanned!(span => #field_type_no_lt::MAGIC => <#field_type as ParseVersioned>::parse(version, input).map(Self::#field_ident)))
					});

					let own_magic = syn::LitByteStr::new(root_ident.to_string().as_bytes(), root_ident.span());

					let _impl = quote! {
						#[automatically_derived]
						impl #root_generics crate::pf::Magic for #root_ident #root_generics {
							const MAGIC : u32 = crate::fcc(#own_magic);
						}

						#[automatically_derived]
						impl #input_lt crate::parse::ParseMagicVariant #input_lt for #root_ident #root_generics {
							fn parse(magic : u32, version : u16, input : &mut crate::parse::Input #input_lt) -> crate::parse::Result<Self> {
								use crate::pf::Magic;
								use crate::parse::ParseVersioned;
								match magic {
									#(#fields),*,
									_ => Err(crate::parse::Error::UnknownMagic { r#type: std::any::type_name::<Self>(), actual: magic }),
								}
							}
						}
					};
					result.extend(_impl);

					derive_deref_if_only_one_variant(&mut result, &root_ident, &root_generics, &_enum);

					break;
				}
			}

			result
		},
		syn::Data::Union(_) => {
			todo!()
		},
	};
	output.extend(output2);
	//panic!("{}", output);
	output.into()
}

type Path = Punctuated<syn::PathSegment, syn::token::PathSep>;
fn strip_lifetimes_from_path(path : &Path) -> Path {
	let mut new_path = Path::new();
	for segment in path {
		let arguments = match segment.arguments {
			syn::PathArguments::AngleBracketed(ref generics) => {
				let mut new_generics = Punctuated::<syn::GenericArgument, syn::token::Comma>::new();
				for generic in &generics.args {
					if matches!(generic, syn::GenericArgument::Lifetime(_)) { continue }
					new_generics.push(generic.clone());
				}
				if new_generics.len() == 0 {
					syn::PathArguments::None
				}
				else {
					syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
						args        : new_generics,
						colon2_token: generics.colon2_token,
						gt_token    : generics.gt_token,
						lt_token    : generics.lt_token,
					})
				}
			},
			_ => syn::PathArguments::None,
		};
		new_path.push(syn::PathSegment {
			arguments,
			ident: segment.ident.clone(),
		});
	}
	new_path
}

fn derive_deref_if_only_one_variant<'a>(output : &mut proc_macro2::TokenStream, root_ident : &syn::Ident, root_generics : &syn::Generics, _enum : &'a syn::DataEnum) -> Option<(&'a syn::Ident, &'a syn::Type)> {
	if _enum.variants.len() != 1 { return None }

	let only_variant = &_enum.variants[0];
	let variant_ident = &only_variant.ident;
	let variant_type = match only_variant.fields {
		Fields::Unnamed(ref f) => &f.unnamed[0].ty,
		_ => todo!(),
	};

	let _impl = quote! {
		#[automatically_derived]
		impl #root_generics std::ops::Deref for #root_ident #root_generics {
			type Target = #variant_type;
			fn deref(&self) -> &Self::Target { match self { Self::#variant_ident(ref s) => s } }
		}
		#[automatically_derived]
		impl #root_generics std::ops::DerefMut for #root_ident #root_generics {
			fn deref_mut(&mut self) -> &mut Self::Target { match self { Self::#variant_ident(ref mut s) => s } }
		}
	};

	output.extend(_impl);

	Some((variant_ident, variant_type))
}
