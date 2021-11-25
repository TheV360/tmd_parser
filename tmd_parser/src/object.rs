use crate::nom_starter_pack::*;
use crate::primitive::Primitive;

#[derive(Debug)]
pub struct Vector {
	pub x: i16, pub y: i16, pub z: i16,
}
type Vertex = Vector; // this doesn't do much, but it's the thought that counts
impl Vertex {
	/// This is used to parse a Vertex.
	/// There may be a need in the future for a "takes only 3 ints" variant.
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, (x, y, z, _)) = tuple((le_i16, le_i16, le_i16, le_i16))(data)?;
		Ok((data, Vector { x, y, z }))
	}
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
impl Normal {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		into(Vertex::parse)(data)
	}
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

/*
// This doesn't have a use anymore, since I can just
// directly create an object via `Object::parse_with`.
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
impl ObjectEntry {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
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
}
*/

#[derive(Debug)]
pub struct Object {
	pub vertices: Vec<Vector>,
	pub normals: Vec<Normal>,
	pub primitives: Vec<Primitive>,
	
	/// This maaaaayyybeee dictates the scaling of the vertices after everything
	/// is loaded? It's encoded as `2**scale`, so that means
	/// `-2 = 1/4`, `-1 = 1/2`, `0 = 1`, `1 = 2`, and `2 = 4`.
	/// **It's also completely unused by everyone.**
	pub scale: i32,
}
impl Object {
	/// This takes a slice of all data past the header and returns a function that,
	/// when called, will maybe return an `Object` with all its vertices, normals,
	/// and primitives. It can, however, also return an error.
	pub fn parse_with<'a>(all_data: &'a [u8]) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Self> {
		move |data: &'a [u8]| {
			fn le_u32_index(data: &[u8]) -> IResult<&[u8], usize> {
				map(le_u32, |i| i as usize)(data)
			}
			
			let (data, (vertex_ptr, vertex_len)) = tuple((le_u32_index, le_u32_index))(data)?;
			let (data, (normal_ptr, normal_len)) = tuple((le_u32_index, le_u32_index))(data)?;
			let (data, (primitive_ptr, primitive_len)) = tuple((le_u32_index, le_u32_index))(data)?;
			
			let (data, scale) = le_i32(data)?;
			
			let (_, vertices) = count(Vertex::parse, vertex_len)(&all_data[vertex_ptr..])?;
			let (_, normals) = count(Normal::parse, normal_len)(&all_data[normal_ptr..])?;
			let (_, primitives) = count(Primitive::parse, primitive_len)(&all_data[primitive_ptr..])?;
			
			Ok((data, Object { vertices, normals, primitives, scale, }))
		}
	}
}
