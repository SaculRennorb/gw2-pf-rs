use crate::parse::{Error, Input, Result};

pub trait PackFile where Self : Sized + Default {
	const MAGIC : u32;

	fn from_bytes<'inp>(bytes : &'inp [u8]) -> Result<Self> {
		let header : PFHeader = unsafe{ std::ptr::read(bytes.as_ptr().cast()) };
		if header.magic != PF_MAGIC { return Err(Error::InvalidFileType { expected: PF_MAGIC as u32, actual: header.magic as u32 }); }
		if header.file_type != Self::MAGIC { return Err(Error::InvalidFileType { expected: Self::MAGIC, actual: header.file_type }) }

		let input = &mut Input{ remaining: &bytes[header.header_size as usize..], is_64_bit: header.flags & PF_FLAG_HAS_64BIT_PTRS != 0 };

		let mut me = Self::default();

		while input.remaining.len() > std::mem::size_of::<ChunkHeader>() {
			let chunk_header : ChunkHeader = unsafe{ std::ptr::read(input.remaining.as_ptr().cast()) };
			let chunk_data = &input.remaining[chunk_header.chunk_header_size as usize..][..chunk_header.descriptor_offset as usize];
			let chunk_input = &mut Input { remaining: chunk_data, is_64_bit: input.is_64_bit };
			me.parse_chunk(&chunk_header, chunk_input)?;

			let next_offset = 8 + chunk_header.next_chunk_offset as usize;  // no clue where the +8 comes from
			if next_offset > input.remaining.len() { break }
			input.remaining = &input.remaining[next_offset..];
		}


		Ok(me)
	}

	fn parse_chunk(&mut self, chunk_header : &ChunkHeader, data : &mut Input) -> Result<()>;
}

#[repr(C)]
#[derive(serde::Deserialize)]
pub struct PFHeader {
	pub magic       : u16,
	pub version     : u16,
	pub flags       : u16,
	pub header_size : u16,
	pub file_type   : u32,
}

pub const PF_FLAG_HAS_64BIT_PTRS : u16 = 1 << 2;
pub const PF_MAGIC : u16 = crate::tcc(b"PF");


pub trait Chunk : Sized {
	const MAGIC : u32;
}

#[repr(C)]
#[derive(serde::Deserialize)]
pub struct ChunkHeader {
	pub magic             : u32,
	pub next_chunk_offset : u32,
	pub version           : u16,
	pub chunk_header_size : u16,
	pub descriptor_offset : u32,
}
