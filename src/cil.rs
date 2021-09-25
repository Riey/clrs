use scroll::Pread;
use scroll::{ctx::TryFromCtx, Endian};

use crate::pe::raw::{ExtendOpcode, Opcode, PeCtx};

#[derive(Debug)]
pub struct MethodBody<'a> {
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> TryFromCtx<'a, Endian> for MethodBody<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let header: u8 = src.gread_with(offset, ctx)?;

        let mut instructions = Vec::new();

        let format = header & 0b11;
        let value = header >> 2;

        match format {
            // Thin
            0b10 => {
                let target = value as usize + 1;
                while *offset < target {
                    instructions.push(src.gread_with(offset, ctx)?);
                }
            }
            _ => todo!(),
        }

        Ok((Self { instructions }, *offset))
    }
}

#[derive(Debug)]
pub struct Instruction<'a> {
    pub opcode: Opcode,
    pub extend_opcode: Option<ExtendOpcode>,
    pub operand: &'a [u8],
}

impl Opcode {
    pub fn operand_size(self) -> usize {
        use Opcode::*;

        match self {
            LdStr => 4,
            Call => 4,
            CpObj => 4,

            // alias
            StLoc0 | StLoc1 | StLoc2 | StLoc3 | LdLoc0 | LdLoc1 | LdLoc2 | LdLoc3 => 0,
            Nop | Ret | Break | Extend => 0,
            _ => todo!("{:?}", self),
        }
    }
}

impl ExtendOpcode {
    pub fn operand_size(self) -> usize {
        use ExtendOpcode::*;

        match self {
            SizeOf => 4,
            _ => todo!(),
        }
    }
}

impl<'a> TryFromCtx<'a, Endian> for Instruction<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let opcode: Opcode = src.gread_with(offset, ctx)?;
        let operand = &src[*offset..];

        if let Opcode::Extend = opcode {
            let extend_opcode: ExtendOpcode = src.gread_with(offset, ctx)?;
            Ok((
                Self {
                    opcode,
                    extend_opcode: Some(extend_opcode),
                    operand: &operand[..extend_opcode.operand_size()],
                },
                *offset,
            ))
        } else {
            Ok((
                Self {
                    opcode,
                    extend_opcode: None,
                    operand: &operand[..opcode.operand_size()],
                },
                *offset,
            ))
        }
    }
}
