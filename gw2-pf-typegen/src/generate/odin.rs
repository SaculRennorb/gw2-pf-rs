pub fn export_chunk<'a>(chunk : &Chunk<'a>, fmt : &mut Formatter) -> FmtResult {
	fmt.write_str("Chunk :: union #no_nil {\n")?;
	for version in chunk.versions.iter() {
		fmt.write_fmt(format_args!("\t v{}.Chunk,\n", version.version))?;
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

	pub fn iter(&self) -> core::slice::Iter<'_, &'b Type<'a>> {
		self.queue.iter()
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

pub fn export_type<'a>(_type : &Type<'a>, fmt : &mut Formatter) -> FmtResult {
	match _type {
		Type::Composite { name, fields, .. } => {
			let longest_name_len = fields.iter().map(|f| format_member_name(f.name).len()).max().unwrap_or(0); 

			fmt.write_str(name)?;
			fmt.write_str(" :: struct {\n")?;
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
			fmt.write_str(&get_variant_type_name(_type))?;
			fmt.write_str(" :: union {\n")?;
			for field in variants {
				fmt.write_fmt(format_args!("\t{},\n", format_type_name(field)))?;
			}
			fmt.write_str("}\n")
		},
		
		other => fmt.write_fmt(format_args!("{other:#?}\n")),
	}
	
}

pub fn export_type_parser<'a>(_type : &Type<'a>, fmt : &mut Formatter) -> FmtResult {

	fn format_field_parse_expression<'a>(fmt: &mut Formatter, field: &Field<'a>) -> FmtResult {
		match &field._type {
			Type::Array { kind: ArrayKind::Inline { .. }, inner } if inner.is_compact() => {
				fmt.write_fmt(format_args!("\tpf.read(reader, &destination.{})", format_member_name(field.name)))?;
			},
			Type::Array { inner, .. } if inner.is_compact() => {
				fmt.write_fmt(format_args!("\tpf.read_slice_packed(reader, &destination.{})", format_member_name(field.name)))?;
			},
			Type::Array { inner, .. } => {
				fmt.write_fmt(format_args!("\tpf.read(reader, &destination.{}, read_{})", format_member_name(field.name), format_type_name(inner)))?;
			},
			_ => {
				fmt.write_fmt(format_args!("\tpf.read(reader, &destination.{})", format_member_name(field.name)))?;
			},
		}
		Ok(())
	}

	match _type {
		Type::Composite { name, fields, .. } => {
			fmt.write_fmt(format_args!("read_{name} :: proc(reader : ^pf.Reader, destination : ^{name}) -> (ok : bool)\n{{\n"))?;
			if fields.len() == 1 {
				fmt.write_str("\treturn ")?;
				format_field_parse_expression(fmt, &fields[0])?;
				fmt.write_char('\n')?;
			}
			else {
				for field in fields.iter() {
					format_field_parse_expression(fmt, field)?;
					fmt.write_str(" or_return\n")?;
				}
				fmt.write_str("\treturn true\n")?;
			}
			fmt.write_str("}\n")
		},

		Type::Variant { variants, .. } => {
			let name = get_variant_type_name(_type);
			fmt.write_fmt(format_args!("read_{name} :: proc(reader : ^pf.Reader, destination : ^{name}) -> (ok : bool)\n{{\n"))?;
			fmt.write_str("\ttag, variant_reader := pf.read_variant_tag(reader) or_return\n")?;
			fmt.write_str("\tswitch(tag) {\n")?;
			let variants_might_need_alignemnt = variants.len() > 9;
			for (i, variant) in variants.iter().enumerate() {
				let vpad = if variants_might_need_alignemnt && i < 10 { " " } else { "" };
				fmt.write_fmt(format_args!("\t\tcase {vpad}{i}: variant : {}; if pf.read(&variant_reader, ) {{ destination^ =  variant; return }},\n", format_type_name(variant)))?;
			}
			fmt.write_str(r#"		case:
			return common.UnknownVersion {
				offset = u64(uintptr(reader.cursor) - uintptr(reader.begin)),
				actual = version,
			}
	}

	return common.OutOfData { offset = u64(uintptr(reader.cursor) - uintptr(reader.begin)) }
}
"#)?;
			fmt.write_str("\t}\n")?;
			fmt.write_str("}\n")
		},
		
		other => fmt.write_fmt(format_args!("{other:#?}\n")),
	}
	
}

pub fn format_type_name<'a>(_type : &Type<'a>) -> Cow<'a, str> {
	match _type {
		Type::U8       => Cow::Borrowed("u8"),
		Type::U16      => Cow::Borrowed("u16"),
		Type::U32      => Cow::Borrowed("u32"),
		Type::U64      => Cow::Borrowed("u64"),
		Type::F32      => Cow::Borrowed("f32"),
		Type::F64      => Cow::Borrowed("f64"),
		Type::FileName => Cow::Borrowed("pf.FileName"),
		Type::FileRef  => Cow::Borrowed("pf.FileRef"),
		Type::Token    => Cow::Borrowed("pf.Token64"),
		Type::UUID     => Cow::Borrowed("UUID"),
		Type::CString { wide: false } => Cow::Borrowed("string"), //turn into annotations?
		Type::CString { wide: true  } => Cow::Borrowed("string16"), //turn into annotations?
		Type::Reference { inner, kind: ReferenceKind::Optional } => Cow::Owned(format!("Maybe({})", format_type_name(inner))),
		Type::Reference { inner, .. } => format_type_name(inner),
		Type::Array { inner, kind: ArrayKind::Inline { size } } => Cow::Owned(format!("[{size}]{}", format_type_name(inner))),
		Type::Array { inner, kind: ArrayKind::Fixed { size } } => Cow::Owned(format!("[{size}]{}", format_type_name(inner))),
		Type::Array { inner, kind: ArrayKind::Pointers { .. } } => Cow::Owned(format!("[]Maybe({})", format_type_name(inner))), //turn size into annotations?
		Type::Array { inner, .. } => {
			match inner.as_ref() {
				Type::U8 => Cow::Borrowed("[]byte"),
				_ => Cow::Owned(format!("[]{}", format_type_name(inner)))
			}
		},
		Type::Variant { .. } => {
			Cow::Owned(get_variant_type_name(_type))
		},
		Type::Composite { name, .. } => {
			Cow::Borrowed(name)
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
		//"import"      => "import_",
		//"foreign"     => "foreign_",
		//"package"     => "package_",
		//"typeid"      => "typeid_",
		//"when"        => "when_",
		//"where"       => "where_",
		//"if"          => "if_",
		//"else"        => "else_",
		//"for"         => "for_",
		//"switch"      => "switch_",
		//"in"          => "in_",
		//"not_in"      => "not_in_",
		//"do"          => "do_",
		//"case"        => "case_",
		//"break"       => "break_",
		//"continue"    => "continue_",
		//"fallthrough" => "fallthrough_",
		//"defer"       => "defer_",
		//"return"      => "return_",
		//"proc"        => "proc_",
		//"struct"      => "struct_",
		//"union"       => "union_",
		//"enum"        => "enum_",
		//"bit_set"     => "bit_set_",
		//"bit_field"   => "bit_field_",
		//"map"         => "map_",
		//"dynamic"     => "dynamic_",
		//"auto_cast"   => "auto_cast_",
		//"cast"        => "cast_",
		//"transmute"   => "transmute_",
		//"distinct"    => "distinct_",
		//"using"       => "using_",
		//"context"     => "context_",
		//"or_else"     => "or_else_",
		//"or_return"   => "or_return_",
		//"or_break"    => "or_break_",
		//"or_continue" => "or_continue_",
		//"asm"         => "asm_",
		//"inline"      => "inline_",
		//"no_inline"   => "no_inline_",
		//"matrix"      => "matrix_",

		other => {
			//could do case reformatting here
			return Cow::Borrowed(other);
		}
	};

	Cow::Borrowed(reserved)
}




pub fn add_required_imports_for_type_recursive<'a>(imports : &mut HashSet<&'a str>, _type : &Type<'a>) {
	match _type {
		Type::FileName => { imports.insert("pf"); },
		Type::FileRef  => { imports.insert("pf"); },
		Type::Token    => { imports.insert("pf"); },
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
use crate::structure::{ArrayKind, Chunk, Field, ReferenceKind, Type};

