use crate::parse::{Input, Parse, Error};

#[derive(Debug)]
pub struct WString(pub Vec<u16>);

impl<'inp> Parse<'inp> for WString {
	fn parse(input : &mut Input<'inp>) -> crate::parse::Result<Self> {
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