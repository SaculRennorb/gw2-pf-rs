#[path = "bkck/bkck.rs"]
pub mod bkck;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ABNK<'a> {
	#[v(1)] V1(bkck::BKCK<'a>),
}
