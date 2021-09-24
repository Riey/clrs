#[macro_export]
macro_rules! bitflags_tryctx {
    ($bitflags:ty, $num_ty:ty) => {
        impl<'a> TryFromCtx<'a, PeCtx<'a>> for $bitflags {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx<'a>) -> Result<(Self, usize), Self::Error> {
                let n: $num_ty = src.pread_with(0, ctx.endian)?;
                let flags = Self::from_bits_truncate(n as u32);
                Ok((flags, std::mem::size_of::<$num_ty>()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
