use std::{collections::HashSet, fs::File, io::{Read, Write}};
use gw2_pf_typegen as dut;

#[test]
fn dump_chunks() {
	let data = {
		let mut file = std::fs::File::open("C:/games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	for _struct in dut::analyze::locate_chunks(&data) {
		println!("{_struct:#?}");
	}
}

#[test]
fn format_asnd() {
	let data = {
		let mut file = std::fs::File::open("C:/games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	let chunk_info = dut::analyze::locate_chunks(&data).filter(|c| c.magic == "ASND").next().unwrap();

	println!("{}", Wrapper(chunk_info));

	struct Wrapper<'a>(dut::structure::Chunk<'a>);
	impl std::fmt::Display for Wrapper<'_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			dut::generate::rust::export_chunk(&self.0, fmt)?;
			fmt.write_str("\n\n")?;
			

			for version in self.0.versions.iter() {
				fmt.write_fmt(format_args!("pub mod v{} {{\n", version.version))?;

				let mut imports = HashSet::new();
				dut::generate::rust::add_required_imports_for_type_recursive(&mut imports, version);

				if !imports.is_empty() {
					fmt.write_str("use crate::{")?;
					for import in imports {
						fmt.write_str(import)?;
						fmt.write_str(", ")?;
					}
					fmt.write_str("};\n\n")?;
				}
				
				let linked_nonprimitive_types = &mut dut::generate::rust::RecursiveTypeReferences::new_with_seed(&version.root);
				for _type in linked_nonprimitive_types.into_iter() {
					dut::generate::rust::export_type(_type, fmt)?;
					fmt.write_str("\n")?;
				}
				fmt.write_str("}\n")?;
			}

			Ok(())
		}
	}
}


#[test] #[ignore = "produces files"]
fn dump_all_rs() {
	use dut::generate::rust as lang;
	let out_path = "out";

	let data = {
		let mut file = std::fs::File::open("C:/games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	struct CWrapper<'a, 'b>(&'b dut::structure::Chunk<'a>);
	impl std::fmt::Display for CWrapper<'_, '_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			for version in &self.0.versions {
				fmt.write_fmt(format_args!("pub mod v{};\n", version.version))?;
			}
			fmt.write_str("\n\n")?;

			lang::export_chunk(&self.0, fmt)
		}
	}

	struct VWrapper<'a>(dut::structure::SpecificChunkVersion<'a>);
	impl std::fmt::Display for VWrapper<'_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

			let mut imports = HashSet::new();
			lang::add_required_imports_for_type_recursive(&mut imports, &self.0);

			match imports.len() {
				0 => {},
				1 => fmt.write_fmt(format_args!("use crate::{};\n\n\n", imports.iter().next().unwrap()))?,
				_ => {
					fmt.write_str("use crate::{")?;
					let mut first = true;
					for import in imports {
						if first { first = false } else { fmt.write_str(", ")?; }
						fmt.write_str(import)?;
					}
					fmt.write_str("};\n\n\n")?;
				}
			}
			
			let linked_nonprimitive_types = &mut lang::RecursiveTypeReferences::new_with_seed(&self.0);
			for (i, _type) in linked_nonprimitive_types.into_iter().enumerate() {
				lang::export_type(_type, fmt)?;
				if i != linked_nonprimitive_types.len() - 1 { fmt.write_str("\n")?; }
			}
			Ok(())
		}
	}


	for chunk in dut::analyze::locate_chunks(&data) {
		let lower_chunk_magic = chunk.magic.to_lowercase();

		let chunk_path = &format!("tests/{out_path}/chunks/{lower_chunk_magic}{}", chunk.versions.iter().map(|v| v.version).max().unwrap());
		let chunk_file_path = format!("{chunk_path}/{lower_chunk_magic}.rs");
		if std::path::Path::new(&chunk_file_path).exists() {
			eprintln!("[warn] Path for chunk {} ({chunk_file_path}) already exists, skipping write.", chunk.magic);
		}
		else {
			println!("{}", chunk.magic);

			_ = std::fs::create_dir(chunk_path);
			
			let chunk_file = &mut File::create(chunk_file_path).unwrap();
			write!(chunk_file, "{}", CWrapper(&chunk)).unwrap();
		}
		

		for version in chunk.versions {
			let version_path = format!("{chunk_path}/v{}.rs", version.version);
			if std::path::Path::new(&version_path).exists() {
				eprintln!("[warn] Path for version {} of chunk {} ({version_path}) already exists, skipping write.", version.version, chunk.magic);
				continue;
			}

			print!(" v{}", version.version);

			let version_file = &mut File::create(version_path).unwrap();
			write!(version_file, "{}", VWrapper(version)).unwrap();
		}

		println!();
	}
}


#[test] #[ignore = "produces files"]
fn dump_all_odin() {
	use dut::generate::odin as lang;
	let out_path = "out_odin";

	let data = {
		let mut file = std::fs::File::open("F:/Games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	struct CWrapper<'a, 'b>(&'b dut::structure::Chunk<'a>);
	impl std::fmt::Display for CWrapper<'_, '_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			fmt.write_fmt(format_args!("package gw2_pf_REPLACE_ME_chunks_{};\n\n", self.0.magic))?;
			fmt.write_str("import \"base:runtime\"\n")?;
			fmt.write_str("import common \"../../../../../\"\n")?;
			fmt.write_str("import pf \"../../../../\"\n")?;
			for (i, version) in self.0.versions.iter().enumerate() {
				if i > 0 { fmt.write_str("; ")?; }
				fmt.write_fmt(format_args!("import \"v{}\"", version.version))?;
			}
			fmt.write_str("\n\n")?;
			fmt.write_fmt(format_args!("Magic := common.fourcc_magic(\"{}\")\n\n", self.0.magic))?;

			lang::export_chunk(&self.0, fmt)?;

			fmt.write_str("\n")?;

			for version in &self.0.versions {
				fmt.write_fmt(format_args!("v{} :: v{}.Chunk\n", version.version, version.version))?;
			}

			fmt.write_str("\n")?;

			fmt.write_str("read :: proc(reader : ^pf.Reader, version : u32, destination : ^Chunk) -> (err : common.ParserError)\n")?;
			fmt.write_str("{\n")?;
			fmt.write_str("\tswitch(version) {\n")?;
			let has_versions_needing_padding = self.0.versions.iter().any(|v| v.version < 10) && self.0.versions.iter().any(|v| v.version >= 10);
			for version in &self.0.versions {
				let vpad = if has_versions_needing_padding && version.version < 10 { " " } else { "" };
				fmt.write_fmt(format_args!("\t\tcase {vpad}{ver}: chunk : {vpad}v{ver}.Chunk; if {vpad}v{ver}.read(reader, &chunk) {{ destination^ = chunk; return }}\n", ver = version.version))?;
			}
			fmt.write_str(r#"		case:
			return common.UnknownVersion {
				offset = u64(uintptr(reader.cursor) - uintptr(reader.begin)),
				actual = version,
			}
	}

	return common.OutOfData { offset = u64(uintptr(reader.cursor) - uintptr(reader.begin)) }
}
"#)?;

			Ok(())
		}
	}

	struct VWrapper<'a>(&'a str, dut::structure::SpecificChunkVersion<'a>);
	impl std::fmt::Display for VWrapper<'_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			fmt.write_fmt(format_args!("package gw2_pf_REPLACE_ME_chunks_{}_v{};\n\n", self.0, self.1.version))?;
			fmt.write_str("import pf \"../../../../../\"\n\n")?;
			fmt.write_fmt(format_args!("Chunk :: {}\n\n", lang::format_type_name(&self.1.root)))?;

			let linked_nonprimitive_types = &mut lang::RecursiveTypeReferences::new_with_seed(&self.1);

			fmt.write_str("read :: proc { ")?;
			for _type in linked_nonprimitive_types.iter() {
				fmt.write_fmt(format_args!("read_{}, ", lang::format_type_name(_type)))?;
			}
			fmt.write_str("}\n\n")?;

			for _type in linked_nonprimitive_types.iter() {
				lang::export_type(_type, fmt)?;
				fmt.write_str("\n")?;
			}

			for (i, _type) in linked_nonprimitive_types.iter().enumerate() {
				lang::export_type_parser(_type, fmt)?;
				if i != linked_nonprimitive_types.len() - 1 { fmt.write_str("\n")?; }
			}

			Ok(())
		}
	}


	for chunk in dut::analyze::locate_chunks(&data) {
		let chunk_path = &format!("tests/{out_path}/chunks/{}{}", chunk.magic, chunk.versions.iter().map(|v| v.version).max().unwrap());
		let chunk_file_path = format!("{chunk_path}/{}.odin", chunk.magic);
		if std::path::Path::new(&chunk_file_path).exists() {
			eprintln!("[warn] Path for chunk {} ({chunk_file_path}) already exists, skipping write.", chunk.magic);
		}
		else {
			println!("{}", chunk.magic);

			std::fs::create_dir_all(chunk_path).unwrap();
			
			let chunk_file = &mut File::create(chunk_file_path).unwrap();
			write!(chunk_file, "{}", CWrapper(&chunk)).unwrap();
		}
		

		for version in chunk.versions {
			let chunk_version_path = format!("{chunk_path}/v{}", version.version);
			let chunk_version_file_path = format!("{chunk_version_path}/{}-v{}.odin", chunk.magic, version.version);
			if std::path::Path::new(&chunk_version_file_path).exists() {
				eprintln!("[warn] Path for version {} of chunk {} ({chunk_version_file_path}) already exists, skipping write.", version.version, chunk.magic);
				continue;
			}
			
			std::fs::create_dir(chunk_version_path).unwrap();

			print!(" v{}", version.version);

			let version_file = &mut File::create(chunk_version_file_path).unwrap();
			write!(version_file, "{}", VWrapper(chunk.magic, version)).unwrap();
		}

		println!();
	}
}