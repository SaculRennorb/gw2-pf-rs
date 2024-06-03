use std::{fs::File, io::Read};
use gw2_pf_rs as dut;


#[test]
fn deserialize() {
	let data = {
		let mut file = File::open("tests/res/179764.abnk").unwrap();
		let mut v = Vec::new();
		file.read_to_end(&mut v).unwrap();
		v
	};

	let file = &mut dut::pf::PackFileReader::<dut::formats::ABNK>::from_bytes(&data).map_err(|e| e.to_string()).unwrap();
	let chunk = file.next().unwrap().map_err(|e| e.to_string()).unwrap();

	assert!(file.next().is_none());
	assert_eq!(chunk.files.len(), 10);
	for file in &chunk.files {
	println!("
	pub voice_id   : {},
	pub flags      : {},
	   _reserved1  : {},
	   _reserved2  : {},
	   _reserved3  : {},
	   _reserved4  : {},
	pub length     : {},
	pub offset     : {},
	   _reserved5  : {},
	   _reserved6  : {},
	   _reserved7  : {},
	   _reserved8  : {},
		 data: {:x?}", 
		 file. voice_id ,
		 file. flags    ,
				file._reserved1,
				file._reserved2,
				file._reserved3,
				file._reserved4,
		 file. length   ,
		 file. offset   ,
				file._reserved5,
				file._reserved6,
				file._reserved7,
				file._reserved8,
				&file.audio_data[..32]
	);
}
}