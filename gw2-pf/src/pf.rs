use crate::parse::{ChunkIter, Error, Input, ParseMagicVariant, Result};

pub struct PackFileReader<'inp, F : Magic + ParseMagicVariant<'inp>> {
	_p : std::marker::PhantomData<&'inp F>,
}

impl<'inp, F : Magic + ParseMagicVariant<'inp>> PackFileReader<'inp, F> {
	pub fn from_bytes(bytes : &'inp [u8]) -> Result<ChunkIter<F>> {
		if bytes.len() < std::mem::size_of::<PFHeader>() { return Err(Error::to_short::<PFHeader>(bytes.len())) }

		let header = unsafe{ bytes.as_ptr().cast::<PFHeader>().as_ref().unwrap() };
		if header.magic != PF_MAGIC { return Err(Error::InvalidFileType { r#type: std::any::type_name::<PFHeader>(), expected: PF_MAGIC as u32, actual: header.magic as u32 }); }
		if header.file_type != F::MAGIC { return Err(Error::wrong_magic::<F>(header.file_type)) }

		let input = Input{ remaining: &bytes[header.header_size as usize..], is_64_bit: header.flags & PF_FLAG_HAS_64BIT_PTRS != 0 };

		Ok(ChunkIter{ input, _p : std::marker::PhantomData })
	}
}

#[repr(C)]
pub struct PFHeader {
	pub magic       : u16,
	pub flags       : u16,
	_reserved       : u16,
	pub header_size : u16,
	pub file_type   : u32,
}

pub const PF_FLAG_HAS_64BIT_PTRS : u16 = 1 << 2;
pub const PF_MAGIC : u16 = crate::tcc(b"PF");


pub trait Magic : Sized {
	const MAGIC : u32;
}

#[repr(C)]
pub struct ChunkHeader {
	pub magic             : u32,
	pub next_chunk_offset : u32,
	pub version           : u16,
	pub chunk_header_size : u16,
	pub descriptor_offset : u32,
}
