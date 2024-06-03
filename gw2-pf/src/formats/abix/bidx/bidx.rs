pub mod v0;

#[derive(Debug, crate::Parse)]
#[versioned_chunk]
pub enum BIDX {
	#[v(0)]
	V0(v0::BankIndexData),
}
