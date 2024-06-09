use std::{fs::File, io::Read};
use gw2_pf_rs as dut;

#[test] #[ignore = "produces files"]
fn extract_all() {
	for path in std::fs::read_dir("tests/res").unwrap() {
		let path = path.unwrap();
		if path.file_type().unwrap().is_dir() { continue }
		let filename = path.file_name();
		let filename = filename.to_str().unwrap();
		if !filename.ends_with(".txtv") { continue }
		println!("{filename}");

		let data = {
			let mut file = File::open(format!("tests/res/{filename}")).unwrap();
			let mut v = Vec::new();
			file.read_to_end(&mut v).unwrap();
			v
		};

		extract(&data, filename);
	}
}

fn extract(data : &[u8], filename : &str) {
	let file = &mut dut::pf::PackFileReader::<dut::formats::txtv>::from_bytes(&data).map_err(|e| e.to_string()).unwrap();

	let destination = &mut File::options().create(true).truncate(true).write(true).open(format!("tests/out/{filename}.csv")).unwrap();

	use std::io::Write;
	writeln!(destination, "textId;voiceId").unwrap();

	for chunk in file {
		let chunk = chunk.unwrap();
		for mapping in &chunk.mappings {
			writeln!(destination, "{};{}", mapping.text_id, mapping.voice_id).unwrap();
		}
	}
}