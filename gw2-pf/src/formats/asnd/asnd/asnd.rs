pub mod v2;

#[derive(Debug, crate::Parse)]
#[chunk]
pub enum ASND<'a> {
	#[v(2)]
	V2(v2::WaveformData<'a>),
}
