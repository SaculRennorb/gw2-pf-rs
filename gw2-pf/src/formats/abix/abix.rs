#[path = "bidx/bidx.rs"]
pub mod bidx;


#[derive(Default)]
pub struct ABIX {
	pub chunks : Vec<bidx::BIDX>,
}

impl crate::pf::PackFile for ABIX {
	const MAGIC : u32 = crate::fcc(b"ABIX");

	fn parse_chunk(&mut self, chunk_header : &crate::pf::ChunkHeader, data : &mut crate::parse::Input) -> Result<(), crate::parse::Error> {
		if chunk_header.magic == <bidx::BIDX as crate::pf::Chunk>::MAGIC {
			let chunk = <bidx::BIDX as crate::parse::ParseVersioned>::parse(chunk_header.version, data)?;
			self.chunks.push(chunk);
		}
		
		Ok(())
	}
}
