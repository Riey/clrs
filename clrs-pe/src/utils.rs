macro_rules! num_tryctx {
    ($($num:ty)+) => {
        $(
            impl<'a> TryFromCtx<'a, PeCtx> for $num {
                type Error = scroll::Error;

                fn try_from_ctx(src: &'a [u8], _: PeCtx) -> Result<(Self, usize), Self::Error> {
                    let offset = &mut 0;
                    // PE file always little endian number
                    let n = src.gread_with(offset, scroll::LE)?;
                    Ok((n, *offset))
                }
            }
        )+
    };
}

macro_rules! bitflags_tryctx {
    (
        $(
            $(#[$outer:meta])*
            $vis:vis struct $bitflags:ident: $num_ty:ty {
                $(
                    const $var:ident = $value:expr;
                )*
            }
        )+
    ) => {
        $(
            ::bitflags::bitflags! {
                $(#[$outer])*
                $vis struct $bitflags: $num_ty {
                    $(const $var = $value;)*
                }
            }

            impl<'a, C: Copy> TryFromCtx<'a, C> for $bitflags where $num_ty: TryFromCtx<'a, C, Error = ::scroll::Error> {
                type Error = ::scroll::Error;

                fn try_from_ctx(src: &'a [u8], ctx: C) -> Result<(Self, usize), Self::Error> {
                    let n = src.pread_with(0, ctx)?;
                    let flags = Self::from_bits_truncate(n);
                    Ok((flags, std::mem::size_of::<$num_ty>()))
                }
            }
        )+
    };
}

macro_rules! enum_tryctx {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident: $inner:ident {
            $(
                $(#[$var_meta:meta])*
                $variant:ident = $value:expr,
            )+
        }
        $($t:tt)*
    ) => {
        $(#[$outer])*
        #[repr($inner)]
        $vis enum $name {
            $(
                $(#[$var_meta])*
                $variant = $value,
            )+
        }

        impl $name {
            pub fn from_n(n: $inner) -> Option<Self> {
                match n {
                    $(
                        $value => Some(Self::$variant),
                    )+
                    _ => None,
                }
            }
        }

        impl<'a, C: Copy> TryFromCtx<'a, C> for $name where $inner: TryFromCtx<'a, C, Error = scroll::Error> {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: C) -> Result<(Self, usize), Self::Error> {
                let n: $inner = src.pread_with(0, ctx)?;
                const SIZE: usize = std::mem::size_of::<$inner>();

                let e = match n {
                    $(
                        $value => Self::$variant,
                    )+
                    _ => return Err(scroll::Error::Custom(format!("Enum {} Get {}", stringify!($name), n))),
                    // _ => return Err(scroll::Error::BadInput { size: SIZE, msg: "Invalid enum variant" }),
                };

                Ok((e, SIZE))
            }
        }

        enum_tryctx! {
            $($t)*
        }

    };
    () => {};
}
