use scroll::Pread;
use scroll::{ctx::TryFromCtx, Endian};

mod opcode;

pub use self::opcode::Instruction;

#[derive(Debug)]
pub struct MethodBody {
    pub instructions: Vec<Instruction>,
}

impl<'a> TryFromCtx<'a, Endian> for MethodBody {
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
