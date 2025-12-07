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
			
			let dst_file = &mut std::fs::File::options().create(true).truncate(true).write(true).open(format!("tests/out/sounds/{}_{i}.{ext}", asnd_file.voice_id)).unwrap();
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


#[test]
fn decode() {
	let key = 12306624562963u64;

	let src_data =  {
		let mut file = File::open("tests/out/sounds/457667_0.bin").unwrap();
		let mut v = Vec::new();
		file.read_to_end(&mut v).unwrap();
		v
	};

	let mut dst_data = Vec::with_capacity(src_data.len());

	decrypt_content(unsafe{ std::slice::from_raw_parts_mut(dst_data.as_mut_ptr(), src_data.len()) }, &src_data, key).unwrap();
	unsafe{ dst_data.set_len(src_data.len()); }


	let dst_file = &mut File::create("tests/out/sounds/457667_0.mp3").unwrap();
	use std::io::Write;
	dst_file.write_all(&dst_data).unwrap();
}

fn decrypt_content(dst: &mut [u8], src: &[u8], decryption_key: u64) -> Result<(), ()> {
	if decryption_key == 0 { return Err(()); }

	use crypto::rc4::Rc4;
	use crypto::buffer::*;
	use crypto::symmetriccipher::Decryptor;

	let mut rc4 = Rc4::new(&hash_sha1_5_rounds(&decryption_key.to_le_bytes()));
	let mut read_buffer = RefReadBuffer::new(src);
	let mut write_buffer = RefWriteBuffer::new(dst);

	rc4.decrypt(&mut read_buffer, &mut write_buffer, true).map_err(|_| ())?;

	Ok(())
}

fn hash_sha1_5_rounds(buffer: &[u8]) -> [u8; 20] {
	let mut block = [0u32; 5];
	for i in 0..5 {
			block[i] = u32::from_le_bytes([
					buffer[(0 + i * std::mem::size_of::<u32>()) % buffer.len()],
					buffer[(1 + i * std::mem::size_of::<u32>()) % buffer.len()],
					buffer[(2 + i * std::mem::size_of::<u32>()) % buffer.len()],
					buffer[(3 + i * std::mem::size_of::<u32>()) % buffer.len()],
			]);
	}

	let digest: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

	let mut a = digest[0];
	let mut b = digest[1];
	let mut c = digest[2];
	let mut d = digest[3];
	let mut e = digest[4];

	macro_rules! round {
			($b:expr, $v:expr, $w:expr, $x:expr, $y:expr, $z:expr) => {{
					$z = $z
							.wrapping_add(($w & ($x ^ $y)) ^ $y)
							.wrapping_add($b)
							.wrapping_add(0x5a827999)
							.wrapping_add(u32::rotate_left($v, 5));
					$w = u32::rotate_left($w, 30);
			}};
	}
	round!(block[0], a, b, c, d, e);
	round!(block[1], e, a, b, c, d);
	round!(block[2], d, e, a, b, c);
	round!(block[3], c, d, e, a, b);
	round!(block[4], b, c, d, e, a);

	block[0] = a.wrapping_add(block[0]);
	block[1] = b.wrapping_add(block[1]);
	block[2] = c.wrapping_add(block[2]);
	block[3] = d.wrapping_add(block[3]);
	block[4] = e.wrapping_add(block[4]);


	unsafe{ *block.as_ptr().cast::<[u8; 20]>() }
}