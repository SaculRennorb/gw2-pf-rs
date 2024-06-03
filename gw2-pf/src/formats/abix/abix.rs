#[path = "bidx/bidx.rs"]
pub mod bidx;

#[derive(crate::Parse)]
#[packfile]
pub enum ABIX {
	BIDX(bidx::BIDX),
}

impl std::ops::Deref for ABIX {
	type Target = bidx::BIDX;

	fn deref(&self) -> &Self::Target { match self { ABIX::BIDX(ref s) => s } }
}
