use serde::Deserialize;



#[derive(Debug)]
pub struct WString(pub Vec<u16>);

struct WStringVisitor;

impl<'v> serde::de::Visitor<'v> for WStringVisitor {
	type Value = WString;

	fn expecting(&self, formatter : &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a null terminated wide (u16 bpc) c-string")
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
		where E: serde::de::Error {
			// already validated in the deserializer
			//if v.len() % 2 != 0 { return Err(E::custom("invalid alignment, wstr must have byte len % 2 == 0")) }

			let slice = unsafe{ core::slice::from_raw_parts(v.as_ptr().cast::<u16>(), v.len() / 2) };

			Ok(WString(slice.to_owned()))
	}
}

impl<'de> Deserialize<'de> for WString {
	fn deserialize<D>(d : D) -> Result<Self, D::Error>
	where D : serde::Deserializer<'de> {
		d.deserialize_newtype_struct(WSTRING_STRUCT_NAME, WStringVisitor)
	}
}

pub const WSTRING_STRUCT_NAME : &'static str = "gw2-pf::WString";
