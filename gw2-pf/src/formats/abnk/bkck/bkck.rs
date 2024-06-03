pub mod v2;

#[derive(Debug, crate::Parse)]
#[versioned_chunk]
pub enum BKCK {
	#[v(2)]
	V2(v2::BankFileData),
}

impl std::ops::Deref for BKCK {
	type Target = v2::BankFileData;
	fn deref(&self) -> &Self::Target { match self {BKCK::V2(ref s) => s } }
}
