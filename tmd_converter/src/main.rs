use std::io::{self, Write};
use std::fs::{File, read, read_dir};
use std::ffi::OsStr;

use tmd_parser::{Tmd, primitive, object::Vector};

fn main() -> io::Result<()> {
	// create_dir("out/")?;
	// create_dir("out/models/")?;
	// create_dir("out/fonts/")?;
	
	println!("Shrinking vertices by a factor of 50. If this is not okay, uh. Idk");
	
	for entry in read_dir("samples/")? {
		let entry = entry?;
		let path = entry.path();
		
		if path.is_file() {
			if let Some(ext) = path.extension().and_then(OsStr::to_str) {
				if ext == "TMD" {
					let name = path.file_stem()
						.map(|p| OsStr::to_string_lossy(p).to_string())
						.unwrap_or_else(|| "funny".to_string());
					
					let tmd = read(&path)?;
					let (_, tmd) = Tmd::parse(&tmd).expect("Failed to parse TMD.");
					
					print!("Converting `{}`... ", name);
					
					make_obj(&tmd, &format!("{}{}", "out/models/", &name))?; print!("OBJ! ");
					make_jhf_font(&tmd, &format!("{}{}", "out/fonts/", &name))?; print!("JHF! ");
					
					println!("Done.");
				}
			}
		}
	}
	
	Ok(())
}

#[derive(Debug, Default)]
struct Bounds {
	min: Vector,
	max: Vector,
}
impl Bounds {
	fn look(&mut self, v: &Vector) {
		self.min.x = self.min.x.min(v.x);
		self.min.y = self.min.y.min(v.y);
		self.min.z = self.min.z.min(v.z);
		
		self.max.x = self.max.x.max(v.x);
		self.max.y = self.max.y.max(v.y);
		self.max.z = self.max.z.max(v.z);
	}
}

fn make_obj(tmd: &Tmd, name: &str) -> io::Result<()> {
	const SMALL_YOUR_OBJ: f64 = 50.0_f64;
	
	let mut f = File::create(format!("{}.obj", name))?;
	
	let mut bounds = Bounds::default();
	
	for (i, object) in tmd.obj_table.iter().enumerate() {
		if i > 0 { writeln!(&mut f)?; }
		writeln!(&mut f, "o obj{}", i)?;
		
		for vertex in object.vertices.iter() {
			let rv = (vertex.x as f64 / SMALL_YOUR_OBJ, vertex.y as f64 / SMALL_YOUR_OBJ, vertex.z as f64 / SMALL_YOUR_OBJ);
			writeln!(&mut f, "  v {} {} {}", rv.0, rv.1, rv.2)?;
			bounds.look(vertex);
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
	
	// TODO: use bounds or something
	
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
	
	// please don't have an empty obj table.
	let first_i_prommy = tmd.obj_table.first().unwrap();
	let glh = first_i_prommy.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
	let grh = first_i_prommy.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
	let (glh, grh) = (-(glh.abs().max(grh.abs())), grh.abs().max(glh.abs()));
	let (glh, grh) = (glh - PADDING, grh + PADDING);
	
	// Write a space character, since the Vib-Ribbon font has none.
	let empty = format!("{:5} {}{}{}", 12345, 1, encode_coord(glh), encode_coord(grh));
	writeln!(&mut f, "{}", &empty)?;
	
	for object in tmd.obj_table.iter() {
		// I have to add 1 to len because the JHF format is freaky
		// and counts left/right hand as a coord pair.
		// Also, each primitive contains two points.
		// TODO: This is annoying -- if I even try to optimize the mesh later,
		//       this vertex total becomes inaccurate.
		write!(&mut f, "{:5} {}", 12345, object.primitives.len() * 2 + 1)?;
		
		// Calculate left/right hand, via min/max coord from the vertices.
		// Maybe it's dangerous to use MIN/MAX here, but eh.
		let lh = object.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
		let rh = object.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
		// let (lh, rh) = (lh.min(rh), rh.max(lh));
		let (lh, rh) = (-(lh.abs().max(rh.abs())), rh.abs().max(lh.abs()));
		let (lh, rh) = (lh - PADDING, rh + PADDING);
		
		// Left/right hand values.
		write!(&mut f, "{}{}", encode_coord(lh), encode_coord(rh))?;
		
		// TODO: Try optimizing the mesh when this format matters. (never)
		for primitive in object.primitives.iter() {
			match primitive.data {
				primitive::PrimitiveData::Line { color: _, indices, } |
				primitive::PrimitiveData::LineGr { colors: _, indices } => {
					let (v0, v1) = (&object.vertices[indices.0], &object.vertices[indices.1]);
					let v0 = (encode_coord(v0.x), encode_coord(v0.y));
					let v1 = (encode_coord(v1.x), encode_coord(v1.y));
					write!(&mut f, "{}{}{}{} R", v0.0, v0.1, v1.0, v1.1)?;
				},
				_ => unimplemented!(),
			}
		}
		
		// Onto the next character!
		writeln!(&mut f)?;
	}
	
	Ok(())
}
