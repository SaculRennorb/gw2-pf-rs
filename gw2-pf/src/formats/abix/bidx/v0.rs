use crate::FileName;

#[derive(Debug, crate::Parse)]
pub struct BankIndexData {
	pub bank_language: Vec<BankLanguageData>,
}

#[derive(Debug, crate::Parse)]
pub struct BankLanguageData {
	pub bank_file_name: Vec<BankFileNameData>,
}

#[derive(Debug, crate::Parse)]
pub struct BankFileNameData {
	pub file_name: Option<FileName>,
}
