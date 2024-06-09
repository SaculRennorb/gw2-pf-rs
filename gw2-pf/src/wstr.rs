use crate::parse::{BinarySize, Error, Input, Parse, Result};

#[derive(Debug)]
pub struct WString(Vec<u16>);

impl<'inp> Parse<'inp> for WString {
	const BINARY_SIZE : BinarySize = BinarySize::dynamic(); // todo hmmm
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let u16slice = unsafe { std::slice::from_raw_parts(input.remaining.as_ptr().cast::<u16>(), input.remaining.len() / 2) };
		match u16slice.iter().position(|c| *c == 0) {
			None => Err(Error::CannotFindNullTerminator),
			Some(terminator_index) => {
				input.remaining = &input.remaining[(terminator_index + 1) * 2..]; // +1 to skip the actual terminator
				Ok(WString(u16slice[..terminator_index].to_owned()))
			},
		}
	}
}

impl std::ops::Deref for WString {
	type Target = Vec<u16>;
	fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for WString {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
