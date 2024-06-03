use crate::parse::{Error, Input, ParseMagicVariant, Result};

pub struct PackFileReader<'inp, C : Magic + ParseMagicVariant> {
	_p : std::marker::PhantomData<C>,
	input : Input<'inp>,
}

impl<'inp, F : Magic + ParseMagicVariant> PackFileReader<'inp, F> {
	pub fn from_bytes(bytes : &'inp [u8]) -> Result<Self> {
		let header : PFHeader = unsafe{ std::ptr::read(bytes.as_ptr().cast()) };
		if header.magic != PF_MAGIC { return Err(Error::InvalidFileType { expected: PF_MAGIC as u32, actual: header.magic as u32 }); }
		if header.file_type != F::MAGIC { return Err(Error::InvalidFileType { expected: F::MAGIC, actual: header.file_type }) }

		let input = Input{ remaining: &bytes[header.header_size as usize..], is_64_bit: header.flags & PF_FLAG_HAS_64BIT_PTRS != 0 };

		Ok(Self{ input, _p: std::marker::PhantomData })
	}
}

impl<'inp, C : Magic + ParseMagicVariant> Iterator for PackFileReader<'inp, C> {
	type Item = crate::parse::Result<C>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.input.remaining.len() < std::mem::size_of::<ChunkHeader>() { return None }

		let chunk_header : ChunkHeader = unsafe{ std::ptr::read(self.input.remaining.as_ptr().cast()) };
		let chunk_data = &self.input.remaining[chunk_header.chunk_header_size as usize..][..chunk_header.descriptor_offset as usize];
		let chunk_input = &mut Input { remaining: chunk_data, is_64_bit: self.input.is_64_bit };

		let next_offset = 8 + chunk_header.next_chunk_offset as usize;  // no clue where the +8 comes from
		if next_offset <= self.input.remaining.len() { self.input.remaining = &self.input.remaining[next_offset..]; }
		
		Some(C::parse(chunk_header.magic, chunk_header.version, chunk_input))
	}
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


pub trait Magic : Sized {
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
