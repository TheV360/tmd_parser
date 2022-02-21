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
		
		let (data, p_data) = match mode {
			// line olen 3, ilen 2, flag 0x1, mode 64.
			PrimitiveMode::Line { translucent: _, gradient } => {
				if gradient { // 2 colors
					let (data, colors) = tuple((Color::parse_pad, Color::parse_pad))(data)?;
					let (data, indices) = tuple((le_u16_index, le_u16_index))(data)?;
					(data, PrimitiveData::LineGr { colors, indices })
				} else { // 1 color
					let (data, color) = Color::parse_pad(data)?;
					let (data, indices) = tuple((le_u16_index, le_u16_index))(data)?;
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
			gradient:     f & 4 > 0,
			double_sided: f & 2 > 0,
			lit:          f & 1 > 0,
		}
	}
}

#[derive(Debug)]
pub enum PrimitiveMode {
	Polygon {
		/// What type of shading will this use?
		/// `false` = flat, `true` = Gouraud
		/// 
		/// Named `iip` in `FILEFRMT.PDF`.
		smooth_shading: bool,
		
		/// Is there a 4th vertex?
		/// 
		/// Not given a name in `FILEFRMT.PDF`.
		more: bool,
		
		/// Will this use a texture?
		/// 
		/// Named `tme` in `FILEFRMT.PDF`.
		textured: bool,
		
		/// Will this be translucent?
		/// (TODO: does this mean "will this have an alpha channel?")
		/// 
		/// Named `abe` in `FILEFRMT.PDF`.
		translucent: bool,
		
		/// Will the texture be lit? (???)
		/// 
		/// Named `tge` in `FILEFRMT.PDF`.
		lit: bool,
	},
	Line {
		/// Will this use two colors and
		/// interpolate between the two?
		/// 
		/// Named `iip` in `FILEFRMT.PDF`.
		gradient: bool,
		
		/// Will this be translucent?
		/// (TODO: does this mean "will this have an alpha channel?")
		/// 
		/// Named `abe` in `FILEFRMT.PDF`.
		translucent: bool,
	},
	Sprite {
		/// What size will this sprite be?
		/// 
		/// Named `siz` in `FILEFRMT.PDF`.
		size: PrimitiveSpriteSize,
		
		/// Will this be translucent?
		/// (TODO: does this mean "will this have an alpha channel?")
		/// 
		/// Named `abe` in `FILEFRMT.PDF`.
		translucent: bool,
	},
}
impl From<u8> for PrimitiveMode {
	fn from(f: u8) -> Self {
		use PrimitiveMode::*;
		match f >> 5 {
			1 => Polygon {
				smooth_shading: f & 16 > 0,
				more:           f &  8 > 0,
				textured:       f &  4 > 0,
				translucent:    f &  2 > 0,
				lit:            f &  1 > 0,
			},
			2 => Line {
				gradient:    f & 16 > 0,
				translucent: f &  2 > 0,
			},
			3 => Sprite {
				size: PrimitiveSpriteSize::from(f),
				translucent: f & 2 > 0,
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
		map(tuple((u8, u8, u8)), |(r, g, b)| Color { r, g, b, })(data)
	}
	pub fn parse_pad(data: &[u8]) -> IResult<&[u8], Self> {
		map(tuple((Color::parse, u8)), |(c, _)| c)(data)
	}
}
