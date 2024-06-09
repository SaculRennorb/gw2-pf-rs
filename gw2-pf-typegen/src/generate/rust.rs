pub fn export_chunk<'a>(chunk : &Chunk<'a>, fmt : &mut Formatter) -> FmtResult {
	fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
	fmt.write_str("#[chunk]\n")?;
	fmt.write_str("pub enum ")?;
	fmt.write_str(chunk.magic)?;
	if chunk.holds_input_references { fmt.write_str("<'a>")?; }
	fmt.write_str(" {\n")?;
	for version in chunk.versions.iter() {
		fmt.write_fmt(format_args!("\t#[v({ver})] V{ver}(v{ver}::{}),\n", format_type_name(&version.root), ver = version.version))?;
	}
	fmt.write_str("}\n")
}

pub struct RecursiveTypeReferences<'a, 'b> {
	already_exported : HashSet<Type<'a>>,
	queue : Vec<&'b Type<'a>>,
}

impl<'a, 'b> RecursiveTypeReferences<'a, 'b> {
	pub fn new_with_seed(seed_type : &'b Type<'a>) -> Self {
		let mut me = Self{ queue: Vec::new(), already_exported: HashSet::new() };
		me.append_recursive(seed_type);
		me
	}

	pub fn len(&self) -> usize { self.queue.len() }

	fn append_recursive(&mut self, _type : &'b Type<'a>) {
		match _type {
				Type::Reference { inner, .. } |
				Type::Array { inner, .. } => {
					if !is_primitive_type(inner) {
						self.append_recursive(inner);
					}
				}
				Type::Variant { variants, .. } => {
					self.append(_type);
					for inner in variants {
						if !is_primitive_type(inner) {
							self.append_recursive(inner);
						}
					}
				}
				Type::Composite { fields, .. } =>  {
					self.append(_type);
					for field in fields {
						if !is_primitive_type(field) {
							self.append_recursive(field);
						}
					}
				},
				_ => {},
		}
	}

	pub fn append(&mut self, _type : &'b Type<'a>) {
		if !self.already_exported.contains(_type) {
			self.queue.push(_type);
		}
	}
}

fn is_primitive_type(_type : &Type) -> bool {
	match _type {
		Type::U8  |
		Type::U16 |
		Type::U32 |
		Type::U64 |
		Type::F32 |
		Type::F64 |
		Type::FileName |
		Type::FileRef |
		Type::Token |
		Type::UUID |
		Type::CString { .. } => true,
		_ => false,
	}
}

impl<'a, 'b, 'c> std::iter::IntoIterator for &'c RecursiveTypeReferences<'a, 'b> {
		type Item = &'c &'b Type<'a>;
		type IntoIter = core::slice::Iter<'c, &'b Type<'a>>;
		fn into_iter(self) -> Self::IntoIter { self.queue.iter() }
}

pub fn export_type<'a>(_type : &Type<'a>, fmt : &mut Formatter) -> FmtResult {
	match _type {
		Type::Composite { name, fields, holds_input_references } => {
			let longest_name_len = fields.iter().map(|f| format_member_name(f.name).len()).max().unwrap_or(0); 

			fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
			fmt.write_str("pub struct ")?;
			fmt.write_str(name)?;
			if *holds_input_references { fmt.write_str("<'a>")?; }
			fmt.write_str(" {\n")?;
			for field in fields.iter() {
				fmt.write_char('\t')?;
				let field_name = format_member_name(field.name);
				fmt.write_str(&field_name)?;
				let mut padding = longest_name_len.saturating_sub(field_name.len());
				while padding > 0 {
					fmt.write_char(' ')?;
					padding -= 1;
				}
				fmt.write_str(" : ")?;
				fmt.write_str(&format_type_name(field))?;
				match field._type {
					Type::Array { kind: ArrayKind::DynamicSmall { .. }, .. } => { fmt.write_str(", // small")? } ,
					_ => fmt.write_str(",")?,
				}
				match field._type {
					Type::Array { kind: ArrayKind::Dynamic { size }, .. } |
					Type::Array { kind: ArrayKind::DynamicSmall { size }, .. } |
					Type::Array { kind: ArrayKind::Pointers { size }, .. } if size > 0 => {
						fmt.write_fmt(format_args!(" // size: {size}\n"))?;
					},
					_ => fmt.write_char('\n')?,
				}
			}
			fmt.write_str("}\n")
		},

		Type::Variant { variants, .. } => {
			fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
			fmt.write_str("pub enum ")?;
			fmt.write_str(&get_variant_type_name(_type))?;
			fmt.write_str(" {\n")?;
			for (i, field) in variants.iter().enumerate() {
				fmt.write_fmt(format_args!("\tVar{i}({}),\n", format_type_name(field)))?;
			}
			fmt.write_str("}\n")
		},
		
		other => fmt.write_fmt(format_args!("{other:#?}\n")),
	}
	
}

fn format_type_name<'a>(_type : &Type<'a>) -> Cow<'a, str> {
	match _type {
		Type::U8       => Cow::Borrowed("u8"),
		Type::U16      => Cow::Borrowed("u16"),
		Type::U32      => Cow::Borrowed("u32"),
		Type::U64      => Cow::Borrowed("u64"),
		Type::F32      => Cow::Borrowed("f32"),
		Type::F64      => Cow::Borrowed("f64"),
		Type::FileName => Cow::Borrowed("FileName"),
		Type::FileRef  => Cow::Borrowed("FileRef"),
		Type::Token    => Cow::Borrowed("Token"),
		Type::UUID     => Cow::Borrowed("UUID"),
		Type::CString { wide: false } => Cow::Borrowed("CString"), //turn into annotations?
		Type::CString { wide: true  } => Cow::Borrowed("WideCString"), //turn into annotations?
		Type::Reference { inner, kind: ReferenceKind::Optional } => Cow::Owned(format!("Option<{}>", format_type_name(inner))),
		Type::Reference { inner, .. } => format_type_name(inner),
		Type::Array { inner, kind: ArrayKind::Inline { size } } => Cow::Owned(format!("[{}; {size}]", format_type_name(inner))),
		Type::Array { inner, kind: ArrayKind::Pointers { .. } } => Cow::Owned(format!("Vec<Option<{}>>", format_type_name(inner))), //turn size into annotations?
		Type::Array { inner, .. } => {
			match inner.as_ref() {
				Type::U8 => Cow::Borrowed("&'a [u8]"), //turn size into annotations?
				_ => Cow::Owned(format!("Vec<{}>", format_type_name(inner))) //turn size into annotations?
			}
		},
		Type::Variant { .. } => {
			Cow::Owned(get_variant_type_name(_type))
		},
		Type::Composite { name, holds_input_references, .. } => {
			if *holds_input_references {
				Cow::Owned(format!("{}<'a>", name))
			}
			else {
				Cow::Borrowed(name)
			}
		},
	}
}

pub fn get_variant_type_name(_variant : &Type) -> String {
	let Type::Variant { holds_input_references, .. } = _variant else { unreachable!() };
	let hasher = &mut DefaultHasher::new();
	_variant.hash(hasher);
	let lifetime = if *holds_input_references { "<'a>" } else { "" };
	format!("Variant_{}{lifetime}", hasher.finish())
}

pub fn format_member_name<'a>(raw_name : &'a str) -> Cow<'a, str> {
	stringify!(type);
	let reserved = match raw_name {
		"as"       => "r#as",
		"break"    => "r#break",
		"const"    => "r#const",
		"continue" => "r#continue",
		"crate"    => "r#crate",
		"else"     => "r#else",
		"enum"     => "r#enum",
		"extern"   => "r#extern",
		"false"    => "r#false",
		"fn"       => "r#fn",
		"for"      => "r#for",
		"if"       => "r#if",
		"impl"     => "r#impl",
		"in"       => "r#in",
		"let"      => "r#let",
		"loop"     => "r#loop",
		"match"    => "r#match",
		"mod"      => "r#mod",
		"move"     => "r#move",
		"mut"      => "r#mut",
		"pub"      => "r#pub",
		"ref"      => "r#ref",
		"return"   => "r#return",
		"self"     => "r#self",
		"Self"     => "r#Self",
		"static"   => "r#static",
		"struct"   => "r#struct",
		"super"    => "r#super",
		"trait"    => "r#trait",
		"true"     => "r#true",
		"type"     => "r#type",
		"unsafe"   => "r#unsafe",
		"use"      => "r#use",
		"where"    => "r#where",
		"while"    => "r#while",
		"async"    => "r#async",
		"await"    => "r#await",
		"dyn"      => "r#dyn",
		"abstract" => "r#abstract",
		"become"   => "r#become",
		"box"      => "r#box",
		"do"       => "r#do",
		"final"    => "r#final",
		"macro"    => "r#macro",
		"override" => "r#override",
		"priv"     => "r#priv",
		"typeof"   => "r#typeof",
		"unsized"  => "r#unsized",
		"virtual"  => "r#virtual",
		"yield"    => "r#yield",
		"try"      => "r#try",

		other => {
			//could do case reformatting here
			return Cow::Borrowed(other);
		}
	};

	Cow::Borrowed(reserved)
}




pub fn add_required_imports_for_type_recursive<'a>(imports : &mut HashSet<&'a str>, _type : &Type<'a>) {
	match _type {
		Type::FileName => { imports.insert("FileName"); },
		Type::FileRef  => { imports.insert("FileRef"); },
		Type::Token    => { imports.insert("Token"); },
		Type::CString { wide: true }  => { imports.insert("WideCString"); },
		Type::CString { wide: false } => { imports.insert("CString"); },
		Type::Array { inner, .. } |
		Type::Reference { inner, .. } => { add_required_imports_for_type_recursive(imports, inner) },
		Type::Variant { variants, .. } => {
			for variant in variants {
				add_required_imports_for_type_recursive(imports, variant);
			}
		}
		Type::Composite { fields, .. } =>  {
			for field in fields {
				add_required_imports_for_type_recursive(imports, field);
			}
		},
		_ => {},
	}
}



use std::{borrow::Cow, collections::HashSet, fmt::{Formatter, Result as FmtResult, Write}, hash::{DefaultHasher, Hash, Hasher}};
use crate::structure::{ArrayKind, Chunk, ReferenceKind, Type};

