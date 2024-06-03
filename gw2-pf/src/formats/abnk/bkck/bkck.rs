pub mod v2;

#[derive(Debug, serde::Deserialize)]
pub struct BKCK {
	data : v2::BankFileData,
}

impl crate::pf::Chunk for BKCK {
	const MAGIC : u32 = crate::fcc(b"BKCK");
}

impl std::ops::Deref for BKCK {
	type Target = v2::BankFileData;
	fn deref(&self) -> &Self::Target { &self.data }
}
