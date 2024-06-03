#[path = "asnd/asnd.rs"]
pub mod asnd;

#[derive(Debug, Default)]
pub struct ASND {
	pub chunks : Vec<asnd::ASND>,
}

impl crate::pf::PackFile for ASND {
	const MAGIC : u32 = crate::fcc(b"ASND");

	fn parse_chunk(&mut self, chunk_header : &crate::pf::ChunkHeader, data : &[u8]) -> Result<(), crate::deserializer::Error> {
		if let Ok(chunk) = <asnd::ASND as crate::pf::Chunk>::try_parse(chunk_header, data) {
			self.chunks.push(chunk);
		}
		
		Ok(())
	}
}