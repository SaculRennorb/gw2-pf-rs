#[derive(Debug, serde::Deserialize)]
pub struct BankFileData {
	   _reserved1     : u32,
	   _reserved2     : u32,
	   _reserved3     : u32,
	   _reserved4     : u32,
	pub files         : Vec<ASNDFile>,
	   _reserved_data : u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct ASNDFile {
	pub voice_id   : u32,
	pub flags      : u32,
	pub   _reserved1  : u32,
	pub   _reserved2  : u32,
	pub   _reserved3  : u32,
	pub   _reserved4  : u32,
	pub length     : f32,
	pub offset     : f32,
	pub   _reserved5  : u8,
	pub   _reserved6  : u8,
	pub   _reserved7  : u8,
	pub   _reserved8  : u8,
	pub audio_data : Vec<crate::formats::asnd::ASND>,
}