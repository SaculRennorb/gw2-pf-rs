#[path = "bkck/bkck.rs"]
pub mod bkck;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ABNK {
	BKCK(bkck::BKCK),
}

impl std::ops::Deref for ABNK {
	type Target = bkck::BKCK;
	fn deref(&self) -> &Self::Target { match self { Self::BKCK(ref s) => s } }
}
