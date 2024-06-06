use std::io::Read;
use gw2_pf_typegen as dut;

#[test]
fn find_magic() {
	let data = {
		let mut file = std::fs::File::open("C:/games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	dut::locate_structs(&data);
}