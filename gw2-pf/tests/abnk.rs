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
		let inner_file = &mut dut::pf::PackFileReader::<dut::formats::ASND>::from_bytes(file.audio_data).map_err(|e| e.to_string()).unwrap();
		for (i, chunk) in inner_file.enumerate() {
			let chunk = chunk.unwrap();

			let mp3 = chunk.audio_data;
			if &mp3[..2] != &[0xff, 0xfb] {
				println!("{:#?} unknown format {:x?}", chunk, &mp3[..2]);
			}
			
			let dst_file = &mut std::fs::File::options().create(true).truncate(true).write(true).open(format!("tests/out/{}_{}.mp3", file.voice_id, i)).unwrap();
			use std::io::Write;
			dst_file.write_all(mp3).unwrap();
		}
	}
}


#[test]
fn print_type() {
	for path in std::fs::read_dir("tests/res").unwrap() {
		let path = path.unwrap();
		if path.file_type().unwrap().is_dir() { continue }
		let filename = path.file_name();
		let filename = filename.to_str().unwrap();
		if !filename.ends_with(".abnk") { continue }
		println!("{filename}");

		let data = {
			let mut file = File::open(format!("tests/res/{filename}")).unwrap();
			let mut v = Vec::new();
			file.read_to_end(&mut v).unwrap();
			v
		};

		let file = &mut dut::pf::PackFileReader::<dut::formats::ABNK>::from_bytes(&data).map_err(|e| e.to_string()).unwrap();
		for chunk in file {
			let abnk = chunk.unwrap();

			for asnd_file in &abnk.files {
				let inner_file = &mut dut::pf::PackFileReader::<dut::formats::ASND>::from_bytes(asnd_file.audio_data).map_err(|e| e.to_string()).unwrap();
				for chunk in inner_file {
					let asnd = chunk.unwrap();

					println!("OFlg: {:b}, IFlg: {:b}, Form: {}, Bytes: {:x?}", asnd_file.flags, asnd.flags, asnd.format, &asnd.audio_data[..2]);
				}
			}
		}
	}
}