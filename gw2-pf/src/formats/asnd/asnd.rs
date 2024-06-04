#[path = "asnd/asnd.rs"]
pub mod asnd;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ASND<'a> {
	#[v(1)] V1(asnd::ASND<'a>),
}
