use std::{borrow::Cow, collections::HashSet, mem::MaybeUninit, str::{from_utf8, from_utf8_unchecked}};

pub fn locate_structs(raw_exe : &[u8]) {
	println!("reading exe header");
	let exe = PE::parse_header(raw_exe);
	println!("ok");

	let parser = &mut Parser{ exe, chunk_cache: HashSet::new() };

	const ASCII_HI_BITS : u8 = 0b01000000;
	const ASCII_MASK    : u8 = 0b11000000;
	// four ascii bytes, e.g. ABIX, and u32 version that realistically doesn't use more than the first byte
	let target = u64::from_le_bytes([ASCII_HI_BITS, ASCII_HI_BITS, ASCII_HI_BITS, ASCII_HI_BITS, 0, 0, 0, 0]);
	let mask = u64::from_le_bytes([ASCII_MASK, ASCII_MASK, ASCII_MASK, ASCII_MASK, 0, 0xff, 0xff, 0xff]);
	let mut remaining = raw_exe;
	while remaining.len() >= 8 {
		let (chunk, rest) = remaining.split_at(8);
		let value = u64::from_le_bytes(chunk.try_into().unwrap());
		if value & mask == target {
			// unfortunately we will catch a lot of special characters with just a mask match, so we filter for ascii chars again
			const _A : u8 = 'A' as u8;
			const _Z : u8 = 'Z' as u8;
			#[allow(non_upper_case_globals)]
			const _a : u8 = 'a' as u8;
			#[allow(non_upper_case_globals)]
			const _z : u8 = 'z' as u8;
			if (&chunk[..4]).iter().all(|c| matches!(*c, _A..=_Z | _a..=_z)) {
				if let Ok(len) = parser.parse_chunk(&mut Reader { remaining }) {
					remaining = &remaining[len..];
					continue;
				}
			}
		}

		remaining = rest;
	}
}

#[derive(Debug)]
struct Chunk<'a> {
	magic    : &'a str, //can be 4 or 3 bytes long
	versions : Vec<SpecificChunkVersion<'a>>,
}

impl<'a> Parser<'a> {
	pub fn parse_chunk(&mut self, input : &mut Reader<'a>) -> Result<usize> {
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
	
		let pf = Chunk { magic, versions };
		println!("{pf:#?}");	
	
		Ok(initial_len - input.remaining.len())
	}
}

#[derive(Debug)]
struct SpecificChunkVersion<'a> {
	pub version : u32,
	pub root : Struct<'a>,
}

impl<'a> Parser<'a> {
	pub fn parse_chunk_versions(&self, input : &mut Reader<'a>, n_versions : u32) -> Result<Vec<SpecificChunkVersion<'a>>> {
		let mut chunks = Vec::with_capacity(n_versions as usize);

		for version in 0..n_versions {
			let chunk_meta_header_input = &mut input.eat_slice(24)?;

			let chunk_offset = chunk_meta_header_input.eat_rva_as_offset(&self.exe)?;
			if chunk_offset == 0 { continue }

			let root_input = &mut self.exe.reader_from_offset(chunk_offset)?;
			let root = self.parse_struct(root_input)?;

			let chunk = SpecificChunkVersion{ version, root };
			chunks.push(chunk);
		}

		Ok(chunks)
	}
}

#[derive(Debug)]
struct Struct<'a> {
	pub name   : &'a str,
	pub fields : Vec<Field<'a>>,
}

impl<'a> Parser<'a> {
	pub fn parse_struct(&self, input : &mut Reader<'a>) -> Result<Struct<'a>> {
		let mut fields = Vec::new();

		let name = loop {
			let field = self.prase_field(input).inspect_err(|f| println!("- {f:?}"))?;
			if matches!(field.detail, FieldDetail::End) { break field.name }
			fields.push(field);
		};

		Ok(Struct { name, fields })
	}
}

#[derive(Debug)]
struct Field<'a> {
	pub name : &'a str,
	pub detail : FieldDetail<'a>,
}

impl<'a> std::ops::Deref for Field<'a> {
	type Target = FieldDetail<'a>;
	fn deref(&self) -> &Self::Target { &self.detail }
}

#[derive(Debug)]
enum FieldDetail<'a> {
	End,
	FixedArray  { size : usize, inner : Struct<'a> },
	Array       { size : usize, inner : Struct<'a> },
	PtrArray    { size : usize, inner : Struct<'a> },
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
	SmallArray  { size : usize, inner : Struct<'a> },
}

impl FieldDetail<'_> {
	pub fn format_type(field : &FieldDetail) -> Cow<'static, str> {
		match field {
			FieldDetail::FixedArray{size, ..}   => Cow::Owned(format!("[; {size}]")),
			FieldDetail::Array{..}        => Cow::Borrowed("Vec"),
			FieldDetail::PtrArray{..}     => Cow::Borrowed("can't represent"),
			FieldDetail::Byte{..}         => Cow::Borrowed("u8"),
			FieldDetail::Byte4{..}        => Cow::Borrowed("[u8; 4]"),
			FieldDetail::Double{..}       => Cow::Borrowed("f64"),
			FieldDetail::DoubleWord{..}   => Cow::Borrowed("u32"),
			FieldDetail::FileName{..}     => Cow::Borrowed("FileId"),
			FieldDetail::Float{..}        => Cow::Borrowed("f32"),
			FieldDetail::Float2{..}       => Cow::Borrowed("[f32; 2]"),
			FieldDetail::Float3{..}       => Cow::Borrowed("[f32; 3]"),
			FieldDetail::Float4{..}       => Cow::Borrowed("[f32; 4]"),
			FieldDetail::Reference{..}    => Cow::Borrowed("can't represent"),
			FieldDetail::QuadWord{..}     => Cow::Borrowed("u64"),
			FieldDetail::WideCString{..}  => Cow::Borrowed("WString"),
			FieldDetail::CString{..}      => Cow::Borrowed("CString"),
			FieldDetail::Inline{..}       => Cow::Borrowed("can't represent"),
			FieldDetail::Word{..}         => Cow::Borrowed("can't represent"),
			FieldDetail::UUID{..}         => Cow::Borrowed("can't represent"),
			FieldDetail::Byte3{..}        => Cow::Borrowed("[u8; 3]"),
			FieldDetail::DoubleWord2{..}  => Cow::Borrowed("[u32; 2]"),
			FieldDetail::DoubleWord4{..}  => Cow::Borrowed("[u32; 4]"),
			FieldDetail::DoubleWord3{..}  => Cow::Borrowed("[u32; 3]"),
			FieldDetail::FileRef{..}      => Cow::Borrowed("FileId"),
			FieldDetail::Variant{..}      => Cow::Borrowed("can't represent"),
			FieldDetail::StructCommon{..} => Cow::Borrowed("can't represent"),
			FieldDetail::SmallArray{..}   => Cow::Borrowed("Vec"),
			FieldDetail::End              => Cow::Borrowed(""),
		}
	}
}

impl<'a> Parser<'a> {
	pub fn prase_field(&self, input : &mut Reader<'a>) -> Result<Field<'a>> {
		let input = &mut input.eat_slice(32)?; // all fields are 32 byte

		let _type = input.eat_u16()?; // 0-2
		input.eat_u16()?; // 2-4
		input.eat_u32()?; // 4-8
		let name = self.exe.get_str_at(input.eat_rva_as_offset(&self.exe)?)?;

		let detail = match _type {
			0 => FieldDetail::End,
			1 => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;
				let inner = self.parse_struct(&mut self.exe.reader_from_offset(offset)?)?;
				FieldDetail::FixedArray { size, inner }
			},
			2 => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;
				let inner = self.parse_struct(&mut self.exe.reader_from_offset(offset)?)?;
				FieldDetail::Array { size, inner }
			},
			3 => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;
				let inner = self.parse_struct(&mut self.exe.reader_from_offset(offset)?)?;
				FieldDetail::PtrArray { size, inner }
			},
			5 => FieldDetail::Byte,
			6 => FieldDetail::Byte4,
			7 => FieldDetail::Double,
			10 => FieldDetail::DoubleWord,
			11 => FieldDetail::FileName,
			12 => FieldDetail::Float,
			13 => FieldDetail::Float2,
			14 => FieldDetail::Float3,
			15 => FieldDetail::Float4,
			16 => FieldDetail::Reference {},
			17 => FieldDetail::QuadWord,
			18 => FieldDetail::WideCString,
			19 => FieldDetail::CString,
			20 => FieldDetail::Inline {},
			21 => FieldDetail::Word,
			22 => FieldDetail::UUID,
			23 => FieldDetail::Byte3,
			24 => FieldDetail::DoubleWord2,
			25 => FieldDetail::DoubleWord4,
			26 => FieldDetail::DoubleWord3,
			27 => FieldDetail::FileRef,
			28 => FieldDetail::Variant {},
			29 => FieldDetail::StructCommon {},
			33 => {
				let offset = input.eat_rva_as_offset(&self.exe)?;
				let size = input.eat_u64()? as usize;
				let inner = self.parse_struct(&mut self.exe.reader_from_offset(offset)?)?;
				FieldDetail::SmallArray { size, inner }
			},
			num => return Err(Error::InvalidFieldType { num })
		};

		Ok(Field { name, detail })
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

struct Parser<'a> {
	pub exe : PE<'a>,
	pub chunk_cache : HashSet<ChunkIdentifier<'a>>,
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
		println!("{n_sections} sections:");

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
			println!("{:?}", sections[i]);
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


#[derive(Debug)]
enum Error {
	NoSectionFound { rva : usize },
	InvalidFieldType { num : u16 },
	RvaOutOfBounds { rva : usize },
	OffsetOutOfBounds { offset : usize },
	DuplicateChunk { magic: [u8; 4], max_version: u32, meta_offset: usize },
	NoChunks,
	DecodingFailed,
}

type Result<T> = std::result::Result<T, Error>;

