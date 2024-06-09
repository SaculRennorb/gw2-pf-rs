#[path = "txtv/txtv.rs"]
pub mod _txtv;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum txtv {
	txtv(_txtv::txtv),
}