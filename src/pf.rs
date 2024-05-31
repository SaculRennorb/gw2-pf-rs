pub trait PackFile where Self : Sized + Default {
	const MAGIC : u32;

	fn from_bytes<'inp>(bytes : &'inp [u8]) -> Result<Self, crate::deserializer::Error> {
		let header : Header = unsafe{ std::ptr::read(bytes.as_ptr().cast()) };
		if header.magic != PF_MAGIC { return Err(crate::deserializer::Error::InvalidFileType { expected: PF_MAGIC as u32, actual: header.magic as u32 }); }
		if header.file_type != Self::MAGIC { return Err(crate::deserializer::Error::InvalidFileType { expected: Self::MAGIC, actual: header.file_type }) }

		let mut data = &bytes[header.header_size as usize..];

		let mut me = Self::default();

		while data.len() > std::mem::size_of::<ChunkHeader>() {
			let chunk_header : ChunkHeader = unsafe{ std::ptr::read(data.as_ptr().cast()) };
			me.parse_chunk(&chunk_header, &data[std::mem::size_of::<ChunkHeader>()..][..chunk_header.desc_offset as usize])?;

			let next_offset = 8 + chunk_header.next_chunk_offset as usize;  // no clue where the +8 comes from
			if next_offset > data.len() { break }
			data = &data[next_offset..];
		}


		Ok(me)
	}

	fn parse_chunk(&mut self, chunk_header : &ChunkHeader, data : &[u8]) -> Result<(), crate::deserializer::Error>;
}



#[repr(C)]
pub struct Header {
	pub magic       : u16,
	pub version     : u16,
	   _reserved    : u16,
	pub header_size : u16,
	pub file_type   : u32,
}

pub const PF_MAGIC       : u16 = 0x4650;

#[repr(C)]
pub struct ChunkHeader {
	pub magic             : u32,
	pub next_chunk_offset : u32,
	pub version           : u16,
	pub size              : u16,
	pub desc_offset       : u32,
}
