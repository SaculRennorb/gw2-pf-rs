use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BankIndexData {
	pub bank_language: Vec<BankLanguageData>,
}

#[derive(Debug, Deserialize)]
pub struct BankLanguageData {
	pub bank_file_name: Vec<BankFileNameData>,
}

#[derive(Debug, Deserialize)]
pub struct BankFileNameData {
	pub file_name: Option<crate::wstr::WString>, // for now, could use cow or similar
}
