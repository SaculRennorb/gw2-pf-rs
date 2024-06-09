#[derive(Debug, crate::Parse)]
pub struct TextPackVoices {
	/// OG: voices
	pub mappings : Vec<TextPackVoice>,
}

#[derive(Debug, crate::Parse)]
pub struct TextPackVoice {
	///  OG: textId
	pub text_id  : u32,
	///  OG: voiceId
	pub voice_id : u32,
}