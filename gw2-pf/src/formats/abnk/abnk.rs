#[path = "bkck/bkck.rs"]
pub mod bkck;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ABNK<'a> {
	BKCK(bkck::BKCK<'a>),
}
