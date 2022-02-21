use crate::nom_starter_pack::*;
use crate::primitive::Primitive;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Vector {
	pub x: i16, pub y: i16, pub z: i16,
}
type Vertex = Vector; // this doesn't do much, but it's the thought that counts
impl Vertex {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, (x, y, z)) = tuple((le_i16, le_i16, le_i16))(data)?;
		Ok((data, Vector { x, y, z }))
	}
	pub fn parse_pad(data: &[u8]) -> IResult<&[u8], Self> {
		map(tuple((Vertex::parse, le_i16)), |(v, _)| v)(data)
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
	pub fn parse_pad(data: &[u8]) -> IResult<&[u8], Self> {
		into(Vertex::parse_pad)(data)
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
			
			let (_, vertices) = count(Vertex::parse_pad, vertex_len)(&all_data[vertex_ptr..])?;
			let (_, normals) = count(Normal::parse_pad, normal_len)(&all_data[normal_ptr..])?;
			let (_, primitives) = count(Primitive::parse, primitive_len)(&all_data[primitive_ptr..])?;
			
			Ok((data, Object { vertices, normals, primitives, scale, }))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn dont_take_too_much() {
		const D: &[u8] = &[64, 0, 128, 0, 255, 0, 69, 96, 255, 0, 128, 0, 64, 0, 8, 0];
		
		let data = D;
		let (data, vector) = Vertex::parse_pad(data)
			.expect("Should only take 8 bytes");
		assert_eq!(data.len(), 8);
		assert_eq!(vector, Vector { x: 64, y: 128, z: 255 });
		
		let (data, vector) = Vertex::parse(data)
			.expect("Should only take 6 bytes");
		assert!(!data.is_empty());
		assert_eq!(vector, Vector { x: 255, y: 128, z: 64 });
	}
}
