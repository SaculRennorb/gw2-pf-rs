pub mod v0;

#[derive(Debug, crate::Parse)]
#[chunk]
pub enum txtv {
	#[v(0)] V0(v0::TextPackVoices),
}