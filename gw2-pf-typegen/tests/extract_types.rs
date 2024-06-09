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
fn dump_all() {
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

			dut::generate::rust::export_chunk(&self.0, fmt)
		}
	}

	struct VWrapper<'a>(dut::structure::SpecificChunkVersion<'a>);
	impl std::fmt::Display for VWrapper<'_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

			let mut imports = HashSet::new();
			dut::generate::rust::add_required_imports_for_type_recursive(&mut imports, &self.0);

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
			
			let linked_nonprimitive_types = &mut dut::generate::rust::RecursiveTypeReferences::new_with_seed(&self.0);
			for (i, _type) in linked_nonprimitive_types.into_iter().enumerate() {
				dut::generate::rust::export_type(_type, fmt)?;
				if i != linked_nonprimitive_types.len() - 1 { fmt.write_str("\n")?; }
			}
			Ok(())
		}
	}


	for chunk in dut::analyze::locate_chunks(&data) {
		let lower_chunk_magic = chunk.magic.to_lowercase();

		let chunk_path = &format!("tests/out/chunks/{lower_chunk_magic}{}", chunk.versions.iter().map(|v| v.version).max().unwrap());
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