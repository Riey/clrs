use super::{indices::*, PeCtx};
use bitflags::bitflags;
use clrs_derive::{make_table, ClrPread};
use scroll::{ctx::TryFromCtx, Pread};

make_table! {
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

    #[derive(Clone, Copy, Debug)]
    pub enum Opcode: u8 {
        Nop = 0x00,
        Break = 0x01,
        LdArg0 = 0x02,
        LdArg1 = 0x03,
        LdArg2 = 0x04,
        LdArg3 = 0x05,
        LdLoc0 = 0x06,
        LdLoc1 = 0x07,
        LdLoc2 = 0x08,
        LdLoc3 = 0x09,
        StLoc0 = 0x0A,
        StLoc1 = 0x0B,
        StLoc2 = 0x0C,
        StLoc3 = 0x0D,
        LdArgS = 0x0E,
        LdArgaS = 0x0F,
        StArgS = 0x10,
        LdLocS = 0x11,
        LdLocaS = 0x12,
        StLocS = 0x13,
        LdNull = 0x14,
        LdcI4M1 = 0x15,
        LdcI40 = 0x16,
        LdcI41 = 0x17,
        LdcI42 = 0x18,
        LdcI43 = 0x19,
        LdcI44 = 0x1A,
        LdcI45 = 0x1B,
        LdcI46 = 0x1C,
        LdcI47 = 0x1D,
        LdcI48 = 0x1E,
        LdcI4S = 0x1F,
        LdcI4 = 0x20,
        LdcI8 = 0x21,
        LdcR4 = 0x22,
        LdcR8 = 0x23,
        Dup = 0x25,
        Pop = 0x26,
        Jmp = 0x27,
        Call = 0x28,
        CallI = 0x29,
        Ret = 0x2A,
        BrS = 0x2B,
        BrFalseS = 0x2C,
        BrTrueS = 0x2D,
        BeqS = 0x2E,
        BgeS = 0x2F,
        BgtS = 0x30,
        BleS = 0x31,
        BltS = 0x32,
        BneUnS = 0x33,
        BgeUnS = 0x34,
        BgtUnS = 0x35,
        BleUnS = 0x36,
        BltUnS = 0x37,
        Br = 0x38,
        BrFalse = 0x39,
        BrTrue = 0x3A,
        Beq = 0x3B,
        Bge = 0x3C,
        Bgt = 0x3D,
        Ble = 0x3E,
        Blt = 0x3F,
        BneUn = 0x40,
        BgeUn = 0x41,
        BgtUn = 0x42,
        BleUn = 0x43,
        BltUn = 0x44,
        Switch = 0x45,
        LdIndI1 = 0x46,
        LdIndU1 = 0x47,
        LdIndI2 = 0x48,
        LdIndU2 = 0x49,
        LdIndI4 = 0x4A,
        LdIndU4 = 0x4B,
        LdIndI8 = 0x4C,
        LdIndU8 = 0x4D,
        LdIndR4 = 0x4E,
        LdIndR8 = 0x4F,
        LdIndRef = 0x50,
        StIntRef = 0x51,
        StIndI1 = 0x52,
        StIndI2 = 0x53,
        StIndI4 = 0x54,
        StIndI8 = 0x55,
        StIndR4 = 0x56,
        StIndR8 = 0x57,

        Add = 0x58,
        Sub = 0x59,
        Mul = 0x5A,
        Div = 0x5B,
        DivUn = 0x5C,
        Rem = 0x5D,
        RemUn = 0x5E,
        And = 0x5F,
        Or = 0x60,
        Xor = 0x61,
        Shl = 0x62,
        Shr = 0x63,
        ShrUn = 0x64,
        Neg = 0x65,
        Not = 0x66,

        ConvI1 = 0x67,
        ConvI2 = 0x68,
        ConvI4 = 0x69,
        ConvI8 = 0x6A,
        ConvR4 = 0x6B,
        ConvR8 = 0x6C,
        ConvU4 = 0x6D,
        ConvU8 = 0x6E,

        CallVirt = 0x6F,
        CpObj = 0x70,
        LdObj = 0x71,

        LdStr = 0x72,
        NewObj = 0x73,
        CastClass = 0x74,
        IsInst = 0x75,
        ConvRUn = 0x76,
        Unbox = 0x79,
        Throw = 0x7A,
        LdFld = 0x7B,
        LdFldA = 0x7C,
        StFld = 0x7D,
        LdSFld = 0x7E,
        LdSFldA = 0x7F,
        StSFld = 0x80,
        StObj = 0x81,

        ConvOvfI1Un = 0x82,
        ConvOvfI2Un = 0x83,
        ConvOvfI4Un = 0x84,
        ConvOvfI8Un = 0x85,
        ConvOvfU1Un = 0x86,
        ConvOvfU2Un = 0x87,
        ConvOvfU4Un = 0x88,
        ConvOvfU8Un = 0x89,
        ConvOvfIUn = 0x8A,
        ConvOvfUUn = 0x8B,

        Box = 0x8C,
        NewArr = 0x8D,
        LdLen = 0x8E,
        LdElemA = 0x8F,
        LdElemI1 = 0x90,
        LdElemU1 = 0x91,
        LdElemI2 = 0x92,
        LdElemU2 = 0x93,
        LdElemI4 = 0x94,
        LdElemU4 = 0x95,
        LdElemI8 = 0x96,
        LdElemI = 0x97,
        LdElemR4 = 0x98,
        LdElemR8 = 0x99,
        LdElemRef = 0x9A,

        StElemI = 0x9B,
        StElemI1 = 0x9C,
        StElemI2 = 0x9D,
        StElemI4 = 0x9E,
        StElemI8 = 0x9F,
        StElemR4 = 0xA0,
        StElemR8 = 0xA1,
        StElemRef = 0xA2,

        LdElem = 0xA3,
        StElem = 0xA4,

        UnboxAny = 0xA5,
        ConvOvfI1 = 0xB3,
        ConvOvfU1 = 0xB4,
        ConvOvfI2 = 0xB5,
        ConvOvfU2 = 0xB6,
        ConvOvfI4 = 0xB7,
        ConvOvfU4 = 0xB8,
        ConvOvfI8 = 0xB9,
        ConvOvfU8 = 0xBA,

        RefAnyVal = 0xC2,
        CkFinite = 0xC3,
        MkRefAny = 0xC6,
        LdToken = 0xD0,
        ConvU2 = 0xD1,
        ConvU1 = 0xD2,
        ConvI = 0xD3,
        ConvOvfI = 0xD4,
        ConvOvfU = 0xD5,

        AddOvf = 0xD6,
        AddOvfUn = 0xD7,
        MulOvf = 0xD8,
        MulOvfUn = 0xD9,
        SubOvf = 0xDA,
        SubOvfUn = 0xDB,

        EndFinally = 0xDC,
        Leave = 0xDD,
        LeaveS = 0xDE,
        StIntI = 0xDF,
        ConvU = 0xE0,
        Extend = 0xFE,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum ExtendOpcode: u8 {
        ArgList = 0x00,
        Ceq = 0x01,
        Cgt = 0x02,
        CgtUn = 0x03,
        Clt = 0x04,
        CltUn = 0x05,
        LdFtn = 0x06,
        LdVirtFtn = 0x07,

        LdArg = 0x09,
        LdArgA = 0x0A,
        StArg = 0x0B,
        LdLoc = 0x0C,
        LdLocA = 0x0D,
        StLoc = 0x0E,
        LocalLoc = 0x0F,

        EndFilter = 0x11,
        Unaligned = 0x12,
        Volatile = 0x13,

        Tail = 0x14,
        InitObj = 0x15,
        Constrained = 0x16,
        CpBlk = 0x17,
        InitBlk = 0x18,
        No = 0x19,
        Rethrow = 0x1A,
        SizeOf = 0x1C,
        RefAnyType = 0x1D,
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
