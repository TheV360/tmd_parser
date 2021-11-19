use nom::{
	IResult,
	combinator::{into, map},
	multi::count,
	number::complete::{u8, le_u32, le_i32, le_i16},
	sequence::tuple
};

#[derive(Debug)]
pub struct TmdInternal {
	header: Header,
	obj_table: Vec<Object>,
}

#[derive(Debug)]
pub struct Header {
	id: u32,
	flags: HeaderFlags,
	n_obj: usize,
}

#[derive(Debug)]
pub struct HeaderFlags {
	fix_p: bool,
}
impl From<u32> for HeaderFlags {
	fn from(f: u32) -> Self {
		HeaderFlags { fix_p: f & 1 > 0 }
	}
}

#[derive(Debug)]
pub struct ObjectEntry {
	vert_top: u32,
	n_vert: u32,
	normal_top: u32,
	n_normal: u32,
	primitive_top: u32,
	n_primitive: u32,
	scale: i32,
}

#[derive(Debug)]
pub struct Object {
	vertices: Vec<Vector>,
	normals: Vec<Normal>,
	primitives: Vec<Primitive>,
	
	/// This maaaaayyybeee dictates the scaling of the vertices after everything
	/// is loaded? It's encoded as `2**scale`, so that means
	/// `-2 = 1/4`, `-1 = 1/2`, `0 = 1`, `1 = 2`, and `2 = 4`.
	scale: i32,
}

#[derive(Debug)]
pub struct Primitive {
	olen: u8,
	ilen: u8,
	flag: PrimitiveFlags,
	mode: PrimitiveMode,
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
pub struct PrimitiveFlags {
	/// Should the polygon should have gradation?
	/// Valid only for non-textured polygons subject to light source calcs.
	grd: bool,
	
	/// Should the polygon should be double-faced?
	/// Valid only for polygons.
	fce: bool,
	
	/// Should light source calculations happen?
	lgt: bool,
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

fn parse_tmd(data: &[u8]) -> IResult<&[u8], TmdInternal> {
	let (data, header) = header(data)?;
	
	let entire = {
		if header.flags.fix_p {
			unimplemented!()
		} else { data }
	};
	
	let (data, obj_table) = count(object_canon(entire), header.n_obj)(data)?;
	
	let tmd = TmdInternal { header, obj_table, };
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
		fn le_u32_to_usize(data: &[u8]) -> IResult<&[u8], usize> {
			map(le_u32, |i| i as usize)(data)
		}
		
		let (data, (vertex_ptr, vertex_len)) = tuple((le_u32_to_usize, le_u32_to_usize))(data)?;
		let (data, (normal_ptr, normal_len)) = tuple((le_u32_to_usize, le_u32_to_usize))(data)?;
		let (data, (primitive_ptr, primitive_len)) = tuple((le_u32_to_usize, le_u32_to_usize))(data)?;
		
		let (data, scale) = le_i32(data)?;
		
		let (_, vertices) = count(vertex, vertex_len)(&all_data[vertex_ptr..])?;
		let (_, normals) = count(normal, normal_len)(&all_data[normal_ptr..])?;
		// let (_, primitives) = count(primitive, primitive_len)(&all_data[primitive_ptr..])?;
		let primitives = Vec::new();
		// FOXME: primitives simply aren't parsed right now.
		
		Ok((data, Object { vertices, normals, primitives, scale }))
	}
}

fn vertex(data: &[u8]) -> IResult<&[u8], Vector> {
	let (data, (x, y, z, _)) = tuple((le_i16, le_i16, le_i16, le_i16))(data)?;
	Ok((data, Vector { x, y, z }))
}

fn normal(data: &[u8]) -> IResult<&[u8], Normal> { into(vertex)(data) }

fn primitive(data: &[u8]) -> IResult<&[u8], Primitive> {
	unimplemented!();
	
	let (data, (olen, ilen, flag, mode)) = tuple((u8, u8, u8, u8))(data)?;
	
	let mode = PrimitiveMode::from(mode);
	let flag = PrimitiveFlags::from(flag);
	
	let primitive = Primitive {
		olen, ilen, flag, mode,
	};
	Ok((data, primitive))
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn decode_thing() {
		let thing: &[u8; 8060] = include_bytes!("../samples/01_FONT.TMD");
		parse_tmd(thing).expect("hello");
	}
}
