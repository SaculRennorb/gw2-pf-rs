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

	let file = &mut dut::pf::PackFileReader::<dut::formats::ABIX>::from_bytes(&data).map_err(|e| e.to_string()).unwrap();

	let chunk = file.next().unwrap().map_err(|e| e.to_string()).unwrap();

	assert_eq!(chunk.bank_language.len(), 6);
	assert_eq!(chunk.bank_language[0].bank_file_name.len(), 43769);
	assert_eq!(chunk.bank_language[0].bank_file_name[0].file_name.as_ref().unwrap().to_id(), 157442);
	assert_eq!(chunk.bank_language[0].bank_file_name[1000].file_name.as_ref().unwrap().to_id(), 0);
}

#[test] #[ignore = "long runtime"]
fn find_voice_id() {
	let search : u32 = 404553;

	let mut found = false;

	for path in std::fs::read_dir("tests/res/abix").unwrap() {
		let path = path.unwrap();
		if path.file_type().unwrap().is_dir() { continue }
		let filename = path.file_name();
		let filename = filename.to_str().unwrap();
		if !filename.ends_with(".raw") { continue }
		println!("{filename}");

		let data = {
			let mut file = File::open(format!("tests/res/abix/{filename}")).unwrap();
			let mut v = Vec::new();
			file.read_to_end(&mut v).unwrap();
			v
		};

		let file = match dut::pf::PackFileReader::<dut::formats::ABIX>::from_bytes(&data).map_err(|e| e.to_string()) { 
			Ok(s) => s,
			Err(e) => {
				println!("Outer file parse failed: {e}");
				continue;
			},
		};
		for chunk in file {
			let chunk = chunk.unwrap();

			for (lang_idx, maps) in chunk.bank_language.iter().enumerate() {
				for (data_idx, data) in maps.bank_file_name.iter().enumerate() {
					if data_idx as u32 == search || matches!(data.file_name, Some(ref n) if n.to_id() == search) {
						println!("lang: {lang_idx}, data: {data_idx}, {:?}", data.file_name.as_ref().map(|n| n.to_id()));
						found = true;
					}
				}
			}
		}
	}

	assert!(found);
}