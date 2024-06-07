pub fn export_chunk<'a>(chunk : &Chunk<'a>, fmt : &mut Formatter) -> FmtResult {
	fmt.write_str("#[derive(Debug, crate::Parse)]\n")?;
	fmt.write_str("#[chunk]\n")?;
	fmt.write_str("pub enum ")?;
	fmt.write_str(chunk.magic)?;
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
			match field.detail {
				FieldDetail::Array      { ref inner, .. } | 
				FieldDetail::PtrArray   { ref inner, .. } | 
				FieldDetail::FixedArray { ref inner, .. } | 
				FieldDetail::SmallArray { ref inner, .. } =>  {
					if !is_primitive_type(inner) {
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
		fmt.write_str(",\n")?;
	}
	fmt.write_str("}\n")
}

fn translate_type_name<'a>(_type : &Type<'a>) -> &'a str {
	match _type.name {
		"byte"     => "u8",
		"byte3"    => "[u8; 3]",
		"byte4"    => "[u8; 3]",
		"double"   => "f64",
		"dword"    => "u32",
		"dword4"   => "[u32; 4]",
		//"filename" => "??",
		//"fileref"  => "",
		"float"    => "f32",
		"float2"   => "[f32; 2]",
		"float3"   => "[f32; 3]",
		"float4"   => "[f32; 4]",
		"qword"    => "u64",
		"token"    => "Token",
		"wchar *"  => "WideCString",
		"char *"   => "CString",
		"word"     => "u16",
		"word3"    => "[u16; 3]",
		_ => _type.name,
	}
}


fn format_field_type<'a>(field : &FieldDetail<'a>) -> Cow<'a, str> {
	match field {
		FieldDetail::FixedArray{inner, size} => Cow::Owned(format!("[{}; {size}]", translate_type_name(inner))),
		FieldDetail::Array{inner, ..}        => Cow::Owned(format!("Vec<{}>", translate_type_name(inner))),
		FieldDetail::PtrArray{inner, ..}     => Cow::Owned(format!("Vec<&{}>", translate_type_name(inner))),
		FieldDetail::Byte                    => Cow::Borrowed("u8"),
		FieldDetail::Byte4                   => Cow::Borrowed("[u8; 4]"),
		FieldDetail::Double                  => Cow::Borrowed("f64"),
		FieldDetail::DoubleWord              => Cow::Borrowed("u32"),
		FieldDetail::FileName                => Cow::Borrowed("cant represent filename"),
		FieldDetail::Float                   => Cow::Borrowed("f32"),
		FieldDetail::Float2                  => Cow::Borrowed("[f32; 2]"),
		FieldDetail::Float3                  => Cow::Borrowed("[f32; 3]"),
		FieldDetail::Float4                  => Cow::Borrowed("[f32; 4]"),
		FieldDetail::Reference{..}           => Cow::Borrowed("can't represent reference"),
		FieldDetail::QuadWord                => Cow::Borrowed("u64"),
		FieldDetail::WideCString             => Cow::Borrowed("WString"),
		FieldDetail::CString                 => Cow::Borrowed("CString"),
		FieldDetail::Inline{..}              => Cow::Borrowed("can't represent inline"),
		FieldDetail::Word                    => Cow::Borrowed("u16"),
		FieldDetail::UUID                    => Cow::Borrowed("UUID"),
		FieldDetail::Byte3                   => Cow::Borrowed("[u8; 3]"),
		FieldDetail::DoubleWord2             => Cow::Borrowed("[u32; 2]"),
		FieldDetail::DoubleWord4             => Cow::Borrowed("[u32; 4]"),
		FieldDetail::DoubleWord3             => Cow::Borrowed("[u32; 3]"),
		FieldDetail::FileRef                 => Cow::Borrowed("cant represent fileref"),
		FieldDetail::Variant{..}             => Cow::Borrowed("can't represent variant"),
		FieldDetail::StructCommon{..}        => Cow::Borrowed("can't represent structcommon"),
		FieldDetail::SmallArray{inner, ..}   => Cow::Owned(format!("Vec<{}>", translate_type_name(inner))),
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

fn get_required_import_for_type<'a>(_type : &Type<'a>) -> Option<&'static str> {
	match _type.name {
		//"filename" => "??",
		//"fileref"  => "",
		"token"    => Some("Token"),
		"wchar *"  => Some("WideCString"),
		"char *"   => Some("CString"),
		_=> None,
	}
}

pub fn get_required_import_for_field<'a>(field : &FieldDetail<'a>) -> Option<&'static str> {
	match field {
		FieldDetail::FixedArray{ref inner, ..} => get_required_import_for_type(inner),
		FieldDetail::Array{ref inner, ..}      => get_required_import_for_type(inner),
		FieldDetail::PtrArray{ref inner, ..}   => get_required_import_for_type(inner),
		FieldDetail::FileName                  => Some("can't represent filename"),
		FieldDetail::Reference{..}             => Some("can't represent reference"),
		FieldDetail::WideCString               => Some("WString"),
		FieldDetail::CString                   => Some("CString"),
		FieldDetail::Inline{..}                => Some("can't represent inline"),
		FieldDetail::UUID                      => Some("UUID"),
		FieldDetail::FileRef                   => Some("can't represent fileref"),
		FieldDetail::Variant{..}               => Some("can't represent variant"),
		FieldDetail::StructCommon{..}          => Some("can't represent structcommon"),
		FieldDetail::SmallArray{ref inner, ..} => get_required_import_for_type(inner),
		_ => None,
	}
}


use std::{borrow::Cow, collections::{HashSet, VecDeque}, fmt::{Formatter, Result as FmtResult, Write}};
use crate::structure::{Chunk, FieldDetail, Type};

