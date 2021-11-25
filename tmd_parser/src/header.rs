use crate::nom_starter_pack::*;

#[derive(Debug)]
pub struct Header {
	pub id: u32,
	pub flags: HeaderFlags,
	pub n_obj: usize,
}
impl Header {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, (id, flags, n_obj)) = tuple((le_u32, le_u32, le_u32))(data)?;
		let flags = HeaderFlags::from(flags);
		let n_obj = n_obj as usize;
		Ok((data, Header { id, flags, n_obj }))
	}
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
