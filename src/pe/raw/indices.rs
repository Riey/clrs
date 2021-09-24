use super::PeCtx;
use scroll::{ctx::TryFromCtx, Pread};

// TODO: 32bit index
macro_rules! make_single_index {
    ($(($name:ident, $target:ident),)+) => {
        $(
            #[derive(Debug, Clone, Copy)]
            pub struct $name(pub u16);

            impl<'a> TryFromCtx<'a, PeCtx> for $name {
                type Error = scroll::Error;

                fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                    let n: u16 = src.pread_with(0, ctx)?;
                    Ok((Self(n), 2))
                }
            }
        )+
    };
}

macro_rules! make_tag_index {
    (($name:ident, $tag_size:expr, [$($ty:ident,)+]), $($t:tt)*) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(pub u16);

        impl<'a> TryFromCtx<'a, PeCtx> for $name {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                let n: u16 = src.pread_with(0, ctx)?;
                Ok((Self(n), 2))
            }
        }

        make_tag_index!($($t)*);
    };
    () => {};
}

make_tag_index! {
    (TypeDefOrRef, 2, [
        TypeDef,
        TypeRef,
        TypeSpec,
    ]),
    (HasConstant, 2, [
        Field,
        Param,
        Property,
    ]),
    (HasCustomAttribute, 5, [
        MethodDef,
        Field,
        TypeRef,
        TypeDef,
        Param,
        InterfaceImpl,
        MemberRef,
        Module,
        Permission,
        Property,
        Event,
        StandAloneSig,
        ModuleRef,
        TypeSpec,
        Assembly,
        AssemblyRef,
        File,
        ExportedType,
        ManifestResource,
        GenericParam,
        GenericParamConstraint,
        MethodSpec,
    ]),
    (HasFieldMarshal, 1, [
        Field,
        Param,
    ]),
    (HasDeclSecurity, 2, [
        TypeDef,
        MethodDef,
        Assembly,
    ]),
    (MemberRefParent, 3, [
        TypeDef,
        TypeRef,
        ModuleRef,
        MethodDef,
        TypeSpec,
    ]),
    (HasSemantics, 1, [
        Event,
        Property,
    ]),
    (MethodDefOrRef, 1, [
        MethodDef,
        MethodRef,
    ]),
    (MemberForwarded, 1, [
        Field,
        MethodDef,
    ]),
    (Implementation, 2, [
        File,
        AssemblyRef,
        ExportedType,
    ]),
    (CustomAttributeType, 3, [
        NotUsed,
        NotUsed,
        MethodDef,
        MemberRef,
        NotUsed,
    ]),
    (ResolutionScope, 2, [
        Module,
        ModuleRef,
        AssemblyRef,
        TypeRef,
    ]),
    (TypeOrMethodDef, 1, [
        TypeDef,
        MethodDef,
    ]),
}

make_single_index! {
    (BlobIndex, Blob),
    (StringIndex, String),
    (GuidIndex, Guid),
    (AssemblyRefIndex, AssemblyRef),
    (TypeDefIndex, TypeDef),
    (TypeRefIndex, TypeRef),
    (TypeSpecIndex, TypeSpec),
    (ExportedTypeIndex, ExportedType),
    (FieldIndex, Field),
    (ParamIndex, Param),
    (PropertyIndex, Property),
    (MethodDefIndex, MethodDef),
    (MethodRefIndex, MethodRef),
    (InterfaceImplIndex, InterfaceImpl),
    (MemberRefIndex, MemberRef),
    (ModuleIndex, Module),
    (ModuleRefIndex, ModuleRef),
    (PermissionIndex, Permission),
    (EventIndex, Event),
    (StandAloneSigIndex, StandAloneSig),
    (FileIndex, File),
    (ManifestResourceIndex, ManifestResource),
    (GenericParamIndex, GenericParam),
    (GenericParamConstraintIndex, GenericParamConstraint),
    (MethodSpecIndex, MethodSpec),
}
