pub mod structure;
pub mod analyze;
pub mod generate;


#[derive(Debug)]
pub enum Error {
	NoSectionFound { rva : usize },
	InvalidFieldType { num : u16 },
	RvaOutOfBounds { rva : usize },
	OffsetOutOfBounds { offset : usize },
	DuplicateChunk { magic: [u8; 4], max_version: u32, meta_offset: usize },
	NoChunks,
	DecodingFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
