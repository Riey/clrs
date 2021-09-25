use super::{indices::*, PeCtx};
use bitflags::bitflags;
use clrs_derive::{sort_lines, ClrPread};
use scroll::{ctx::TryFromCtx, Pread};

macro_rules! make_table {
    ($($field:ident: $table:ty => $index:expr,)+) => {
        #[derive(Clone, Debug)]
        pub struct MetadataTable {
            $(
                $field: Vec<$table>,
            )+
        }

        impl<'a> TryFromCtx<'a, PeCtx> for MetadataTable {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                let offset = &mut 0;

                let mut table_present_bitvec: u64 = src.gread_with(offset, ctx)?;
                let _sorted_table_bitvec: u64 = src.gread_with(offset, ctx)?;

                eprintln!("valid_bitvec: 0x{:08X}", table_present_bitvec);

                $(
                    let mut $field = (Vec::new(), 0);
                )+

                $(
                    if table_present_bitvec & (1 << $index) != 0 {
                        $field.1 = src.gread_with::<u32>(offset, ctx)?;
                        table_present_bitvec &= (!(1 << $index));
                    }
                )+

                assert_eq!(table_present_bitvec, 0, "Unknown table bitvec presents {:X}", table_present_bitvec);

                $(
                    for _ in 0..$field.1 {
                        $field.0.push(src.gread_with(offset, ctx)?);
                    }

                    let $field = $field.0;
                )+

                Ok((Self {
                    $($field,)+
                }, *offset))
            }
        }
    };
}

sort_lines! {
    make_table

    assembly: Assembly => 0x20,
    assembly_os: AssemblyOS => 0x22,
    assembly_processor: AssemblyProcessor => 0x21,
    assembly_ref: AssemblyRef => 0x23,
    assembly_ref_os: AssemblyRefOS => 0x25,
    assembly_ref_processor: AssemblyRefProcessor => 0x24,
    class_layout: ClassLayout => 0x0F,
    constant: Constant => 0x0B,
    custom_attribute: CustomAttribute => 0x0C,
    decl_security: DeclSecurity => 0x0E,
    event_map: EventMap => 0x12,
    event: Event => 0x14,
    exported_type: ExportedType => 0x27,
    field: Field => 0x04,
    field_layout: FieldLayout => 0x10,
    field_marshal: FieldMarshal => 0x0D,
    field_rva: FieldRVA => 0x1D,
    file: File => 0x26,
    generic_param: GenericParam => 0x2A,
    generic_param_constraint: GenericParamConstraint => 0x2C,
    impl_map: ImplMap => 0x1C,
    interface_impl: InterfaceImpl => 0x09,
    manifest_resource: ManifestResource => 0x28,
    member_ref: MemberRef => 0x0A,
    method_def: MethodDef => 0x06,
    method_impl: MethodImpl => 0x19,
    method_semantics: MethodSemantics => 0x18,
    method_spec: MethodSpec => 0x2B,
    module: Module => 0x00,
    module_ref: ModuleRef => 0x1A,
    nested_class: NestedClass => 0x29,
    param: Param => 0x08,
    property: Property => 0x17,
    property_map: PropertyMap => 0x15,
    stand_along_sig: StandAloneSig => 0x11,
    type_def: TypeDef => 0x02,
    type_ref: TypeRef => 0x01,
    type_spec: TypeSpec => 0x1B,
}

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
        $(#[$outer:meta])*
        $vis:vis struct $bitflags:ident: $num_ty:ty {
            $(
                const $var:ident = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        bitflags! {
            $vis struct $bitflags: $num_ty {
                $(const $var = $value;)*
            }
        }

        impl<'a> TryFromCtx<'a, PeCtx> for $bitflags {
            type Error = scroll::Error;

            fn try_from_ctx(src: &'a [u8], ctx: PeCtx) -> Result<(Self, usize), Self::Error> {
                let n = src.pread_with(0, ctx)?;
                let flags = Self::from_bits_truncate(n);
                Ok((flags, std::mem::size_of::<$num_ty>()))
            }
        }

        bitflags_tryctx! {
            $($t)*
        }
    };
    () => {};
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

num_tryctx!(u8 u16 u32 u64);

bitflags_tryctx! {
    pub struct AssemblyFlags: u32 {
        const PUBLIC_KEY = 0x0001;
        const RETARGETABLE= 0x0100;
        const DISABLE_JIT_COMPILE_OPTIMIZER = 0x4000;
        const ENABLE_JIT_COMPILE_TRACKING = 0x8000;
    }

    pub struct EventAttributes: u16 {
        const SPECIAL_NAME = 0x0200;
        const RT_SPECIAL_NAME = 0x0400;
    }

    pub struct FieldAttributes: u16 {
        const FIELD_ACCESS_MASK = 0x0007;
        const COMPILER_CONTROLLED = 0x0000;
        // TODO
    }

    pub struct FileAttributes: u32 {
        // TODO
    }

    pub struct GenericParamAttributes: u16 {
        // TODO
    }

    pub struct PInvokeAttributes: u16 {
        // TODO
    }

    pub struct ManifestResourceAttributes: u32 {
        // TODO
    }

    pub struct MethodAttributes: u16 {
        // TODO
    }

    pub struct MethodImplAttributes: u16 {
        // TODO
    }

    pub struct MethodSemanticsAttributes: u16 {
        // TODO
    }

    pub struct ParamAttributes: u16 {
        // TODO
    }

    pub struct PropertyAttributes: u16 {
        // TODO
    }

    pub struct TypeAttributes: u32 {
        // TODO
    }
}

enum_tryctx! {
    #[derive(Clone, Copy, Debug)]
    pub enum AssemblyHashAlgorithm: u32 {
        None = 0x0000,
        /// Reserved
        MD5 = 0x8003,
        SHA1 = 0x8004,
        SHA256 = 0x800C,
        SHA384 = 0x800D,
        SHA512 = 0x800E,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum ElementType: u8 {
        /// Marks end of a list
        End = 0x00,
        Void = 0x01,
        Boolean = 0x02,
        Char = 0x03,
        I1 = 0x04,
        U1 = 0x05,
        I2 = 0x06,
        U2 = 0x07,
        I4 = 0x08,
        U4 = 0x09,
        I8 = 0x0a,
        U8 = 0x0b,
        R4 = 0x0c,
        R8 = 0x0d,
        String = 0x0e,
        /// Followed by *type*
        Ptr = 0x0f,
        /// Followed by *type*
        Byref = 0x10,
        /// Followed by TypeDef or TypeRef token
        ValueType = 0x11,
        /// Followed by TypeDef or TypeRef token
        Class = 0x12,
        /// Generic parameter in a generic type definition, represented as *number* (compressed unsigned integer)
        Var = 0x13,
        /// *type rank boundsCount bound1 ... loCount lo1 ...*
        Array = 0x14,
        /// Generic type instantiation.  Followed by *type type-arg-count  type-1 ... type-n*
        GenericInst = 0x15,
        TypedByref = 0x16,

        /// `System.IntPtr`
        I = 0x18,
        /// `System.UIntPtr`
        U = 0x19,
        /// Followed by full method signature
        FnPtr = 0x1b,
        /// `System.Object`
        Object = 0x1c,
        /// Single-dim array with 0 lower bound
        SzArray = 0x1d,
        /// Generic parameter in a generic type definition, represented as *number* (compressed unsigned integer)
        MVar = 0x1e,

        /// Required modifier: followed by a TypeDef or TypeRef token
        CmodReqd = 0x1f,
        /// Optinal modifier: followed by a TypeDef or TypeRef token
        CmodOpt = 0x20,

        /// Implemented within the CLI
        Internal = 0x21,
        /// Or'd with following element types
        Modifier = 0x40,
        /// Sentinel for vararg method signature
        Sentinel = 0x41,
        /// Denotes a local variable that points at a pinned object
        Pinned = 0x45,

        /// Indicates an argument of type `System.Type`
        Type = 0x50,
        /// Used in custom attributes to specify a boxed object
        Boxed = 0x51,
        Reserved = 0x52,
        /// Used in custom attributes to indicate a `FIELD`
        Field = 0x53,
        /// Used in custom attributes to indicate a `PROPERTY`
        Property = 0x54,
        /// Used in custom attributes to indicate a enum
        Enum = 0x55,
    }
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Assembly {
    pub hash_alg_id: AssemblyHashAlgorithm,
    pub version: AssemblyVersion,
    pub flags: AssemblyFlags,
    pub public_key: BlobIndex,
    pub name: StringIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyOS {
    pub platform_id: u32,
    pub major_version: u32,
    pub minor_version: u32,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyProcessor {
    pub processor: u32,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyVersion {
    pub major_version: u16,
    pub minor_version: u16,
    pub build_number: u16,
    pub revision_number: u16,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyRef {
    pub version: AssemblyVersion,
    pub flags: u32,
    pub public_key_or_token: BlobIndex,
    pub name: StringIndex,
    pub culture: StringIndex,
    pub hash_value: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyRefOS {
    pub platform_id: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub asm_ref: AssemblyRefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct AssemblyRefProcessor {
    pub processor: u32,
    pub asm_ref: AssemblyRefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct ClassLayout {
    pub packing_size: u16,
    pub class_size: u32,
    pub parent: TypeDefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Constant {
    pub const_ty: ElementType,
    pub parent: HasConstant,
    pub value: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct CustomAttribute {
    pub parent: HasCustomAttribute,
    pub ty: CustomAttributeType,
    pub value: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct DeclSecurity {
    pub action: u16,
    pub parent: u16,
    pub permission_set: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct EventMap {
    pub parent: TypeDefIndex,
    pub event_list: EventIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Event {
    pub flags: EventAttributes,
    pub name: StringIndex,
    pub ty: TypeDefOrRef,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct ExportedType {
    pub flags: TypeAttributes,
    pub def_id: u32,
    pub name: StringIndex,
    pub namespace: StringIndex,
    pub implementation: Implementation,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Field {
    pub flags: FieldAttributes,
    pub name: StringIndex,
    pub signature: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct FieldLayout {
    pub offset: u32,
    pub field: FieldIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct FieldMarshal {
    pub parent: HasFieldMarshal,
    pub native_type: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct FieldRVA {
    pub rva: u32,
    pub field: FieldIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct File {
    pub flags: FileAttributes,
    pub name: StringIndex,
    pub hash_value: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct GenericParam {
    pub number: u16,
    pub flags: GenericParamAttributes,
    pub owner: TypeOrMethodDef,
    pub name: StringIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct GenericParamConstraint {
    pub owner: GenericParamIndex,
    pub constraint: TypeDefOrRef,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct ImplMap {
    pub mapping_flags: u16,
    pub member_forwarded: MemberForwarded,
    pub import_name: StringIndex,
    pub import_scope: ModuleRefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct InterfaceImpl {
    pub class: TypeDefIndex,
    pub interface: TypeDefOrRef,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct ManifestResource {
    pub offset: u32,
    pub flags: ManifestResourceAttributes,
    pub name: StringIndex,
    pub implementation: Implementation,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct MemberRef {
    pub class: MemberRefParent,
    pub name: StringIndex,
    pub signature: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct MethodDef {
    pub rva: u32,
    pub impl_flags: MethodImplAttributes,
    pub flags: MethodAttributes,
    pub name: StringIndex,
    pub signature: BlobIndex,
    pub param_list: ParamIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct MethodImpl {
    pub class: TypeDefIndex,
    pub body: MethodDefOrRef,
    pub declaration: MethodDefOrRef,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct MethodSemantics {
    pub semantics: MethodSemanticsAttributes,
    pub method: MethodDefIndex,
    pub association: HasSemantics,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct MethodSpec {
    pub method: MethodDefOrRef,
    pub instantiation: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Module {
    pub generation: u16,
    pub name: StringIndex,
    pub mvid: GuidIndex,
    pub enc_id: GuidIndex,
    pub env_base_id: GuidIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct ModuleRef {
    pub name: StringIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct NestedClass {
    pub nested_class: TypeDefIndex,
    pub enclosing_class: TypeDefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Param {
    pub flags: ParamAttributes,
    pub sequence: u16,
    pub name: StringIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct Property {
    pub flags: PropertyAttributes,
    pub name: StringIndex,
    pub ty: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct PropertyMap {
    pub parent: TypeDefIndex,
    pub property_list: PropertyIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct StandAloneSig {
    pub signature: BlobIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct TypeDef {
    pub flags: TypeAttributes,
    pub type_name: StringIndex,
    pub type_namespace: StringIndex,
    pub extends: TypeDefOrRef,
    pub field_list: FieldIndex,
    pub method_list: MethodDefIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct TypeRef {
    pub resolution_scope: ResolutionScope,
    pub type_name: StringIndex,
    pub type_namespace: StringIndex,
}

#[repr(C)]
#[derive(Debug, ClrPread, Clone, Copy)]
pub struct TypeSpec {
    pub signature: BlobIndex,
}
