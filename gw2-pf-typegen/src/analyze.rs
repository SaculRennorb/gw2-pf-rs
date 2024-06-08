pub fn locate_chunks<'a>(raw_exe : &'a [u8]) -> impl Iterator<Item = Chunk<'a>> {
	Parser{
		exe            : PE::parse_header(raw_exe),
		chunk_cache    : HashSet::new(),
		remaining_bytes: raw_exe,
	}
}


struct Parser<'a> {
	pub exe : PE<'a>,
	pub chunk_cache : HashSet<ChunkIdentifier<'a>>,
	remaining_bytes : &'a [u8],
}

impl<'a> std::iter::Iterator for Parser<'a> {
	type Item = Chunk<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		const ASCII_HI_BITS : u8 = 0b01000000;
		const ASCII_MASK    : u8 = 0b11000000;
		// four ascii bytes, e.g. ABIX, and u32 version that realistically doesn't use more than the first byte
		const TARGET : u64 = u64::from_le_bytes([ASCII_HI_BITS, ASCII_HI_BITS, ASCII_HI_BITS, ASCII_HI_BITS, 0, 0, 0, 0]);
		const MASK   : u64 = u64::from_le_bytes([ASCII_MASK, ASCII_MASK, ASCII_MASK, ASCII_MASK, 0, 0xff, 0xff, 0xff]);
		while self.remaining_bytes.len() >= 8 {
			let (chunk, rest) = self.remaining_bytes.split_at(8); // we are looking for aligned segments, so we can move in chunks of 8
			let value = u64::from_le_bytes(chunk.try_into().unwrap());
			if value & MASK == TARGET {
				// unfortunately we will catch a lot of special characters with just a mask match, so we filter for ascii chars again
				const _A : u8 = 'A' as u8;
				const _Z : u8 = 'Z' as u8;
				#[allow(non_upper_case_globals)]
				const _a : u8 = 'a' as u8;
				#[allow(non_upper_case_globals)]
				const _z : u8 = 'z' as u8;
				if (&chunk[..4]).iter().all(|c| matches!(*c, _A..=_Z | _a..=_z)) {
					if let Ok((chunk, len)) = self.parse_chunk(&mut Reader { remaining: self.remaining_bytes }) {
						self.remaining_bytes = &self.remaining_bytes[len..];
						return Some(chunk)
					}
				}
			}

			self.remaining_bytes = rest;
		}

		None
	}
}

impl<'a> Parser<'a> {
	pub fn parse_chunk(&mut self, input : &mut Reader<'a>) -> Result<(Chunk<'a>, usize)> {
		let initial_len = input.remaining.len();

		let magic_bytes = input.eat_slice(4).unwrap().remaining;
		let magic =  {
			let final_bytes = if magic_bytes[3] == 0 { &magic_bytes[..3] } else { magic_bytes };
			unsafe { from_utf8_unchecked(final_bytes) }
		};

		let n_versions = input.eat_u32()?;
		let meta_offset = input.eat_rva_as_offset(&self.exe)?;

		if !self.chunk_cache.insert(ChunkIdentifier { magic, max_version: n_versions, meta_offset }) {
			return Err(Error::DuplicateChunk { magic: magic_bytes.try_into().unwrap(), max_version: n_versions, meta_offset })
		}

		let mut versions = Vec::new();
		if meta_offset != 0 {
			let meta_input = &mut self.exe.reader_from_offset(meta_offset)?;
			versions = self.parse_chunk_versions(meta_input, n_versions)?;
			
		}
		
		if versions.is_empty() { return Err(Error::NoChunks) }

		let holds_input_references = versions.iter().any(|c| c.holds_input_references());
		let chunk = Chunk { magic, versions, holds_input_references };
	
		Ok((chunk, initial_len - input.remaining.len()))
	}
	
	pub fn parse_chunk_versions(&self, input : &mut Reader<'a>, n_versions : u32) -> Result<Vec<SpecificChunkVersion<'a>>> {
		let mut chunks = Vec::with_capacity(n_versions as usize);

		for version in 0..n_versions {
			let chunk_meta_header_input = &mut input.eat_slice(24)?;

			let chunk_offset = chunk_meta_header_input.eat_rva_as_offset(&self.exe)?;
			if chunk_offset == 0 { continue }

			let root_input = &mut self.exe.reader_from_offset(chunk_offset)?;
			let root = self.parse_type(root_input)?;

			let chunk = SpecificChunkVersion{ version, root };
			chunks.push(chunk);
		}

		Ok(chunks)
	}
	
	pub fn parse_type(&self, input : &mut Reader<'a>) -> Result<Type<'a>> {
		let mut fields = Vec::new();
		let mut holds_input_references = false;

		let name = loop {
			match self.prase_field(input)? { 
				FieldParseResult::Field(field) => {
					holds_input_references |= field.holds_input_references();
					fields.push(field);
				},
				FieldParseResult::TypeName(name) => {
					break name
				},
			}
		};

		Ok(match name {
			"byte"     => Type::U8,
			"dword"    => Type::U32,
			"word"     => Type::U16,
			"qword"    => Type::U64,
			"float"    => Type::F32,
			"double"   => Type::F64,
			"byte3"    => Type::inline_array(Type::U8, 3),
			"byte4"    => Type::inline_array(Type::U8, 4),
			"word3"    => Type::inline_array(Type::U16,3),
			"dword4"   => Type::inline_array(Type::U32,4),
			"float2"   => Type::inline_array(Type::F32,2),
			"float3"   => Type::inline_array(Type::F32,3),
			"float4"   => Type::inline_array(Type::F32,4),
			"filename" => Type::FileName,
			"fileref"  => Type::FileRef,
			"token"    => Type::Token,
			"char *"   => Type::CString { wide: false },
			"wchar *"  => Type::CString { wide: true },
			_ => Type::Composite { name, fields, holds_input_references },
		})
	}

	pub fn prase_field(&self, input : &mut Reader<'a>) -> Result<FieldParseResult<'a>> {
		let input = &mut input.eat_slice(32)?; // all fields are 32 byte

		use BuiltinFieldType as FT;
		let _type = input.eat_u16()?.try_into()?; // 0-2
		input.eat_u16()?; // 2-4
		input.eat_u32()?; // 4-8
		let name = self.exe.get_str_at(input.eat_rva_as_offset(&self.exe)?)?;

		let _type = match _type {
			FT::Byte        => Type::U8,
			FT::Word        => Type::U16,
			FT::DWord       => Type::U32,
			FT::QWord       => Type::U64,
			FT::Float       => Type::F32,
			FT::Double      => Type::F64,
			FT::FileName    => Type::FileName,
			FT::FileRef     => Type::FileRef,
			FT::Byte3       => Type::inline_array(Type::U8, 3),
			FT::Byte4       => Type::inline_array(Type::U8, 4),
			FT::Word3       => Type::inline_array(Type::U16, 3),
			FT::DWord2      => Type::inline_array(Type::U32, 2),
			FT::DWord4      => Type::inline_array(Type::U32, 4),
			FT::Float2      => Type::inline_array(Type::F32, 2),
			FT::Float3      => Type::inline_array(Type::F32, 3),
			FT::Float4      => Type::inline_array(Type::F32, 4),
			FT::UUID        => Type::UUID,
			FT::CString     => Type::CString{ wide: false },
			FT::WideCString => Type::CString { wide: true },
			FT::Array | FT::FixedArray | FT::SmallArray | FT::PtrArray => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;
				let inner = self.parse_type(&mut self.exe.reader_from_offset(offset)?)?;
				match _type {
					FT::FixedArray => Type::Array { inner: Box::new(inner), kind: ArrayKind::Fixed        { size }},
					FT::Array      => Type::Array { inner: Box::new(inner), kind: ArrayKind::Dynamic      { size }},
					FT::SmallArray => Type::Array { inner: Box::new(inner), kind: ArrayKind::DynamicSmall { size }},
					FT::PtrArray   => Type::Array { inner: Box::new(inner), kind: ArrayKind::Pointers     { size }},
					_ => unreachable!()
				}
			},
			FT::Reference | FT::Inline | FT::StructCommon => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let inner = self.parse_type(&mut self.exe.reader_from_offset(offset)?)?;
				match _type {
					FT::Reference    => Type::Reference { inner: Box::new(inner), kind: ReferenceKind::Optional },
					FT::Inline       => Type::Reference { inner: Box::new(inner), kind: ReferenceKind::Inline },
					FT::StructCommon => Type::Reference { inner: Box::new(inner), kind: ReferenceKind::StructCommon },
					_ => unreachable!()
				}
			},
			FT::Variant => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;

				let input = &mut self.exe.reader_from_offset(offset)?;
				let mut holds_input_references = false;
				let mut variants = Vec::with_capacity(size);
				for _i in 0..size {
					let variant_offset = input.eat_rva_as_offset(&self.exe)?;
					let input = &mut self.exe.reader_from_offset(variant_offset)?;
					let inner = self.parse_type(input)?;
					holds_input_references |= inner.holds_input_references();
					variants.push(inner);
				}
				Type::Variant { variants, holds_input_references }
			},
			FT::End => return Ok(FieldParseResult::TypeName(name)),
		};

		Ok(FieldParseResult::Field(Field { name, _type }))
	}
}

/// The last field in a type contains its name and terminates field iteration
pub enum FieldParseResult<'a> {
	Field(Field<'a>),
	TypeName(&'a str),
}

macro_rules! impl_try_from {
	{ enum $type:ident { $($name:ident = $value:literal),* $(,)? } } => {
		enum $type {
			$($name = $value),*
		}

		impl TryFrom<u16> for $type {
			type Error = Error;
		
			fn try_from(value : u16) -> std::prelude::v1::Result<Self, Self::Error> {
				match value {
					$($value => Ok($type::$name)),*,
					num => Err(Error::InvalidFieldType { num }),
				}
			}
		}
	}
}

impl_try_from! {
enum BuiltinFieldType {
	End          =  0,
	FixedArray   =  1,
	Array        =  2,
	PtrArray     =  3,
	Byte         =  5,
	Byte4        =  6,
	Double       =  7,
	DWord        = 10,
	FileName     = 11,
	Float        = 12,
	Float2       = 13,
	Float3       = 14,
	Float4       = 15,
	Reference    = 16,
	QWord        = 17,
	WideCString     = 18,
	CString      = 19,
	Inline       = 20,
	Word         = 21,
	UUID         = 22,
	Byte3        = 23,
	DWord2       = 24,
	DWord4       = 25,
	Word3        = 26,
	FileRef      = 27,
	Variant      = 28,
	StructCommon = 29,
	SmallArray   = 33,
}
}

struct Reader<'a> {
	pub remaining : &'a [u8]
}

impl<'a> Reader<'a> {
	fn eat_slice(&mut self, len : usize) -> Result<Reader<'a>> {
		let (c, r) = self.remaining.split_at(len);
		self.remaining = r;
		Ok(Reader { remaining: c })
	}
	fn eat_u16(&mut self) -> Result<u16> {
		let (c, r) = self.remaining.split_at(2);
		self.remaining = r;
		Ok(u16::from_le_bytes(c.try_into().unwrap()))
	}
	fn eat_u32(&mut self) -> Result<u32> {
		let (c, r) = self.remaining.split_at(4);
		self.remaining = r;
		Ok(u32::from_le_bytes(c.try_into().unwrap()))
	}
	fn eat_u64(&mut self) -> Result<u64> {
		let (c, r) = self.remaining.split_at(8);
		self.remaining = r;
		Ok(u64::from_le_bytes(c.try_into().unwrap()))
	}
	pub fn eat_rva_as_offset(&mut self, exe : &PE) -> Result<usize> {
		let rva = self.eat_u64()? as usize;
		if rva == 0 { return Ok(0) }
		match rva.checked_sub(exe.base_addr) {
			Some(v) => Ok(exe.translate_rva_to_bin_offset(v)?),
			None => Err(Error::RvaOutOfBounds { rva }),
		}
	}
}


#[derive(Debug, Hash, PartialEq, Eq)]
struct ChunkIdentifier<'a> {
	pub magic : &'a str,
	pub max_version : u32,
	pub meta_offset : usize,
}




struct PE<'a> {
	raw_exe : &'a [u8],
	base_addr : usize,
	sections : [PESection; 32],
	n_sections : usize,
}

#[derive(Debug)]
struct PESection {
	virtual_address : usize,
	virtual_size    : usize,
	ptr_to_raw_data : usize,
}

impl<'a> PE<'a> {
	pub fn parse_header(raw_exe : &'a [u8]) -> Self {
		let signature_offset = u32::from_le_bytes(raw_exe[0x3c..][..4].try_into().unwrap()) as usize;
		assert!(&raw_exe[signature_offset..][..4] == &['P' as u8, 'E' as u8, 0, 0]);
		let coff_hdr_start = &raw_exe[signature_offset + 4..]; // skip signature to get to the actual header

		let n_sections = u16::from_le_bytes(coff_hdr_start[2..][..2].try_into().unwrap()) as usize;

		let opt_hdr_size = u16::from_le_bytes(coff_hdr_start[16..][..2].try_into().unwrap()) as usize;
		assert!(opt_hdr_size >= 20);
		let (optional_hdr, section_data_start) = &coff_hdr_start[20..].split_at(opt_hdr_size); // section headers are directly after the optional hdr
		let base_addr = u64::from_le_bytes(optional_hdr[24..][..8].try_into().unwrap()) as usize;

		let mut sections : [PESection; 32] = unsafe { MaybeUninit::zeroed().assume_init() };

		for (i, section_data) in section_data_start.chunks_exact(40).enumerate().take(n_sections) { //section headers are 40 bytes in size
			sections[i] = PESection {
				virtual_size   : u32::from_le_bytes(section_data[8..][..4].try_into().unwrap()) as usize,
				virtual_address: u32::from_le_bytes(section_data[12..][..4].try_into().unwrap()) as usize,
				ptr_to_raw_data: u32::from_le_bytes(section_data[20..][..4].try_into().unwrap()) as usize,
			};
		}

		PE { raw_exe, base_addr, sections, n_sections }
	}

	pub fn translate_rva_to_bin_offset(&self, rva : usize) -> Result<usize> {
		for i in 0..self.n_sections {
			let section = &self.sections[i];
			if rva < section.virtual_address || rva >= section.virtual_address + section.virtual_size { continue }

			return Ok(rva - section.virtual_address + section.ptr_to_raw_data);
		}

		Err(Error::NoSectionFound { rva })
	}

	pub fn get_str_at(&self, offset : usize) -> Result<&'a str> {
		if self.raw_exe.len() <= offset { return Err(Error::OffsetOutOfBounds { offset: offset }); }
		let mem = &self.raw_exe[offset..];
		let mut len = 0;
		while mem[len] != 0 { len += 1; }
		from_utf8(&mem[..len]).map_err(|_| Error::DecodingFailed)
	}

	pub fn reader_from_offset(&self, offset : usize) -> Result<Reader<'a>> {
		if self.raw_exe.len() <= offset { return Err(Error::OffsetOutOfBounds { offset }); }
		Ok(Reader{ remaining: &self.raw_exe[offset..] })
	}
}


use std::{collections::HashSet, mem::MaybeUninit, str::{from_utf8, from_utf8_unchecked}};
use crate::{structure::{ArrayKind, Chunk, Field, ReferenceKind, SpecificChunkVersion, Type}, Error, Result};
