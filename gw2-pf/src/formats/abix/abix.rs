#[path = "bidx/bidx.rs"]
pub mod bidx;

#[derive(crate::Parse)]
#[packfile]
pub enum ABIX {
	BIDX(bidx::BIDX),
}
