use crate::pe::{ElementType, TypeDefIndex, TypeRefIndex, TypeSpecIndex};
use scroll::{ctx::TryFromCtx, Endian, Pread};

/// Compressed UInt32
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(transparent)]
pub struct U(pub u32);

impl U {
    pub fn byte_size(self) -> usize {
        match self.0 {
            0x00..=0x7F => 1,
            0x80..=0x3FFF => 2,
            0x4000..=0x3FFF_FFFF => 4,
            _ => unreachable!(),
        }
    }
}

impl<'a> TryFromCtx<'a, Endian> for U {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let first: u8 = src.pread_with(0, ctx)?;

        if let 0x00..=0x7F = first {
            return Ok((U(first as _), 1));
        }

        let second: u16 = src.pread_with(0, ctx)?;

        if let 0x8080..=0xBFFF = second {
            return Ok((U((second & !0x8000) as _), 2));
        }

        let last: u32 = src.pread_with(0, ctx)?;

        Ok((U((last & !0xC000_0000) as _), 4))
    }
}

#[test]
fn decode_num() -> Result<(), scroll::Error> {
    assert_eq!(
        0x03u8.to_ne_bytes().pread_with::<U>(0, Endian::Little)?,
        U(0x03)
    );
    assert_eq!(
        0x8080u16.to_ne_bytes().pread_with::<U>(0, Endian::Little)?,
        U(0x80)
    );
    assert_eq!(
        0xDFFF_FFFFu32
            .to_ne_bytes()
            .pread_with::<U>(0, Endian::Little)?,
        U(0x1FFF_FFFF)
    );

    Ok(())
}

bitflags_tryctx! {
    pub struct MethodCallingConvension: u8 {
        const DEFAULT = 0x0;
        const VAR_ARG = 0x5;
        const GENERIC = 0x10;
        const HAS_THIS = 0x20;
        const EXPLICT_THIS = 0x40;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MethodDefSig {
    pub calling_convension: MethodCallingConvension,
    pub ret: RetType,
    pub params: Vec<Param>,
}

impl<'a> TryFromCtx<'a, Endian> for MethodDefSig {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let calling_convension = src.gread_with(offset, ctx)?;
        let param_count: U = src.gread_with(offset, ctx)?;
        let ret = src.gread_with(offset, ctx)?;
        let params = std::iter::repeat_with(|| src.gread_with(offset, ctx))
            .take(param_count.0 as usize)
            .collect::<Result<_, _>>()?;

        Ok((
            Self {
                calling_convension,
                ret,
                params,
            },
            *offset,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeDefOrRefOrSpecEncoded {
    TypeDef(TypeDefIndex),
    TypeRef(TypeRefIndex),
    TypeSpec(TypeSpecIndex),
}

impl<'a> TryFromCtx<'a, Endian> for TypeDefOrRefOrSpecEncoded {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let raw: u32 = src.pread_with(0, ctx)?;

        let tag = raw & 0xFF00_0000;
        let row = raw & 0x00FF_FFFF;

        let s = match tag {
            0x00 => Self::TypeDef(row.into()),
            0x01 => Self::TypeRef(row.into()),
            0x02 => Self::TypeSpec(row.into()),
            _ => {
                return Err(scroll::Error::BadInput {
                    size: 4,
                    msg: "Invalid TypeDefOrRefOrSpecEncoded tag",
                })
            }
        };

        Ok((s, 4))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CustomMod {
    Opt(TypeDefOrRefOrSpecEncoded),
    Reqd(TypeDefOrRefOrSpecEncoded),
}

impl<'a> TryFromCtx<'a, Endian> for CustomMod {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let ty = ElementType::from_n(src.gread_with::<U>(offset, ctx)?.0 as _);

        let s = match ty {
            Some(ElementType::CmodOpt) => Self::Opt(src.gread_with(offset, ctx)?),
            Some(ElementType::CmodReqd) => Self::Reqd(src.gread_with(offset, ctx)?),
            _ => {
                return Err(scroll::Error::BadInput {
                    size: 1,
                    msg: "Invalid ElementType for CustomMod",
                })
            }
        };

        Ok((s, *offset))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RetType {
    Type { byref: bool, ty: Type },
    Void,
    TypedByref,
}

impl<'a> TryFromCtx<'a, Endian> for RetType {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let ty: ElementType = src.gread_with(offset, ctx)?;

        let s = match ty {
            ElementType::Void => Self::Void,
            ElementType::TypedByref => Self::TypedByref,
            ElementType::Byref => Self::Type {
                byref: true,
                ty: src.gread_with(offset, ctx)?,
            },
            _ => {
                // Reset offset
                *offset = 0;
                Self::Type {
                    byref: false,
                    ty: src.gread_with(offset, ctx)?,
                }
            }
        };

        Ok((s, *offset))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Param {
    Type { byref: bool, ty: Type },
    TypedByref,
}

impl<'a> TryFromCtx<'a, Endian> for Param {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let ty: ElementType = src.gread_with(offset, ctx)?;

        let s = match ty {
            ElementType::TypedByref => Self::TypedByref,
            ElementType::Byref => Self::Type {
                byref: true,
                ty: src.gread_with(offset, ctx)?,
            },
            _ => {
                // Reset offset
                *offset = 0;
                Self::Type {
                    byref: false,
                    ty: src.gread_with(offset, ctx)?,
                }
            }
        };

        Ok((s, *offset))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Boolean,
    Char,
    I1,
    U1,
    I2,
    U2,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    I,
    U,
    Object,
    String,

    SzArray {
        element_ty: Box<Type>,
        mods: Vec<CustomMod>,
    },

    Var {
        count: U,
    },
    // TODO
}

impl<'a> TryFromCtx<'a, Endian> for Type {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let s = match src.gread_with(offset, ctx)? {
            ElementType::Boolean => Self::Boolean,
            ElementType::Char => Self::Char,
            ElementType::I1 => Self::I1,
            ElementType::U1 => Self::U1,
            ElementType::I2 => Self::I2,
            ElementType::U2 => Self::U2,
            ElementType::I4 => Self::I4,
            ElementType::U4 => Self::U4,
            ElementType::I8 => Self::I8,
            ElementType::U8 => Self::U8,
            ElementType::R4 => Self::R4,
            ElementType::R8 => Self::R8,
            ElementType::I => Self::I,
            ElementType::U => Self::U,
            ElementType::Object => Self::Object,
            ElementType::String => Self::String,
            ElementType::SzArray => Self::SzArray {
                // TODO
                mods: vec![],
                element_ty: Box::new(src.gread_with(offset, ctx)?),
            },
            ElementType::Var => Self::Var {
                count: src.gread_with(offset, ctx)?,
            },
            _ => todo!(),
        };

        Ok((s, *offset))
    }
}

#[test]
fn signature_main() {
    let sig: MethodDefSig = [
        0,  // default
        1,  // one param
        1,  // void return
        29, // array of
        14, // string
    ]
    .pread_with(0, scroll::LE)
    .unwrap();

    assert_eq!(
        sig,
        MethodDefSig {
            ret: RetType::Void,
            params: vec![Param::Type {
                byref: false,
                ty: Type::SzArray {
                    element_ty: Box::new(Type::String),
                    mods: vec![],
                },
            },],
            calling_convension: MethodCallingConvension::DEFAULT,
        }
    );
}
