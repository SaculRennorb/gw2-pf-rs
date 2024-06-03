#[derive(Debug, serde::Deserialize)]
pub struct WaveformData {
	pub length        : f32,
	pub offset        : f32,
	   _reserved_data : u32, //ptr
	   _reserved1     : u32,
	   _reserved2     : u32,
	pub crc           : u32,
	pub num_samples   : u32,
	pub loop_start    : u32,
	pub loop_end      : u32,
	pub flags         : u32,
	pub format        : u8,
	   _reserved3     : u8,
	   _reserved4     : u8,
	   _reserved5     : u8,
	pub num_channels  : u8,
	   _reserved6     : u8,
	   _reserved7     : u8,
	   _reserved8     : u8,
	pub audio_data    : Vec<u8>,
	pub other_data    : Vec<u8>,
}