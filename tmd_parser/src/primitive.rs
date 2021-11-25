use crate::nom_starter_pack::*;

#[derive(Debug)]
pub struct Primitive {
	/// Word length of a 2D drawing primitive.(???)
	/// This is mostly redundant, but is here for completion's sake.
	/// In the future, it may be used for sanity/validity checks.
	pub olen: u8,
	
	/// Length of the packet data section, in words.
	/// This is mostly redundant, but is here for completion's sake.
	/// In the future, it may be used for sanity/validity checks.
	pub ilen: u8,
	
	pub flag: PrimitiveFlags,
	pub mode: PrimitiveMode,
	pub data: PrimitiveData,
}
impl Primitive {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, (olen, ilen, flag, mode)) = tuple((u8, u8, u8, u8))(data)?;
		
		let mode = PrimitiveMode::from(mode);
		let flag = PrimitiveFlags::from(flag);
		
		fn le_u16_index(data: &[u8]) -> IResult<&[u8], usize> {
			map(le_u16, |i| i as usize)(data)
		}
		
		// line olen 3, ilen 2, flag 0x1, mode 64.
		let (data, p_data) = match mode {
			PrimitiveMode::Line { abe: _, iip } => {
				let mut indices = tuple((le_u16_index, le_u16_index));
				if iip { // 2 colors
					let (data, colors) = tuple((Color::parse, Color::parse))(data)?;
					let (data, indices) = indices(data)?;
					(data, PrimitiveData::LineGr { colors, indices })
				} else { // 1 color (twice)
					let (data, color) = Color::parse(data)?;
					let (data, indices) = indices(data)?;
					(data, PrimitiveData::Line { color, indices })
				}
			},
			_ => { dbg!((mode, flag)); unimplemented!() },
			// FIXME: primitives aren't completely done
		};
		
		Ok((data, Primitive { olen, ilen, flag, mode, data: p_data }))
	}
}

#[derive(Debug)]
pub struct PrimitiveFlags {
	/// Should the polygon should have gradation / have a gradient color?
	/// Valid only for non-textured polygons subject to light source calcs.
	/// 
	/// Named `grd` in `FILEFRMT.PDF`.
	pub gradient: bool,
	
	/// Should the polygon should be double-faced?
	/// Valid only for polygons.
	/// 
	/// Named `fce` in `FILEFRMT.PDF`.
	pub double_sided: bool,
	
	/// Should light source calculations happen?
	/// 
	/// Named `lgt` in `FILEFRMT.PDF`.
	pub lit: bool,
}
impl From<u8> for PrimitiveFlags {
	fn from(f: u8) -> Self {
		PrimitiveFlags {
			gradient: f & 4 > 0,
			double_sided: f & 2 > 0,
			lit: f & 1 > 0,
		}
	}
}

#[derive(Debug)]
pub enum PrimitiveMode {
	// TODO: give these the same renaming treatment as the ones above?
	//       maybe if they're not too technical?
	Polygon {
		iip: bool,
		more: bool,
		tme: bool,
		abe: bool,
		tge: bool,
	},
	Line {
		iip: bool,
		abe: bool,
	},
	Sprite {
		siz: PrimitiveSpriteSize,
		abe: bool,
	},
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
	Line { color: Color, indices: (usize, usize), },
	LineGr { colors: (Color, Color), indices: (usize, usize), },
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
	pub r: u8, pub g: u8, pub b: u8,
}
impl Color {
	pub fn parse(data: &[u8]) -> IResult<&[u8], Self> {
		let (data, (r, g, b, _)) = tuple((u8, u8, u8, u8))(data)?;
		Ok((data, Color { r, g, b, }))
	}
}
