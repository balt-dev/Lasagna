// TODO: Rewrite parser according to new spec
use std::str::FromStr;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Literal<'a> {
	Float(f32),
	Boolean(bool),
	U8(u8),
	I8(i8),
	U16(u16),
	I16(i16),
	U32(u32),
	I32(i32),
	String(Cow<'a, str>),
	Bytes(Box<[u8]>)
}

impl<'a> TryFrom<&'a str> for Literal<'a> {
	type Error = ();

	fn try_from(raw: &'a str) -> Result<Self, Self::Error> {
		if (raw == "true") || (raw == "false") {
			Ok(Self::Boolean(raw == "true"))
		} else if let Some(raw) = raw.strip_suffix("_u8") {
			raw.parse::<u8>()
				.map(|v| Self::U8(v))
				.map_err(|_| ())
		} else if let Some(raw) = raw.strip_suffix("_i8") {
			raw.parse::<i8>()
				.map(|v| Self::I8(v))
				.map_err(|_| ())
		} else if let Some(raw) = raw.strip_suffix("_u16") {
			raw.parse::<u16>()
				.map(|v| Self::U16(v))
				.map_err(|_| ())
		} else if let Some(raw) = raw.strip_suffix("_i16") {
			raw.parse::<i16>()
				.map(|v| Self::I16(v))
				.map_err(|_| ())
		} else if let Some(raw) = raw.strip_suffix("_u32") {
			raw.parse::<u32>()
				.map(|v| Self::U32(v))
				.map_err(|_| ())
		} else if let Some(raw) = raw.strip_suffix("_i32") {
			raw.parse::<i32>()
				.map(|v| Self::I32(v))
				.map_err(|_| ())
 		} else if let Ok(string) = snailquote::unescape(raw) {
 			Ok(Self::String(Cow::Owned(string)))
 		}
 	}
}
