use std::{fs::File, io::Read};
use gw2_pf_rs as dut;



#[test]
fn deserialize() {
	let data = {
		let mut file = File::open("tests/res/184691").unwrap();
		let mut v = Vec::new();
		file.read_to_end(&mut v).unwrap();
		v
	};

	use dut::pf::PackFile;
	let parsed = dut::formats::abix::ABIX::from_bytes(&data).map_err(|e| e.to_string()).unwrap();

	fn to_id(_str : &dut::wstr::WString) -> u32 { //TODO @temp hackery for the trivial test
		if _str.0.is_empty() {
			0
		} else {
				((_str.0[1] as i32 * 0xff00) + (_str.0[0] as i32 - 0xff00ff)) as u32
		}
	}

	assert_eq!(parsed.chunks[0].bank_language.len(), 6);
	assert_eq!(parsed.chunks[0].bank_language[0].bank_file_name.len(), 43769);
	assert_eq!(to_id(parsed.chunks[0].bank_language[0].bank_file_name[0].file_name.as_ref().unwrap()), 157442);
	assert_eq!(to_id(parsed.chunks[0].bank_language[0].bank_file_name[1000].file_name.as_ref().unwrap()), 0);
}