use crate::pe::{ElementType, TypeDefIndex, TypeRefIndex, TypeSpecIndex};
use scroll::{ctx::TryFromCtx, Endian, Pread};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct U(pub u32);

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

#[derive(Clone, Debug)]
pub enum MethodCallingConvension {
    Default,
    VarArg,
    Generic(u32),
}

#[derive(Clone, Debug)]
pub struct MethodDefSig {
    pub has_this: bool,
    pub explict_this: bool,
    pub calling_convension: MethodCallingConvension,
    pub ret: RetType,
    pub params: Vec<Param>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum RetType {
    Type { byref: bool, ty: Type },
    Void,
    TypedByref,
}

#[derive(Clone, Debug)]
pub enum Param {
    Type { byref: bool, ty: Type },
    TypedByref,
}

#[derive(Clone, Debug)]
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
    // TODO
}
