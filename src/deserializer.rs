use core::slice;

use crate::wstr::WSTRING_STRUCT_NAME;


pub struct Deserializer<'inp, const HAS_64_BIT_OFFSET : bool> {
	pub remaining : &'inp [u8],
}


#[derive(Debug)]
pub enum Error {
	InvalidFileType{ expected : u32, actual : u32 },
	NotSupported,
	DataTooShort,
	Custom(String), //todo
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
				Error::InvalidFileType { ref expected, ref actual } => {
					f.write_fmt(format_args!("Unexpected File Type: Expected: {}, Actual: {}", 
						unsafe { std::str::from_utf8_unchecked(slice::from_raw_parts(std::ptr::from_ref(expected).cast(), 4)) },
						unsafe { std::str::from_utf8_unchecked(slice::from_raw_parts(std::ptr::from_ref(actual).cast(), 4)) }
					))
				},
				_ => f.write_fmt(format_args!("{:?}", self))
		}
	}
}

impl std::error::Error for Error {}

impl serde::de::Error for Error { fn custom<T : std::fmt::Display>(msg : T) -> Self { Self::Custom(msg.to_string()) } }


//TODO could split impls i guess
impl<'inp, 'de, const HAS_64_BIT_OFFSET : bool>  Deserializer<'inp, HAS_64_BIT_OFFSET> {
	fn eat_u32(&mut self) -> Result<u32, Error> {
		if self.remaining.len() < 4 { return Err(Error::DataTooShort) }
		let v = u32::from_le_bytes(self.remaining[..4].try_into().unwrap());
		self.remaining = &self.remaining[4..];
		Ok(v)
	}

	fn eat_u64(&mut self) -> Result<u64, Error> {
		if self.remaining.len() < 8 { return Err(Error::DataTooShort) }
		let v = u64::from_le_bytes(self.remaining[..8].try_into().unwrap());
		self.remaining = &self.remaining[8..];
		Ok(v)
	}

	fn eat_offset(&mut self) -> Result<usize, Error> {
		if HAS_64_BIT_OFFSET {
			let v = self.eat_u64()? as usize;
			Ok(if v != 0 { v - 8 } else { 0 })
		}
		else {
			let v = self.eat_u32()? as usize;
			Ok(if v != 0 { v - 4 } else { 0 })
		}
	}

	/// Eats the terminator but does not return it.
	fn eat_wstr_bytes_no_term(&mut self) -> Result<&'inp [u8], Error> {
		let start = &self.remaining;

		let mut u16slice = unsafe { slice::from_raw_parts(self.remaining.as_ptr().cast::<u16>(), self.remaining.len() / 2) };
		let old_len = u16slice.len();
		while u16slice[0] != 0 { u16slice = &u16slice[1..]; }

		let result = &start[..(old_len - u16slice.len()) * 2]; // terminator not eaten yet

		self.remaining = &start[(old_len - u16slice.len() + 1) * 2..]; // +1 to skip the terminator

		Ok(result)
	}
}


// just plain copied form the old version. i feel like this is not neccesary, but i might be wrong
struct SeqAccess<'a, 'inp, const HAS_64_BIT_OFFSET : bool> {
	de: &'a mut Deserializer<'inp, HAS_64_BIT_OFFSET>,
	length: usize,
}

impl<'de, 'a, 'inp, const HAS_64_BIT_OFFSET : bool> serde::de::SeqAccess<'de> for SeqAccess<'a, 'inp, HAS_64_BIT_OFFSET> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where T: serde::de::DeserializeSeed<'de>,
	{
			if self.length > 0 {
					self.length -= 1;
					let value = seed.deserialize(&mut *self.de);
					Ok(Some(value?))
			} else {
					Ok(None)
			}
	}

	fn size_hint(&self) -> Option<usize> {
			Some(self.length)
	}
}

impl<'inp, 'de, const HAS_64_BIT_OFFSET : bool> serde::Deserializer<'de> for &mut Deserializer<'inp, HAS_64_BIT_OFFSET> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		Err(Error::NotSupported)
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		let offset = self.eat_offset()?;
		if offset == 0 { visitor.visit_none() }
		else { visitor.visit_some(&mut Deserializer::<HAS_64_BIT_OFFSET> { remaining: &self.remaining[offset..] }) }
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		visitor.visit_unit()
	}

	fn deserialize_unit_struct<V>(
			self,
			name: &'static str,
			visitor: V,
	) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_newtype_struct<V>(
			self,
			name: &'static str,
			visitor: V,
	) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		match name {
			WSTRING_STRUCT_NAME => {
				let bytes = self.eat_wstr_bytes_no_term()?;
				visitor.visit_bytes(bytes) // this is teh only implemented function by the custom wstring visitor we use
			},
			_ => todo!(),
		}
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		let length = self.eat_u32()? as usize;
		let offset = self.eat_offset()?;
		if length > 0 {
			visitor.visit_seq(SeqAccess { de: &mut Deserializer::<HAS_64_BIT_OFFSET> { remaining: &self.remaining[offset..] }, length })
		}
		else{
			visitor.visit_seq(SeqAccess { de: self, length: 0 })
		}
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		visitor.visit_seq(SeqAccess { de: self, length: len })
	}

	fn deserialize_tuple_struct<V>(
			self,
			name: &'static str,
			len: usize,
			visitor: V,
	) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_struct<V>(
			self,
			name: &'static str,
			fields: &'static [&'static str],
			visitor: V,
	) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		self.deserialize_tuple(fields.len(), visitor)
	}

	fn deserialize_enum<V>(
			self,
			name: &'static str,
			variants: &'static [&'static str],
			visitor: V,
	) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where V: serde::de::Visitor<'de> {
		todo!()
	}
}