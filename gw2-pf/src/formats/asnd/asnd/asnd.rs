pub mod v2;

#[derive(Debug, serde::Deserialize)]
pub struct ASND {
	data : v2::WaveformData,
}

impl crate::pf::Chunk for ASND {
	const MAGIC : u32 = crate::fcc(b"ASND");
}

impl std::ops::Deref for ASND {
	type Target = v2::WaveformData;
	fn deref(&self) -> &Self::Target { &self.data }
}
