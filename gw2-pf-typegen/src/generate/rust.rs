pub fn export_chunk<'a>(chunk : &Chunk<'a>, fmt : &mut Formatter) -> FmtResult {
	fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
	fmt.write_str("#[chunk]\n")?;
	fmt.write_str("pub enum ")?;
	fmt.write_str(chunk.magic)?;
	if chunk.holds_input_references { fmt.write_str("<'a>")?; }
	fmt.write_str(" {\n")?;
	for version in chunk.versions.iter() {
		fmt.write_fmt(format_args!("\t#[v({ver})] V{ver}(v{ver}::{}),\n", translate_type_name(&version.root), ver = version.version))?;
	}
	fmt.write_str("}\n")
}

pub struct RecursiveTypeReferences<'a, 'b> {
	already_exported : HashSet<&'a str>,
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
		for field in _type.fields.iter() {
			match &field.detail {
				FieldDetail::Array        { inner, .. } | 
				FieldDetail::PtrArray     { inner, .. } | 
				FieldDetail::FixedArray   { inner, .. } | 
				FieldDetail::SmallArray   { inner, .. } |
				FieldDetail::Inline       { inner }     | 
				FieldDetail::Reference    { inner }     |
				FieldDetail::StructCommon { inner }     =>  {
					if !is_primitive_type(inner) {
						self.append(inner);
						self.append_children(inner);
					}
				},
				FieldDetail::Variant { variants } => {
					for inner in variants {
						self.append(inner);
						self.append_children(inner);
					}
				},
				_ => {},
			}
		}
	}

	pub fn append(&mut self, _type : &'b Type<'a>) {
		if !self.already_exported.contains(_type.name) {
			self.queue.push(_type);
		}
	}
}

impl<'a, 'b, 'c> std::iter::IntoIterator for &'c RecursiveTypeReferences<'a, 'b> {
		type Item = &'c &'b Type<'a>;
		type IntoIter = core::slice::Iter<'c, &'b Type<'a>>;
		fn into_iter(self) -> Self::IntoIter { self.queue.iter() }
}

pub fn export_struct<'a>(struct_type : &Type<'a>, fmt : &mut Formatter) -> FmtResult {
	let longest_name_len = struct_type.fields.iter().map(|f| f.name.len()).max().unwrap_or(0); 

	fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
	fmt.write_str("pub struct ")?;
	fmt.write_str(struct_type.name)?;
	if struct_type.holds_input_references { fmt.write_str("<'a>")?; }
	fmt.write_str(" {\n")?;
	for field in struct_type.fields.iter() {
		fmt.write_char('\t')?;
		fmt.write_str(field.name)?;
		let mut padding = longest_name_len.saturating_sub(field.name.len());
		while padding > 0 {
			fmt.write_char(' ')?;
			padding -= 1;
		}
		fmt.write_str(" : ")?;
		fmt.write_str(&format_field_type(field))?;
		match field.detail {
			FieldDetail::Array { size, .. } |
			FieldDetail::PtrArray { size, .. } | 
			FieldDetail::SmallArray { size, .. } if size > 0 => {
				fmt.write_fmt(format_args!(", // size: {size}\n"))?;
			},

			_ => fmt.write_str(",\n")?,
		}
	}
	fmt.write_str("}\n")
}

fn translate_type_name<'a>(_type : &Type<'a>) -> Cow<'a, str> {
	match _type.name {
		"byte"     => Cow::Borrowed("u8"),
		"dword"    => Cow::Borrowed("u32"),
		"word"     => Cow::Borrowed("u16"),
		"qword"    => Cow::Borrowed("u64"),
		"float"    => Cow::Borrowed("f32"),
		"double"   => Cow::Borrowed("f64"),
		"byte3"    => Cow::Borrowed("[u8; 3]"),
		"byte4"    => Cow::Borrowed("[u8; 4]"),
		"word3"    => Cow::Borrowed("[u16; 3]"),
		"dword4"   => Cow::Borrowed("[u32; 4]"),
		"float2"   => Cow::Borrowed("[f32; 2]"),
		"float3"   => Cow::Borrowed("[f32; 3]"),
		"float4"   => Cow::Borrowed("[f32; 4]"),
		"filename" => Cow::Borrowed("filename ?"),
		"fileref"  => Cow::Borrowed("fielref ?"),
		"token"    => Cow::Borrowed("Token"),
		"wchar *"  => Cow::Borrowed("WideCString"),
		"char *"   => Cow::Borrowed("CString"),
		_ => {
			if _type.holds_input_references {
				Cow::Owned(format!("{}<'a>", _type.name))
			}
			else {
				Cow::Borrowed(_type.name)
			}
		},
	}
}


fn format_field_type<'a>(field : &Field<'a>) -> Cow<'a, str> {
	match &field.detail {
		FieldDetail::FixedArray{inner, size} => Cow::Owned(format!("[{}; {size}]", translate_type_name(inner))),
		FieldDetail::SmallArray{inner, ..}   |
		FieldDetail::Array{inner, ..}        => {
			if inner.name == "byte" {
				Cow::Borrowed("&'a [u8]") // special case since we don't want to copy byte slices
			}
			else {
				Cow::Owned(format!("Vec<{}>", translate_type_name(inner)))
			}
		},
		FieldDetail::PtrArray{inner, ..}     => Cow::Owned(format!("Vec<&{}>", translate_type_name(inner))),
		FieldDetail::Reference{inner}        |
		FieldDetail::Inline{inner}           |
		FieldDetail::StructCommon{inner}     => translate_type_name(inner),
		FieldDetail::Byte                    => Cow::Borrowed("u8"),
		FieldDetail::Word                    => Cow::Borrowed("u16"),
		FieldDetail::DoubleWord              => Cow::Borrowed("u32"),
		FieldDetail::QuadWord                => Cow::Borrowed("u64"),
		FieldDetail::Float                   => Cow::Borrowed("f32"),
		FieldDetail::Double                  => Cow::Borrowed("f64"),
		FieldDetail::Byte3                   => Cow::Borrowed("[u8; 3]"),
		FieldDetail::Byte4                   => Cow::Borrowed("[u8; 4]"),
		FieldDetail::DoubleWord2             => Cow::Borrowed("[u32; 2]"),
		FieldDetail::DoubleWord4             => Cow::Borrowed("[u32; 4]"),
		FieldDetail::DoubleWord3             => Cow::Borrowed("[u32; 3]"),
		FieldDetail::Float2                  => Cow::Borrowed("[f32; 2]"),
		FieldDetail::Float3                  => Cow::Borrowed("[f32; 3]"),
		FieldDetail::Float4                  => Cow::Borrowed("[f32; 4]"),
		FieldDetail::FileName                => Cow::Borrowed("cant represent filename"),
		FieldDetail::FileRef                 => Cow::Borrowed("cant represent fileref"),
		FieldDetail::WideCString             => Cow::Borrowed("WString"),
		FieldDetail::CString                 => Cow::Borrowed("CString"),
		FieldDetail::UUID                    => Cow::Borrowed("UUID"),
		FieldDetail::Variant{..}             => {
			let hasher = &mut DefaultHasher::default();
			field.hash(hasher);
			let lifetime = if field.holds_input_references { "<'a>" } else { "" };
			Cow::Owned(format!("Variant_{}{lifetime}", hasher.finish()))
		},
		FieldDetail::End                     => unreachable!(),
	}
}

fn is_primitive_type(_type : &Type) -> bool {
	match _type.name {
		"byte"     |
		"byte3"    |
		"byte4"    |
		"double"   |
		"dword"    |
		"dword4"   |
		"filename" |
		"fileref"  |
		"float"    |
		"float2"   |
		"float3"   |
		"float4"   |
		"qword"    |
		"token"    |
		"wchar *"  |
		"char *"   |
		"word"     |
		"word3"    => true,
		_ => false,
	}
}

pub fn add_required_imports_for_type<'a>(imports : &mut HashSet<&'a str>, _type : &Type<'a>) {
	match _type.name {
		"filename" => { imports.insert("cant import filename"); },
		"fileref"  => { imports.insert("cant import fielref"); },
		"token"    => { imports.insert("Token"); },
		"wchar *"  => { imports.insert("WideCString"); },
		"char *"   => { imports.insert("CString"); },
		_ => {
			for field in _type.fields.iter() {
				add_required_imports_for_field(imports, field);
			}
		},
	}
}

pub fn add_required_imports_for_field<'a>(imports : &mut HashSet<&'a str>, field : &FieldDetail<'a>) {
	match field {
		FieldDetail::FixedArray{ref inner, ..} |
		FieldDetail::Array{ref inner, ..}      |
		FieldDetail::PtrArray{ref inner, ..}   |
		FieldDetail::SmallArray{ref inner, ..} |
		FieldDetail::Inline{ref inner}         |
		FieldDetail::Reference{ref inner}      |
		FieldDetail::StructCommon{ref inner}   => add_required_imports_for_type(imports, inner),
		FieldDetail::WideCString               => { imports.insert("WString"); },
		FieldDetail::CString                   => { imports.insert("CString"); },
		FieldDetail::UUID                      => { imports.insert("UUID"); },
		FieldDetail::FileName                  => { imports.insert("can't represent filename"); },
		FieldDetail::FileRef                   => { imports.insert("can't represent fileref"); },
		FieldDetail::Variant{ ref variants }   => {
			for inner in variants {
				add_required_imports_for_type(imports, inner);
			}
		},
		_ => {},
	}
}


use std::{borrow::{Borrow, Cow}, collections::HashSet, fmt::{Formatter, Result as FmtResult, Write}, hash::{DefaultHasher, Hash, Hasher}};
use crate::structure::{Chunk, Field, FieldDetail, Type};

