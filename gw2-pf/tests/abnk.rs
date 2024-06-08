use std::{fs::File, io::Read};
use gw2_pf_rs as dut;


#[test] #[ignore = "produces files"]
fn extract_asnd_179764() {
	let data = {
		let mut file = File::open("tests/res/179764.abnk").unwrap();
		let mut v = Vec::new();
		file.read_to_end(&mut v).unwrap();
		v
	};

	extract_asnd(&data);
}

#[test] #[ignore = "produces files"]
fn extract_asnd_2324279() {
	let data = {
		let mut file = File::open("tests/res/2324279.abnk").unwrap();
		let mut v = Vec::new();
		file.read_to_end(&mut v).unwrap();
		v
	};

	extract_asnd(&data);
}

#[test] #[ignore = "produces files"]
fn extract_asnd_all() {
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

		extract_asnd(&data);
	}
}

fn extract_asnd(data : &[u8]) {
	let file = &mut dut::pf::PackFileReader::<dut::formats::ABNK>::from_bytes(&data).map_err(|e| e.to_string()).unwrap();
	let chunk = file.next().unwrap().map_err(|e| e.to_string()).unwrap();

	assert!(file.next().is_none());
	for asnd_file in &chunk.files {
		if asnd_file.audio_data.is_empty() { continue }
		let inner_file = dut::pf::PackFileReader::<dut::formats::ASND>::from_bytes(asnd_file.audio_data).map_err(|e| e.to_string()).unwrap();
		for (i, chunk) in inner_file.into_iter().enumerate() {
			let asnd_chunk = chunk.unwrap();

			let mp3 = asnd_chunk.audio_data;
			let ext;
			if &mp3[..2] != &[0xff, 0xfb] {
				println!("vid: {}/{i} OFlg: {:b}, IFlg: {:b}, Form: {}, Bytes: {:x?} unknown format", asnd_file.voice_id, asnd_file.flags, asnd_chunk.flags, asnd_chunk.format, &asnd_chunk.audio_data[..2]);
				ext = "bin";
			}
			else {
				println!("vid: {}/{i} OFlg: {:b}, IFlg: {:b}, Form: {}, Bytes: {:x?}", asnd_file.voice_id, asnd_file.flags, asnd_chunk.flags, asnd_chunk.format, &asnd_chunk.audio_data[..2]);
				ext = "mp3";
			}
			
			let dst_file = &mut std::fs::File::options().create(true).truncate(true).write(true).open(format!("tests/out/{}_{i}.{ext}", asnd_file.voice_id)).unwrap();
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

		let file = match dut::pf::PackFileReader::<dut::formats::ABNK>::from_bytes(&data).map_err(|e| e.to_string()) { 
			Ok(s) => s,
			Err(e) => {
				println!("Outer file parse failed: {e}");
				continue;
			},
		};
		for chunk in file {
			let abnk = chunk.unwrap();

			for asnd_file in &abnk.files {
				if asnd_file.audio_data.is_empty() { continue }
				let inner_file = dut::pf::PackFileReader::<dut::formats::ASND>::from_bytes(asnd_file.audio_data).map_err(|e| e.to_string()).unwrap();
				for chunk in inner_file {
					let asnd = chunk.unwrap();

					println!("OFlg: {:b}, IFlg: {:b}, Form: {}, Bytes: {:x?}", asnd_file.flags, asnd.flags, asnd.format, &asnd.audio_data[..2]);
				}
			}
		}
	}
}