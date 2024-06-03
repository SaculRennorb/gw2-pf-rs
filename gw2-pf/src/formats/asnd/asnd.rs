#[path = "asnd/asnd.rs"]
pub mod asnd;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ASND {
	ASND(asnd::ASND),
}

impl std::ops::Deref for ASND {
	type Target = asnd::ASND;
	fn deref(&self) -> &Self::Target { match self { Self::ASND(ref s) => s } }
}
