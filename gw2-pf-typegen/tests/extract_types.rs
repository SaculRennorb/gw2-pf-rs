use std::{collections::HashSet, io::Read};
use gw2_pf_typegen as dut;

#[test]
fn find_types() {
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
fn format_eula() {
	let data = {
		let mut file = std::fs::File::open("C:/games/Guild Wars 2/Gw2-64.exe").unwrap();
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).unwrap();
		buffer
	};

	let eula = dut::analyze::locate_chunks(&data).filter(|c| c.magic == "eula").next().unwrap();

	println!("{}", Wrapper(eula));

	struct Wrapper<'a>(dut::structure::Chunk<'a>);
	impl std::fmt::Display for Wrapper<'_> {
		fn fmt(&self, fmt : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			dut::generate::rust::export_chunk(&self.0, fmt)?;
			fmt.write_str("\n\n")?;
			

			for version in self.0.versions.iter() {
				fmt.write_fmt(format_args!("pub mod v{} {{\n", version.version))?;
				let linked_nonprimitive_types = &mut dut::generate::rust::RecursiveTypeReferences::new_with_seed(&version.root);

				let mut import_set = HashSet::new();
				for _type in linked_nonprimitive_types.into_iter() {
					for field in _type.fields.iter() {
						if let Some(import) = dut::generate::rust::get_required_import_for_field(field) {
							import_set.insert(import);
						}
					}
				}

				if !import_set.is_empty() {
					fmt.write_str("use crate::{")?;
					let mut first = true;
					for import in import_set {
						fmt.write_str(import)?;
						fmt.write_str(", ")?;
					}
					fmt.write_str("};\n\n")?;
				}

				for _type in linked_nonprimitive_types.into_iter() {
					dut::generate::rust::export_struct(_type, fmt)?;
					fmt.write_str("\n")?;
				}
				fmt.write_str("}\n")?;
			}

			Ok(())
		}
	}
}