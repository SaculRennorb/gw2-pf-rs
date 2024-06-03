//todo parseinto

#[derive(Debug)]
pub enum Error {
	InvalidFileType{ r#type : &'static str, expected : u32, actual : u32 },
	NotSupported,
	DataTooShort { r#type : Option<&'static str>, required : usize, actual : usize },
	CannotFindNullTerminator,
	UnknownVersion{ r#type : &'static str, actual : u16 },
	UnknownMagic{ r#type : &'static str, actual : u32 },
}

impl Error {
	pub fn to_short<T>(actual : usize) -> Self { Self::DataTooShort {
		r#type: Some(std::any::type_name::<T>()), required: std::mem::size_of::<T>(), actual
	}}

	pub fn wrong_magic<T : crate::pf::Magic>(actual : u32) -> Self { Self::InvalidFileType {
		r#type: std::any::type_name::<T>(), expected: T::MAGIC, actual
	}} 
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::DataTooShort { r#type, required, actual } => {
				f.write_str("Data too short")?;
				if let Some(r#type) = r#type {
					f.write_fmt(format_args!(" for {}", r#type))?;
				}
				f.write_fmt(format_args!(": required: {required}, actual: {actual}"))
			},
			Error::InvalidFileType { r#type, expected, actual } => {
				f.write_fmt(format_args!("Unexpected file type for {}: expected: {:x?} ({}), actual: {:x?} ({})", r#type,
					&expected.to_le_bytes(), unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(expected).cast(), 4)) },
					&actual.to_le_bytes(), unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(actual).cast(), 4)) }
				))
			},
			_ => f.write_fmt(format_args!("{:?}", self))
		}
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Parse<'inp> : Sized {
	fn parse(input : &mut Input<'inp>) -> Result<Self>;
}

pub struct Input<'inp> {
	pub remaining : &'inp [u8],
	pub is_64_bit : bool,
}

impl Input<'_> {
	pub fn clone_with_offset(&self, offset : usize) -> Result<Self> {
		if offset > self.remaining.len() { Err(Error::DataTooShort{ r#type: None, required: offset, actual: self.remaining.len() }) }
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
		impl<'inp> Parse<'inp> for $type {
			fn parse(input : &mut Input<'inp>) -> Result<Self> {
				const SIZE : usize = std::mem::size_of::<$type>();
				if input.remaining.len() < SIZE { return Err(Error::to_short::<$type>(input.remaining.len())) }
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

impl<'inp, T : Parse<'inp>> Parse<'inp> for Option<T> {
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let offset = input.eat_offset()?;
		if offset == 0 { Ok(None) }
		else { T::parse(&mut input.clone_with_offset(offset)?).map(Some) }
	}
}

impl<'inp, T : Parse<'inp>> Parse<'inp> for Vec<T> {
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let length = u32::parse(input)? as usize;
		let offset = input.eat_offset()?;
		match length {
			0 => Ok(vec![]),
			mut length => {
				let vec_input = &mut input.clone_with_offset(offset)?;

				let mut vec = Vec::with_capacity(length);
				while length > 0 {
					vec.push(T::parse(vec_input)?);
					length -= 1;
				}
	
				Ok(vec)
			}
		}
	}
}

impl<'inp> Parse<'inp> for &'inp [u8] {
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let length = u32::parse(input)? as usize;
		let offset = input.eat_offset()?;
		match length { 
			0 => Ok(&[]),
			length if input.remaining.len() < offset + length => Err(Error::DataTooShort {
				r#type: Some(std::any::type_name::<Self>()), required: length, actual: input.remaining.len(),
			}),
			length => Ok(&input.remaining[offset..][..length]),
		}
	}
}

pub trait ParseVersioned<'inp> : Sized {
	fn parse(version : u16, input : &mut Input<'inp>) -> Result<Self>;
}

pub trait ParseMagicVariant<'inp> : Sized {
	fn parse(magic : u32, version : u16, input : &mut Input<'inp>) -> Result<Self>;
}

