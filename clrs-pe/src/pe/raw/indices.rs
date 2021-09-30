use crate::pe::Heap;

use super::tables::*;
use super::PeCtx;
use scroll::{ctx::TryFromCtx, Pread};

// TODO: 32bit index
macro_rules! make_single_index {
    ($($name:ident,)+) => {
        $(
            #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
            pub struct $name(pub u32);

            impl From<u32> for $name {
                fn from(n: u32) -> Self {
                    Self(n)
                }
            }

            impl<'a> TryFromCtx<'a, PeCtx> for $name {
                type Error = scroll::Error;

                fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                    let n: u16 = src.pread_with(0, ctx)?;
                    Ok((Self(n as u32), 2))
                }
            }
        )+
    };
}

const fn get_tag_mask(tag_size: u16) -> u16 {
    (1 << tag_size) - 1
}

macro_rules! make_coded_index {
    (($name:ident, $tag_size:expr, [$($ty:ident,)+]), $($t:tt)*) => {
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        pub enum $name {
            $(
                $ty($ty),
            )+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        Self::$ty($ty(index)) => {
                            write!(f, "{}({}, {})", stringify!($name), stringify!($ty), index)
                        }
                    )+
                }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        Self::$ty($ty(index)) => {
                            write!(f, "{}({}, {})", stringify!($name), stringify!($ty), index)
                        }
                    )+
                }
            }
        }

        impl<'a> TryFromCtx<'a, PeCtx> for $name {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                let n: u16 = src.pread_with(0, ctx)?;

                const TAG_MASK: u16 = get_tag_mask($tag_size);

                let mut tag = n & TAG_MASK;
                let real = n >> $tag_size;

                $(
                    #[allow(unused_assignments)]
                    if let Some(new_tag) = tag.checked_sub(1) {
                        tag = new_tag;
                    } else {
                        return Ok((Self::$ty($ty(real as u32)), 2));
                    }
                )+

                Err(scroll::Error::BadInput { msg: "Invalid tag", size: 2 })
            }
        }

        make_coded_index!($($t)*);
    };
    () => {};
}

make_single_index!(
    NotUsed1Index,
    NotUsed2Index,
    NotUsed3Index,
    StringIndex,
    UserStringIndex,
    BlobIndex,
    GuidIndex,
);

impl StringIndex {
    pub fn resolve<'a>(self, heap: Heap<'a>) -> &'a str {
        heap.ref_string(self.0 as usize).unwrap()
    }
}

impl UserStringIndex {
    pub fn resolve<'a>(self, heap: Heap<'a>) -> &'a [u8] {
        heap.ref_user_string(self.0 as usize).unwrap()
    }
}

impl BlobIndex {
    pub fn resolve<'a>(self, heap: Heap<'a>) -> &'a [u8] {
        heap.ref_blob(self.0 as usize).unwrap()
    }
}

make_coded_index! {
    (TypeDefOrRef, 2, [
        TypeDefIndex,
        TypeRefIndex,
        TypeSpecIndex,
    ]),
    (HasConstant, 2, [
        FieldIndex,
        ParamIndex,
        PropertyIndex,
    ]),
    (HasCustomAttribute, 5, [
        MethodDefIndex,
        FieldIndex,
        TypeRefIndex,
        TypeDefIndex,
        ParamIndex,
        InterfaceImplIndex,
        MemberRefIndex,
        ModuleIndex,
        // Permission
        BlobIndex,
        PropertyIndex,
        EventIndex,
        StandAloneSigIndex,
        ModuleRefIndex,
        TypeSpecIndex,
        AssemblyIndex,
        AssemblyRefIndex,
        FileIndex,
        ExportedTypeIndex,
        ManifestResourceIndex,
        GenericParamIndex,
        GenericParamConstraintIndex,
        MethodSpecIndex,
    ]),
    (HasFieldMarshal, 1, [
        FieldIndex,
        ParamIndex,
    ]),
    (HasDeclSecurity, 2, [
        TypeDefIndex,
        MethodDefIndex,
        AssemblyIndex,
    ]),
    (MemberRefParent, 3, [
        TypeDefIndex,
        TypeRefIndex,
        ModuleRefIndex,
        MethodDefIndex,
        TypeSpecIndex,
    ]),
    (HasSemantics, 1, [
        EventIndex,
        PropertyIndex,
    ]),
    (MethodDefOrRef, 1, [
        MethodDefIndex,
        MemberRefIndex,
    ]),
    (MemberForwarded, 1, [
        FieldIndex,
        MethodDefIndex,
    ]),
    (Implementation, 2, [
        FileIndex,
        AssemblyRefIndex,
        ExportedTypeIndex,
    ]),
    (CustomAttributeType, 3, [
        NotUsed1Index,
        NotUsed2Index,
        MethodDefIndex,
        MemberRefIndex,
        NotUsed3Index,
    ]),
    (ResolutionScope, 2, [
        ModuleIndex,
        ModuleRefIndex,
        AssemblyRefIndex,
        TypeRefIndex,
    ]),
    (TypeOrMethodDef, 1, [
        TypeDefIndex,
        MethodDefIndex,
    ]),
}
