use std::mem::Discriminant;

#[derive(Debug)]
pub struct Chunk<'a> {
	pub magic    : &'a str, //can be 4 or 3 bytes long
	pub holds_input_references : bool,
	pub versions : Vec<SpecificChunkVersion<'a>>,
}

#[derive(Debug)]
pub struct SpecificChunkVersion<'a> {
	pub version : u32,
	pub root : Type<'a>,
}

impl<'a> std::ops::Deref for SpecificChunkVersion<'a> {
	type Target = Type<'a>;
	fn deref(&self) -> &Self::Target { &self.root }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Type<'a> {
	U8,
	U16,
	U32,
	U64,
	F32,
	F64,
	FileName,
	FileRef,
	Token,
	UUID,
	CString     { wide : bool },
	Reference   { kind : ReferenceKind, inner : Box<Type<'a>> },
	Array       { kind : ArrayKind, inner : Box<Type<'a>> },
	Variant     { variants : Vec<Type<'a>>, holds_input_references : bool },
	Composite   { name : &'a str, fields : Vec<Field<'a>>, holds_input_references : bool }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ReferenceKind { Default, Inline, StructCommon }
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ArrayKind {
	/// I have no idea why this has a size field. Seems to always be 0
	Dynamic      { size: usize },
	/// I have no idea why this has a size field. Seems to always be 0
	DynamicSmall { size: usize },
	/// I have no idea why this has a size field. Seems to always be 0
	Pointers     { size: usize },
	Inline  { size: usize },
	Fixed   { size: usize },
}

impl<'a> Type<'a> {
	pub fn inline_array(inner : Type<'a>, size : usize) -> Self {
		Self::Array { kind: ArrayKind::Inline { size }, inner: Box::new(inner) }
	}

	pub fn holds_input_references(&self) -> bool {
		match self {
			Type::CString {..} => true,
			Type::Reference { inner, .. } => inner.holds_input_references(),
			//NOTE(Rennorb): Byte arrays should not be copied over, their derserialize should just hold a pointer to the original data. 
			Type::Array { inner, .. }  => matches!(inner.as_ref(), Type::U8) || inner.holds_input_references(),
			Type::Variant { holds_input_references, .. } |
			Type::Composite { holds_input_references, .. } => *holds_input_references,
			_ => false,
		}
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Field<'a> {
	pub name : &'a str,
	pub _type : Type<'a>,
}

impl<'a> std::ops::Deref for Field<'a> {
	type Target = Type<'a>;
	fn deref(&self) -> &Self::Target { &self._type }
}