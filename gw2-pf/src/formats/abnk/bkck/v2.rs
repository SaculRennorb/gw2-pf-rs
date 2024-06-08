#[derive(Debug, crate::Parse)]
pub struct BankFileData<'a> {
	   _reserved1     : u32,
	   _reserved2     : u32,
	   _reserved3     : u32,
	   _reserved4     : u32,
	pub files         : Vec<ASNDFile<'a>>,
	   _reserved_data : u32,
}

#[derive(Debug, crate::Parse)]
pub struct ASNDFile<'a> {
	pub voice_id   : u32,
	pub flags      : u32,
	   _reserved1  : u32,
	   _reserved2  : u32,
	   _reserved3  : u32,
	   _reserved4  : u32,
	pub length     : f32,
	pub offset     : f32,
	   _reserved5  : u8,
	   _reserved6  : u8,
	   _reserved7  : u8,
	   _reserved8  : u8,
	pub audio_data : &'a [u8], // crate::formats::asnd::ASND
}
