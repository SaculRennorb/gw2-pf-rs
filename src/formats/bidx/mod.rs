pub mod v0;

#[derive(Default)]
pub struct BIDX {
	pub chunks : Vec<v0::BankIndexData>,
}

impl crate::pf::PackFile for BIDX {
	const MAGIC : u32 = 0x58494241;//todo macro

	// i chose to declare the interface like this to allwo you to have one large vector of stuff in your packfile struct and gradually fill it during deserialization.
	// chose to not do that, but this doesnt matter either way
	fn parse_chunk(&mut self, chunk_header : &crate::pf::ChunkHeader, data : &[u8]) -> Result<(), crate::deserializer::Error> {
		use serde::Deserialize;

		const MAGIC : u32 = 0x58444942; //todo macro

		if chunk_header.magic == MAGIC {
			let mut d = crate::deserializer::Deserializer::<false>{ remaining: data };
			self.chunks.push(v0::BankIndexData::deserialize(&mut d)?);
		}
		
		Ok(())
	}
}

