pub mod v2;

#[derive(Debug, crate::Parse)]
#[versioned_chunk]
pub enum ASND {
	#[v(2)]
	V2(v2::WaveformData),
}

impl std::ops::Deref for ASND {
	type Target = v2::WaveformData;
	fn deref(&self) -> &Self::Target { match self { ASND::V2(ref s) => s } }
}
