pub mod v2;

#[derive(Debug, crate::Parse)]
#[chunk]
pub enum BKCK<'a> {
	#[v(2)] V2(v2::BankFileData<'a>),
}
