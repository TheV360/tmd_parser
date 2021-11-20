use nom::{
	IResult,
	combinator::{into, map, peek},
	multi::count,
	number::complete::{le_i16, le_i32, le_u16, le_u32, u8},
	sequence::tuple
};

#[derive(Debug)]
pub struct Tmd {
	pub header: Header,
	pub obj_table: Vec<Object>,
}

#[derive(Debug)]
pub struct Header {
	pub id: u32,
	pub flags: HeaderFlags,
	pub n_obj: usize,
}

#[derive(Debug)]
pub struct HeaderFlags {
	pub fix_p: bool,
}
impl From<u32> for HeaderFlags {
	fn from(f: u32) -> Self {
		HeaderFlags { fix_p: f & 1 > 0 }
	}
}

#[derive(Debug)]
pub struct ObjectEntry {
	pub vert_top: u32,
	pub n_vert: u32,
	pub normal_top: u32,
	pub n_normal: u32,
	pub primitive_top: u32,
	pub n_primitive: u32,
	pub scale: i32,
}

#[derive(Debug)]
pub struct Object {
	pub vertices: Vec<Vector>,
	pub normals: Vec<Normal>,
	pub primitives: Vec<Primitive>,
	
	/// This maaaaayyybeee dictates the scaling of the vertices after everything
	/// is loaded? It's encoded as `2**scale`, so that means
	/// `-2 = 1/4`, `-1 = 1/2`, `0 = 1`, `1 = 2`, and `2 = 4`.
	/// It's also completely unused by everyone.
	pub scale: i32,
}

#[derive(Debug)]
pub struct Primitive {
	pub olen: u8,
	pub ilen: u8,
	pub flag: PrimitiveFlags,
	pub mode: PrimitiveMode,
	pub data: PrimitiveData,
}

#[derive(Debug)]
pub struct PrimitiveFlags {
	/// Should the polygon should have gradation?
	/// Valid only for non-textured polygons subject to light source calcs.
	pub grd: bool,
	
	/// Should the polygon should be double-faced?
	/// Valid only for polygons.
	pub fce: bool,
	
	/// Should light source calculations happen?
	pub lgt: bool,
}
impl From<u8> for PrimitiveFlags {
	fn from(f: u8) -> Self {
		PrimitiveFlags {
			grd: f & 4 > 0,
			fce: f & 2 > 0,
			lgt: f & 1 > 0,
		}
	}
}

#[derive(Debug)]
pub enum PrimitiveMode {
	Polygon { iip: bool, more: bool, tme: bool, abe: bool, tge: bool },
	Line { iip: bool, abe: bool },
	Sprite { siz: PrimitiveSpriteSize, abe: bool },
}
impl From<u8> for PrimitiveMode {
	fn from(f: u8) -> Self {
		use PrimitiveMode::*;
		match f >> 5 {
			1 => Polygon {
				iip: f & 16 > 0,
				more: f & 8 > 0, // has 4th vertex?
				tme: f & 4 > 0,
				abe: f & 2 > 0,
				tge: f & 1 > 0,
			},
			2 => Line {
				iip: f & 16 > 0,
				abe: f & 2 > 0,
			},
			3 => Sprite {
				siz: PrimitiveSpriteSize::from(f),
				abe: f & 2 > 0,
			},
			_ => unimplemented!(),
		}
	}
}

#[derive(Debug)]
pub enum PrimitiveSpriteSize { Free, S1x1, S8x8, S16x16, }
impl From<u8> for PrimitiveSpriteSize {
	fn from(f: u8) -> Self {
		use PrimitiveSpriteSize::*;
		match f >> 3 & 3 {
			0 => Free, 1 => S1x1,
			2 => S8x8, 3 => S16x16,
			_ => unreachable!(),
		}
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PrimitiveData {
	Line { colors: (primitive::Color, primitive::Color), indices: (usize, usize) }
}

#[derive(Debug)]
pub struct Vector {
	pub x: i16, pub y: i16, pub z: i16,
}
impl From<Normal> for Vector {
	fn from(n: Normal) -> Self {
		Vector {
			x: (n.x * NORMAL_ACC) as i16,
			y: (n.y * NORMAL_ACC) as i16,
			z: (n.z * NORMAL_ACC) as i16,
		}
	}
}

const NORMAL_ACC: f32 = 0x1000 as f32;
#[derive(Debug)]
pub struct Normal {
	pub x: f32, pub y: f32, pub z: f32,
}
impl From<Vector> for Normal {
	fn from(v: Vector) -> Self {
		Normal {
			x: v.x as f32 / NORMAL_ACC,
			y: v.y as f32 / NORMAL_ACC,
			z: v.z as f32 / NORMAL_ACC,
		}
	}
}

fn parse_tmd(data: &[u8]) -> IResult<&[u8], Tmd> {
	let (data, header) = header(data)?;
	
	let entire = {
		if header.flags.fix_p {
			unimplemented!()
		} else { data }
	};
	
	let (data, obj_table) = count(object_canon(entire), header.n_obj)(data)?;
	
	let tmd = Tmd { header, obj_table, };
	Ok((data, tmd))
}

fn header(data: &[u8]) -> IResult<&[u8], Header> {
	let (data, (id, flags, n_obj)) = tuple((le_u32, le_u32, le_u32))(data)?;
	let flags = HeaderFlags::from(flags);
	let n_obj = n_obj as usize;
	Ok((data, Header { id, flags, n_obj }))
}

fn object(data: &[u8]) -> IResult<&[u8], ObjectEntry> {
	let mut size_ptr = tuple((le_u32, le_u32));
	
	let (data, (vert_top, n_vert)) = size_ptr(data)?;
	let (data, (normal_top, n_normal)) = size_ptr(data)?;
	let (data, (primitive_top, n_primitive)) = size_ptr(data)?;
	
	let (data, scale) = le_i32(data)?;
	
	let entry = ObjectEntry {
		vert_top, n_vert,
		normal_top, n_normal,
		primitive_top, n_primitive,
		scale,
	};
	Ok((data, entry))
}

/// This takes a slice of all data past the header and returns a function that,
/// when called, will maybe return an `Object` with all its vertices, normals,
/// and primitives. It can, however, also return an error.
fn object_canon<'a>(all_data: &'a [u8]) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Object> {
	move |data: &'a [u8]| {
		fn le_u32_index(data: &[u8]) -> IResult<&[u8], usize> {
			map(le_u32, |i| i as usize)(data)
		}
		
		let (data, (vertex_ptr, vertex_len)) = tuple((le_u32_index, le_u32_index))(data)?;
		let (data, (normal_ptr, normal_len)) = tuple((le_u32_index, le_u32_index))(data)?;
		let (data, (primitive_ptr, primitive_len)) = tuple((le_u32_index, le_u32_index))(data)?;
		
		let (data, scale) = le_i32(data)?;
		
		let (_, vertices) = count(vertex, vertex_len)(&all_data[vertex_ptr..])?;
		let (_, normals) = count(normal, normal_len)(&all_data[normal_ptr..])?;
		let (_, primitives) = count(primitive, primitive_len)(&all_data[primitive_ptr..])?;
		
		Ok((data, Object { vertices, normals, primitives, scale, }))
	}
}

fn vertex(data: &[u8]) -> IResult<&[u8], Vector> {
	let (data, (x, y, z, _)) = tuple((le_i16, le_i16, le_i16, le_i16))(data)?;
	Ok((data, Vector { x, y, z }))
}

fn normal(data: &[u8]) -> IResult<&[u8], Normal> { into(vertex)(data) }

fn primitive(data: &[u8]) -> IResult<&[u8], Primitive> {
	let (data, (olen, ilen, flag, mode)) = tuple((u8, u8, u8, u8))(data)?;
	
	let mode = PrimitiveMode::from(mode);
	let flag = PrimitiveFlags::from(flag);
	
	fn le_u16_index(data: &[u8]) -> IResult<&[u8], usize> {
		map(le_u16, |i| i as usize)(data)
	}
	
	// line olen 3, ilen 2, flag 0x1, mode 64.
	let (data, p_data) = match mode {
		PrimitiveMode::Line { abe: _, iip } => {
			let (data, colors) = {
				if iip { // 2 colors
					tuple((primitive::color, primitive::color))(data)?
				} else { // 1 color (twice)
					tuple((peek(primitive::color), primitive::color))(data)?
				}
			};
			let (data, indices) = tuple((le_u16_index, le_u16_index))(data)?;
			(data, PrimitiveData::Line { colors, indices })
		},
		_ => unimplemented!(), // FIXME: primitives aren't completely done
	};
	
	Ok((data, Primitive { olen, ilen, flag, mode, data: p_data }))
}

mod primitive {
	use super::*;
	
	#[derive(Debug, Clone, Copy)]
	pub struct Color {
		pub r: u8, pub g: u8, pub b: u8,
	}
	
	pub fn color(data: &[u8]) -> IResult<&[u8], Color> {
		let (data, (r, g, b, _)) = tuple((u8, u8, u8, u8))(data)?;
		Ok((data, Color { r, g, b, }))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn decode_thing() -> std::io::Result<()> {
		use std::fs;
		use std::io::Write;
		use std::fs::{read_dir, read};
		use std::ffi::OsStr;
		
		fn make_obj(tmd: &Tmd, name: &str) -> std::io::Result<()> {
			let mut f = fs::File::create(format!("{}.obj", name))?;
			
			for (i, object) in tmd.obj_table.iter().enumerate() {
				if i > 0 { writeln!(&mut f)?; }
				writeln!(&mut f, "  o obj{}", i)?;
				
				for vertex in object.vertices.iter() {
					writeln!(&mut f, "  v {} {} {}", vertex.x, vertex.y, vertex.z)?;
				}
				
				for primitive in object.primitives.iter() {
					match primitive.data {
						PrimitiveData::Line { colors: _, indices, } => {
							writeln!(&mut f, "  l {} {}", indices.0 + 1, indices.1 + 1)?;
						},
						_ => unimplemented!(),
					}
				}
			}
			
			Ok(())
		}
		
		fn make_jhf_font(tmd: &Tmd, name: &str) -> std::io::Result<()> {
			// chars too big. small them.
			const SMALL_YOUR_JHF: i16 = 5;
			
			// real game might actually have global lh/rh?
			const PADDING: i16 = 8;
			
			// pretending size isn't a plroblekrlm here.
			fn encode_coord(i: i16) -> char {
				((i / SMALL_YOUR_JHF) as i8 + b'R' as i8) as u8 as char
			}
			
			let mut f = fs::File::create(format!("{}.jhf", name))?;
			
			// please don'thave empty obj table
			let first_i_prommy = tmd.obj_table.first().unwrap();
			let glh = first_i_prommy.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
			let grh = first_i_prommy.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
			let (glh, grh) = (-(glh.abs().max(grh.abs())), grh.abs().max(glh.abs()));
			let (glh, grh) = (glh - PADDING, grh + PADDING);
			
			let empty = format!("{:5} {}{}{}", 12345, 1, encode_coord(glh), encode_coord(grh));
			writeln!(&mut f, "{}", &empty)?;
			
			// sorry
			
			for object in tmd.obj_table.iter() {
				write!(&mut f, "{:5} {}", 12345, object.primitives.len() + 1)?; // must take into account left/right hand
				
				let lh = object.vertices.iter().fold(i16::MAX, |a, v| a.min(v.x));
				let rh = object.vertices.iter().fold(i16::MIN, |a, v| a.max(v.x));
				
				// let (lh, rh) = (lh.min(rh), rh.max(lh));
				let (lh, rh) = (-(lh.abs().max(rh.abs())), rh.abs().max(lh.abs()));
				
				let (lh, rh) = (lh - PADDING, rh + PADDING);
				
				write!(&mut f, "{}{}", encode_coord(lh), encode_coord(rh))?;
				
				let mut last_vert: Option<usize> = None;
				for primitive in object.primitives.iter() {
					match primitive.data {
						PrimitiveData::Line { colors: _, indices, } => {
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
						let (_, tmd) = parse_tmd(&tmd).expect("aauaaahgh");
						
						make_obj(&tmd, &name)?;
						make_jhf_font(&tmd, &name)?;
					}
				}
			}
		}
		
		Ok(())
	}
}
