#[path = "asnd/asnd.rs"]
pub mod asnd;

#[derive(Debug, crate::Parse)]
#[packfile]
pub enum ASND<'a> {
	ASND(asnd::ASND<'a>),
}
