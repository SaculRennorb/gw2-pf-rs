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
		me.append(seed_type);
		me.append_children(seed_type);
		me
	}

	fn append_children(&mut self, _type : &'b Type<'a>) {
		match _type {
				Type::Reference { inner, .. } |
				Type::Array { inner, .. } => {
					if !is_primitive_type(inner) {
						self.append(inner);
						self.append_children(inner);
					}
				}
				Type::Variant { variants, .. } => {
					for inner in variants {
						if !is_primitive_type(inner) {
							self.append(inner);
							self.append_children(inner);
						}
					}
				}
				Type::Composite { fields, .. } =>  {
					for field in fields {
						if !is_primitive_type(field) {
							self.append(field);
							self.append_children(field);
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
			let longest_name_len = fields.iter().map(|f| f.name.len()).max().unwrap_or(0); 

			fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
			fmt.write_str("pub struct ")?;
			fmt.write_str(name)?;
			if *holds_input_references { fmt.write_str("<'a>")?; }
			fmt.write_str(" {\n")?;
			for field in fields.iter() {
				fmt.write_char('\t')?;
				fmt.write_str(field.name)?;
				let mut padding = longest_name_len.saturating_sub(field.name.len());
				while padding > 0 {
					fmt.write_char(' ')?;
					padding -= 1;
				}
				fmt.write_str(" : ")?;
				fmt.write_str(&format_type_name(field))?;
				match field._type {
					Type::Array { kind: ArrayKind::Dynamic { size }, .. } |
					Type::Array { kind: ArrayKind::DynamicSmall { size }, .. } |
					Type::Array { kind: ArrayKind::Pointers { size }, .. } if size > 0 => {
						fmt.write_fmt(format_args!(", // size: {size}\n"))?;
					},
		
					_ => fmt.write_str(",\n")?,
				}
			}
			fmt.write_str("}\n")
		},
		_=> Ok(()),
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
		Type::Reference { inner, .. } => format_type_name(inner),
		Type::Array { inner, kind: ArrayKind::Inline { size } } => Cow::Owned(format!("[{}; {size}]", format_type_name(inner))),
		Type::Array { inner, kind: ArrayKind::Pointers { .. } } => Cow::Owned(format!("Vec<*{}>", format_type_name(inner))), //turn size into annotations?
		Type::Array { inner, .. } => {
			match inner.as_ref() {
				Type::U8 => Cow::Borrowed("&'a [u8]"), //turn size into annotations?
				_ => Cow::Owned(format!("Vec<{}>", format_type_name(inner))) //turn size into annotations?
			}
		},
		Type::Variant { holds_input_references, .. } => {
			let hasher = &mut DefaultHasher::new();
			_type.hash(hasher);
			let lifetime = if *holds_input_references { "<'a>" } else { "" };
			Cow::Owned(format!("Variant_{}{lifetime}", hasher.finish()))
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

pub fn add_required_imports_for_type<'a>(imports : &mut HashSet<&'a str>, _type : &Type<'a>) {
	match _type {
		Type::FileName => { imports.insert("FileName"); },
		Type::FileRef  => { imports.insert("FileRef"); },
		Type::Token    => { imports.insert("Token"); },
		Type::CString { wide: true }  => { imports.insert("WideCString"); },
		Type::CString { wide: false } => { imports.insert("CString"); },
		Type::Array { inner, .. } |
		Type::Reference { inner, .. } => { add_required_imports_for_type(imports, inner) },
		Type::Variant { variants, .. } => {
			for variant in variants {
				add_required_imports_for_type(imports, variant);
			}
		}
		Type::Composite { fields, .. } =>  {
			for field in fields {
				add_required_imports_for_type(imports, field);
			}
		},
		_ => {},
	}
}



use std::{borrow::Cow, collections::HashSet, fmt::{Formatter, Result as FmtResult, Write}, hash::{DefaultHasher, Hash, Hasher}};
use crate::structure::{ArrayKind, Chunk, Type};

