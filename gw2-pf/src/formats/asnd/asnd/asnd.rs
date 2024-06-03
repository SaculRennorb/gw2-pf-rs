pub mod v2;

#[derive(Debug, crate::Parse)]
#[versioned_chunk]
pub enum ASND {
	#[v(2)]
	V2(v2::WaveformData),
}
