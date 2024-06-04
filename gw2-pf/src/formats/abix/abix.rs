#[path = "bidx/bidx.rs"]
pub mod bidx;


#[derive(crate::Parse)]
#[packfile]
pub enum ABIX {
	#[v(1)] V1(bidx::BIDX),
}
