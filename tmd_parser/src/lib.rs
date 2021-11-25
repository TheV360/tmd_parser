mod nom_starter_pack {
	pub use nom::{
		IResult,
		combinator::{into, map},
		multi::count,
		number::complete::{le_i16, le_i32, le_u16, le_u32, u8},
		sequence::tuple
	};
}
use crate::nom_starter_pack::*;

/// The [`Header`] and its astounding 1 (one) flag.
pub mod header;
use header::Header;

/// [`Object`]s and their associated [`Vertex`]ies and [`Normal`]s.
pub mod object;
use object::Object;

pub mod primitive;

#[derive(Debug)]
pub struct Tmd {
	pub header: Header,
	pub obj_table: Vec<Object>,
}
impl Tmd {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, header) = Header::parse(data)?;
		
		let entire = {
			if header.flags.fix_p {
				unimplemented!() // i'm lazy.
				// The only way I can really think of that makes this flag
				// work is if you read the following bytes completely, stuffed
				// all the vertices, normals, and primitives into a Vec, and
				// sorted them -- and then sorted the objects too -- and then
				// use the object's reference to find the verticess and uhh..
				// You know!! This kinda makes sense, right?
			} else { data }
		};
		
		let (data, obj_table) = count(Object::parse_with(entire), header.n_obj)(data)?;
		
		let tmd = Tmd { header, obj_table, };
		Ok((data, tmd))
	}
}

#[cfg(test)]
mod tests {
	// use super::*;
	// TODO: tests need to be written
}
