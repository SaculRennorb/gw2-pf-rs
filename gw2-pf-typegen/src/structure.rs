#[derive(Debug)]
pub struct Chunk<'a> {
	pub magic    : &'a str, //can be 4 or 3 bytes long
	pub versions : Vec<SpecificChunkVersion<'a>>,
}

#[derive(Debug)]
pub struct SpecificChunkVersion<'a> {
	pub version : u32,
	pub root : Type<'a>,
}

#[derive(Debug)]
pub struct Type<'a> {
	pub name   : &'a str,
	pub fields : Vec<Field<'a>>,
}

#[derive(Debug)]
pub struct Field<'a> {
	pub name : &'a str,
	pub detail : FieldDetail<'a>,
}

impl<'a> std::ops::Deref for Field<'a> {
	type Target = FieldDetail<'a>;
	fn deref(&self) -> &Self::Target { &self.detail }
}


#[derive(Debug)]
pub enum FieldDetail<'a> {
	End,
	FixedArray  { size : usize, inner : Type<'a> },
	Array       { size : usize, inner : Type<'a> },
	PtrArray    { size : usize, inner : Type<'a> },
	Byte,
	Byte4,
	Double,
	DoubleWord,
	FileName,
	Float,
	Float2,
	Float3,
	Float4,
	Reference {},
	QuadWord,
	WideCString,
	CString,
	Inline {},
	Word,
	UUID,
	Byte3,
	DoubleWord2,
	DoubleWord4,
	DoubleWord3,
	FileRef,
	Variant {},
	StructCommon {},
	SmallArray  { size : usize, inner : Type<'a> },
}