#[derive(Debug)]
pub enum Error {
	InvalidFileType{ r#type : &'static str, expected : u32, actual : u32 },
	NotSupported,
	DataTooShort { r#type : Option<&'static str>, required : usize, actual : usize },
	CannotFindNullTerminator,
	UnknownVersion{ r#type : &'static str, actual : u16 },
	UnknownMagic{ r#type : &'static str, actual : u32 },
	UnknownMagicOrVersion{ r#type : &'static str, actual_magic : u32, actual_version : u32 }, //TODO(Rennorb) @cleanup
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
			Error::UnknownMagic { r#type, actual } => {
				f.write_fmt(format_args!("Unexpected magic bytes for {}: {:x?} ({})", r#type,
					&actual.to_le_bytes(), unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(actual).cast(), 4)) }
				))
			},
			Error::UnknownMagicOrVersion { r#type, actual_magic, actual_version } => {
				f.write_fmt(format_args!("Unexpected magic bytes or version for {}: actual magic: {:x?} ({}), version: {}", r#type,
					&actual_magic.to_le_bytes(), unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(std::ptr::from_ref(actual_magic).cast(), 4)) },
					actual_version
				))
			},
			_ => f.write_fmt(format_args!("{:?}", self))
		}
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct BinarySize {
	pub fixed_bytes : usize,
	pub ptrs : usize,
	pub is_dynamic : bool,
}

impl BinarySize {
	pub const fn fixed(fixed_bytes : usize) -> Self { Self { fixed_bytes, ptrs: 0, is_dynamic: false }}
	pub const fn ptrs(ptrs : usize) -> Self { Self { fixed_bytes: 0, ptrs, is_dynamic: false }}
	pub const fn dynamic() -> Self { Self { fixed_bytes: 0, ptrs: 0, is_dynamic: true }}
	pub const fn add(&self, rhs : &Self) -> Self { Self {
		fixed_bytes: self.fixed_bytes + rhs.fixed_bytes,
		ptrs: self.ptrs + rhs.ptrs,
		is_dynamic: self.is_dynamic | rhs.is_dynamic,
	}}

	pub const fn actual_size(&self, uses_64_bit_ptrs : bool) -> Option<usize> {
		if self.is_dynamic { return None }
		Some(self.fixed_bytes + self.ptrs * if uses_64_bit_ptrs { 8 } else { 4 })
	}
}

pub trait Parse<'inp> : Sized {
	const BINARY_SIZE : BinarySize;
	fn parse(input : &mut Input<'inp>) -> Result<Self>;
}

#[derive(Clone)]
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
			const BINARY_SIZE : BinarySize = BinarySize::fixed(std::mem::size_of::<$type>());
			fn parse(input : &mut Input<'inp>) -> Result<Self> {
				const BINARY_SIZE : usize = std::mem::size_of::<$type>();
				if input.remaining.len() < BINARY_SIZE { return Err(Error::to_short::<$type>(input.remaining.len())) }
				let v = <$type>::from_le_bytes(input.remaining[..BINARY_SIZE].try_into().unwrap());
				input.remaining = &input.remaining[BINARY_SIZE..];
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
	const BINARY_SIZE : BinarySize = BinarySize::ptrs(1);
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let offset = input.eat_offset()?;
		if offset == 0 { Ok(None) }
		else { T::parse(&mut input.clone_with_offset(offset)?).map(Some) }
	}
}

impl<'inp, T : Parse<'inp>> Parse<'inp> for Vec<T> {
	const BINARY_SIZE : BinarySize = BinarySize{ fixed_bytes: 4, ptrs: 1, is_dynamic: false };
	fn parse(input : &mut Input<'inp>) -> Result<Self> {
		let length = u32::parse(input)? as usize;
		let offset = input.eat_offset()?;
		match length {
			0 => Ok(vec![]),
			//todo length check
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

pub fn parse_null_terminated_vec<'inp, T : Parse<'inp>>(input : &mut Input<'inp>) -> Result<Vec<T>> {
	let length = u32::parse(input)? as usize;
	let offset = input.eat_offset()?;
	match length { 
		0 => Ok(vec![]),
		//todo length check
		mut length => {
			let vec_input = &mut input.clone_with_offset(offset)?;

			let mut vec = Vec::with_capacity(length);
			while length > 0 {
				//todo test during macro expansion
				if let Some(el_size) = T::BINARY_SIZE.actual_size(input.is_64_bit) {
					if vec_input.remaining.len() < el_size { return Err(Error::to_short::<T>(vec_input.remaining.len())); }

					if vec_input.remaining[..el_size].iter().all(|b| *b == 0) { break }
				}

				vec.push(T::parse(vec_input)?);
				length -= 1;
			}

			Ok(vec)
		}
	}
}

impl<'inp> Parse<'inp> for &'inp [u8] {
	const BINARY_SIZE : BinarySize = BinarySize{ fixed_bytes: 4, ptrs: 1, is_dynamic: false };
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
	type Output;
	fn parse(version : u16, input : &mut Input<'inp>) -> Result<Self::Output>;
}

pub trait ParseMagicVariant<'inp> : Sized {
	fn parse(magic : u32, version : u16, input : &mut Input<'inp>) -> Result<Self>;
}

pub trait ChunkVariant<'inp> : Sized + ParseMagicVariant<'inp> {
	fn parse_sequence(input : Input<'inp>) -> ChunkIter<'inp, Self>;
}

pub struct ChunkIter<'inp, V : ChunkVariant<'inp> + ParseMagicVariant<'inp>> {
	pub input : Input<'inp>,
	pub _p : std::marker::PhantomData<V>
}

impl<'inp, V : ChunkVariant<'inp> + ParseMagicVariant<'inp>> Iterator for ChunkIter<'inp, V> {
	type Item = Result<V>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.input.remaining.len() < std::mem::size_of::<crate::pf::ChunkHeader>() { return None }

		let chunk_header = unsafe{ self.input.remaining.as_ptr().cast::<crate::pf::ChunkHeader>().as_ref().unwrap() };
		let chunk_data = &self.input.remaining[chunk_header.chunk_header_size as usize..][..chunk_header.descriptor_offset as usize];
		let chunk_input = &mut Input { remaining: chunk_data, is_64_bit: self.input.is_64_bit };

		let next_offset = 8 + chunk_header.next_chunk_offset as usize;  // no clue where the +8 comes from
		if next_offset <= self.input.remaining.len() { self.input.remaining = &self.input.remaining[next_offset..]; }
		
		Some(V::parse(chunk_header.magic, chunk_header.version, chunk_input))
	}
}
