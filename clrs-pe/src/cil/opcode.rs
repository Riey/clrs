use crate::pe::MetadataToken;
use scroll::{ctx::TryFromCtx, Endian, Pread};

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Nop,
    Break,
    LdArg(u32),
    LdArgA(u32),
    StLoc(u32),
    LdLoc(u32),
    LdLocA(u32),
    LdcI4(u32),
    LdcI8(u64),
    LdcR4(f32),
    LdcR8(f64),
    LdStr(MetadataToken),
    Dup,
    Pop,
    Jmp,
    Call(MetadataToken),
    CallI(MetadataToken),
    Ret,

    Neg,
    Not,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    ShrUn,

    Add,
    AddOvf,
    AddOvfUn,
    Sub,
    SubOvf,
    SubOvfUn,
    Div,
    DivUn,
    Rem,
    RemUn,
    Mul,
    MulOvf,
    MulOvfUn,

    Br,
    BrTrue,
    BrFalse,
    Ble,
    BleUn,
    Blt,
    BltUn,
    Bge,
    BgeUn,
    Bgt,
    BgtUn,
    Beq,
    BeqUn,
    Bne,
    BneUn,
}

impl<'a> TryFromCtx<'a, Endian> for Instruction {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let opcode: u8 = src.gread_with(offset, ctx)?;

        let inst = match opcode {
            0x00 => Self::Nop,
            0x01 => Self::Break,
            0x02 => Self::LdArg(0),
            0x03 => Self::LdArg(1),
            0x04 => Self::LdArg(2),
            0x05 => Self::LdArg(3),

            0x28 => Self::Call(src.gread_with(offset, ctx)?),
            0x2A => Self::Ret,

            0x40 => Self::BneUn,
            0x41 => Self::BgeUn,

            0x72 => Self::LdStr(src.gread_with(offset, ctx)?),
            _ => todo!("code: {:X}", opcode),
        };

        Ok((inst, *offset))
    }
}
