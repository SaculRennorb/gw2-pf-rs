pub mod v0;

#[derive(Debug, crate::Parse)]
#[versioned_chunk]
pub enum BIDX {
	#[v(0)]
	V0(v0::BankIndexData),
}

impl std::ops::Deref for BIDX {
	type Target = v0::BankIndexData;
	fn deref(&self) -> &Self::Target { match self { Self::V0(ref s) => s } }
}
