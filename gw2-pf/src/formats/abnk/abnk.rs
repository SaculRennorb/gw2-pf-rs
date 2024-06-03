#[path = "bkck/bkck.rs"]
pub mod bkck;

#[derive(Debug, Default)]
pub struct ABNK {
	pub chunks : Vec<bkck::BKCK>,
}

impl crate::pf::PackFile for ABNK {
	const MAGIC : u32 = crate::fcc(b"ABNK");

	fn parse_chunk(&mut self, chunk_header : &crate::pf::ChunkHeader, data : &[u8]) -> Result<(), crate::deserializer::Error> {
		if let Ok(chunk) = <bkck::BKCK as crate::pf::Chunk>::try_parse(chunk_header, data) {
			self.chunks.push(chunk);
		}
		
		Ok(())
	}
}