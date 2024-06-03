//todo parseinto

#[derive(Debug)]
pub enum Error {
	InvalidFileType{ expected : u32, actual : u32 },
	NotSupported,
	DataTooShort,
	CannotFindNullTerminator,
	UnknownVersion{ actual : u16 },
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
				Error::InvalidFileType { ref expected, ref actual } => {
					f.write_fmt(format_args!("Unexpected File Type: Expected: {}, Actual: {}", 
						unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(expected).cast(), 4)) },
						unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(actual).cast(), 4)) }
					))
				},
				_ => f.write_fmt(format_args!("{:?}", self))
		}
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Parse : Sized {
	fn parse(input : &mut Input) -> Result<Self>;
}

pub struct Input<'inp> {
	pub remaining : &'inp [u8],
	pub is_64_bit : bool,
}

impl Input<'_> {
	pub fn clone_with_offset(&self, offset : usize) -> Result<Self> {
		if offset > self.remaining.len() { Err(Error::DataTooShort) }
		else { Ok(Self { remaining: &self.remaining[offset..], is_64_bit: self.is_64_bit }) }
	}

	pub fn eat_offset(&mut self) -> Result<usize>  {
		if self.is_64_bit {
			let v = u64::parse(self)? as usize;
			Ok(if v != 0 { v - 8 } else { 0 })
		}
		else {
			let v = u32::parse(self)? as usize;
			Ok(if v != 0 { v - 4 } else { 0 })
		}
	}
}

macro_rules! impl_le_bit_prase {
	($($type:ty),*) => {$(
		impl Parse for $type {
			fn parse(input : &mut Input) -> Result<Self> {
				const SIZE : usize = std::mem::size_of::<$type>();
				if input.remaining.len() < SIZE { return Err(Error::DataTooShort) }
				let v = <$type>::from_le_bytes(input.remaining[..SIZE].try_into().unwrap());
				input.remaining = &input.remaining[SIZE..];
				Ok(v)
			}
		}
	)*}
}

impl_le_bit_prase! {
	u8, u16, u32, u64,
	i8, i16, i32, i64,
	         f32, f64
}

impl<T : Parse> Parse for Option<T> {
	fn parse(input : &mut Input) -> Result<Self> {
		let offset = input.eat_offset()?;
		if offset == 0 { Ok(None) }
		else { T::parse(&mut input.clone_with_offset(offset)?).map(Some) }
	}
}

impl<T : Parse> Parse for Vec<T> {
	fn parse(input : &mut Input) -> Result<Self> {
		let mut length = u32::parse(input)? as usize;
		let offset = input.eat_offset()?;
		if length > 0 {
			let vec_input = &mut input.clone_with_offset(offset)?;

			let mut vec = Vec::with_capacity(length);
			while length > 0 {
				vec.push(T::parse(vec_input)?);
				length -= 1;
			}

			Ok(vec)
		}
		else {
			Ok(vec![])
		}
	}
}

pub trait ParseVersioned : Sized {
	fn parse(version : u16, input : &mut Input) -> Result<Self>;
}
