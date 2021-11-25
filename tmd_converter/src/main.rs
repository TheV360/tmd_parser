use std::io::{self, Write};
use std::fs::{read_dir, read, File};
use std::ffi::OsStr;

use tmd_parser::{Tmd, primitive};

fn main() -> io::Result<()> {
	for entry in read_dir("samples/")? {
		let entry = entry?;
		let path = entry.path();
		
		if path.is_file() {
			if let Some(ext) = path.extension().and_then(OsStr::to_str) {
				if ext == "TMD" {
					let name = path.file_stem()
						.map(|p| OsStr::to_string_lossy(p).to_string())
						.unwrap_or_else(|| "funny".to_string());
					let name = format!("{}{}", "fonts/".to_string(), name);
					
					let tmd = read(&path)?;
					let (_, tmd) = Tmd::parse(&tmd).expect("aauaaahgh");
					
					print!("Converting `{}`... ", name);
					
					make_obj(&tmd, &name)?; print!("OBJ! ");
					make_jhf_font(&tmd, &name)?; print!("JHF! ");
					
					println!("Done.");
				}
			}
		}
	}
	
	Ok(())
}

fn make_obj(tmd: &Tmd, name: &str) -> io::Result<()> {
	let mut f = File::create(format!("{}.obj", name))?;
	
	for (i, object) in tmd.obj_table.iter().enumerate() {
		if i > 0 { writeln!(&mut f)?; }
		writeln!(&mut f, "o obj{}", i)?;
		
		for vertex in object.vertices.iter() {
			writeln!(&mut f, "  v {} {} {}", vertex.x, vertex.y, vertex.z)?;
		}
		
		let vertices_len = object.vertices.len();
		
		for primitive in object.primitives.iter() {
			match primitive.data {
				primitive::PrimitiveData::Line { color: _, indices, } |
				primitive::PrimitiveData::LineGr { colors: _, indices } => {
					writeln!(&mut f, "  l {} {}", indices.0 as isize - vertices_len as isize, indices.1 as isize - vertices_len as isize)?;
				},
				_ => unimplemented!(),
			}
		}
	}
	
	Ok(())
}

fn make_jhf_font(tmd: &Tmd, name: &str) -> io::Result<()> {
	// chars too big. small them.
	const SMALL_YOUR_JHF: i16 = 5;
	
	// real game might actually have global lh/rh?
	const PADDING: i16 = 8;
	
	// pretending size isn't a plroblekrlm here.
	fn encode_coord(i: i16) -> char {
		((i / SMALL_YOUR_JHF) as i8 + b'R' as i8) as u8 as char
	}
	
	let mut f = File::create(format!("{}.jhf", name))?;
	
	// please don'thave empty obj table
	let first_i_prommy = tmd.obj_table.first().unwrap();
	let glh = first_i_prommy.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
	let grh = first_i_prommy.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
	let (glh, grh) = (-(glh.abs().max(grh.abs())), grh.abs().max(glh.abs()));
	let (glh, grh) = (glh - PADDING, grh + PADDING);
	
	// Write a space character, since the Vib-Ribbon font has none.
	let empty = format!("{:5} {}{}{}", 12345, 1, encode_coord(glh), encode_coord(grh));
	writeln!(&mut f, "{}", &empty)?;
	
	// sorry
	
	for object in tmd.obj_table.iter() {
		// I have to add 1 to len because the JHF format is freaky
		// and counts left/right hand as a coord pair.
		write!(&mut f, "{:5} {}", 12345, object.primitives.len() + 1)?;
		
		// Calculate left/right hand, via min/max coord from the vertices.
		// Maybe it's dangerous to use MIN/MAX here, but eh.
		let lh = object.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
		let rh = object.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
		// let (lh, rh) = (lh.min(rh), rh.max(lh));
		let (lh, rh) = (-(lh.abs().max(rh.abs())), rh.abs().max(lh.abs()));
		let (lh, rh) = (lh - PADDING, rh + PADDING);
		
		// Left/right hand values.
		write!(&mut f, "{}{}", encode_coord(lh), encode_coord(rh))?;
		
		let mut last_vert: Option<usize> = None;
		for primitive in object.primitives.iter() {
			match primitive.data {
				primitive::PrimitiveData::Line { color: _, indices, } |
				primitive::PrimitiveData::LineGr { colors: _, indices } => {
					let v1 = &object.vertices[indices.1];
					let v1 = (encode_coord(v1.x), encode_coord(v1.y));
					if last_vert != Some(indices.0) {
						let v0 = &object.vertices[indices.0];
						let v0 = (encode_coord(v0.x), encode_coord(v0.y));
						write!(&mut f, "{}{}", v0.0, v0.1)?;
					}
					write!(&mut f, "{}{}", v1.0, v1.1)?;
					if last_vert != Some(indices.0) {
						write!(&mut f, " R")?;
					}
					last_vert = Some(indices.1);
				},
				_ => unimplemented!(),
			}
		}
		
		writeln!(&mut f)?;
	}
	
	Ok(())
}
