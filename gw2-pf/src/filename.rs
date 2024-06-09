use crate::{parse::{BinarySize, Result, Input, Parse}, wstr::WString};

#[derive(Debug)]
pub struct FileName(WString);

impl<'inp> Parse<'inp> for FileName {
	const BINARY_SIZE : BinarySize = WString::BINARY_SIZE;
	fn parse(input : &mut Input<'inp>) -> Result<Self> { WString::parse(input).map(Self) }
}

impl std::ops::Deref for FileName {
	type Target = WString;
	fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for FileName {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl FileName {
	pub fn to_id(&self) -> u32 {
		if self.len() < 2 {
			0
		} else {
			((self[1] as i32 * 0xff00) + (self[0] as i32 - 0xff00ff)) as u32
		}
	}
}
