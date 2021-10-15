use crate::{
    cil::MethodBody,
    pe::{Heap, Image},
};

use super::{indices::*, FieldSig, MethodDefSig, PeCtx};
use clrs_derive::{make_table, ClrPread};
use scroll::{ctx::TryFromCtx, Pread};

make_table! {
    {
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

    {
        Document: u32 => 0x30,
        MethodDebugInformation: u32 => 0x31,
        LocalScope: u32 => 0x32,
        LocalVariable: u32 => 0x33,
        LocalConstant: u32 => 0x34,
        ImportScope: u32 => 0x35,
        StateMachineMethod: u32 => 0x36,
        CustomDebugInformation: u32 => 0x37,
        UserString: UserStringIndex => 0x70,
    }
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

    // II.23.1.5
    pub struct FieldAttributes: u16 {
        const FIELD_ACCESS_MASK = 0x0007;
        const COMPILER_CONTROLLED = 0x0000;
        const PRIVATE = 0x0001;
        const FAM_AND_ASSEM = 0x0002;
        const ASSEMBLY = 0x0003;
        const FAMILY = 0x0004;
        const FAM_OR_ASSEM = 0x0005;
        const PUBLIC = 0x0006;

        const STATIC = 0x0010;
        const INIT_ONLY = 0x0020;
        const LITERAL = 0x0040;
        const NOT_SERIALIZED = 0x0080;
        const SPECIAL_NAME = 0x0200;

        const PINVOKE_IMPL = 0x2000;

        const RT_SPECIAL_NAME = 0x0400;
        const HAS_FIELD_MARSHAL = 0x1000;
        const HAS_DEFAULT = 0x8000;
        const HAS_FIELD_RVA = 0x0100;
    }

    pub struct FileAttributes: u32 {
        const CONTAINS_METADATA = 0x0000;
        const CONTAINS_NO_METADATA = 0x0001;
    }

    pub struct GenericParamAttributes: u16 {
        const VARIANCE_MASK = 0x0003;
        const NONE = 0x0000;
        const COVARIANT = 0x0001;
        const CONTRAVARIANT = 0x0002;
        const SPECIAL_CONSTRAINT_MASK = 0x001C;
        const REFERENCE_TYPE_CONSTRAINT = 0x0004;
        const NOT_NULLABLE_VALUE_TYPE_CONSTRAINT = 0x0008;
        const DEFAULT_CONSTRUCTOR_CONSTRAINT = 0x0010;
    }

    pub struct PInvokeAttributes: u16 {
        const NO_MANGLE = 0x0001;
        const CHARSET_MASK = 0x0006;
        const CHARSET_NOT_SPEC = 0x0000;
        const CHARSET_ANSI = 0x0002;
        const CHARSET_UNICODE = 0x0004;
        const CHARSET_AUTO = 0x0006;

        const SUPPORTS_LAST_ERROR = 0x0040;

        const CALL_CONV_MASK = 0x0700;
        const CALL_CONV_PLATFORMAPI = 0x0100;
        const CALL_CONV_CDECL = 0x0200;
        const CALL_CONV_STDCALL = 0x0300;
        const CALL_CONV_THISCALL = 0x0400;
        const CALL_CONV_FASTCALL = 0x0500;
    }

    pub struct ManifestResourceAttributes: u32 {
        const VISIBILITY_MASK = 0x0007;
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
    }

    pub struct MethodAttributes: u16 {
        const MEMBER_ACCESS_MASK = 0x0007;
        const COMPILER_CONTROLLED = 0x0000;
        const PRIVATE = 0x0001;
        const FAM_AND_ASSEM = 0x0002;
        const ASSEMBLY = 0x0003;
        const FAMILY = 0x0004;
        const FAM_OR_ASSEM = 0x0005;
        const PUBLIC = 0x0006;

        const STATIC = 0x0010;
        const FINAL = 0x0020;
        const VIRTUAL = 0x0040;
        const HIDE_BY_SIG = 0x0080;
        const VTABLE_LAYOUT_MASK = 0x0100;

        const REUSE_SLOT = 0x0000;
        const NEW_SLOT = 0x0100;
        const STRICT = 0x0200;
        const ABSTRACT = 0x0400;
        const SPECIAL_NAME = 0x0800;

        const PINVOKE_IMPL = 0x2000;
        const UNMANAGED_EXPORT = 0x0008;
    }

    pub struct MethodImplAttributes: u16 {
        const CODE_TYPE_MASK = 0x0003;
        const IL = 0x0000;
        const NATIVE = 0x0001;
        const OPTIL = 0x0002;
        const RUNTIME = 0x0003;
        const MANAGED_MASK = 0x0004;
        const UNMANAGED = 0x0004;
        const MANAGED = 0x0000;

        const FORWARD_REF = 0x0010;
        const PRESERVE_SIG = 0x0080;
        const INTERNAL_CALL = 0x1000;
        const SYNCHRONIZED = 0x0020;
        const NO_INLINING = 0x0008;
        const MAX_METHOD_IMPL_VAL = 0xFFFF;
        const NO_OPTIMIZATION = 0x0040;
    }

    pub struct MethodSemanticsAttributes: u16 {
        const SETTER = 0x0001;
        const GETTER = 0x0002;
        const OTHER = 0x0004;
        const ADD_ON = 0x0008;
        const REMOVE_ON = 0x0010;
        const FIRE = 0x0020;
    }

    pub struct ParamAttributes: u16 {
        const IN = 0x0001;
        const OUT = 0x0002;
        const HAS_DEFAULT = 0x1000;
        const HAS_FIELD_MARSHAL = 0x2000;
        const UNUSED = 0xCFE0;
    }

    pub struct PropertyAttributes: u16 {
        const SPECIAL_NAME = 0x0200;
        const RT_SPECIAL_NAME = 0x0400;
        const HAS_DEFAULT = 0x1000;
        const UNUSED = 0xE9FF;
    }

    pub struct TypeAttributes: u32 {
        const VISIBILITY_MASK = 0x0000_0007;
        const NOT_PUBLIC = 0x0000_0000;
        const PUBLIC = 0x0000_0001;
        const NESTED_PUBLIC = 0x0000_0002;
        const NESTED_PRIVATE = 0x0000_0003;
        const NESTED_FAMILY = 0x0000_0004;
        const NESTED_ASSEMBLY = 0x0000_0005;
        const NESTED_FAM_AND_ASSEM = 0x0000_0006;
        const NESTED_FAM_OR_ASSEM = 0x0000_0007;

        const LAYOUT_MASK = 0x0000_0018;
        const AUTO_LAYOUT = 0x0000_0000;
        const SEQUENTIAL_LAYOUT = 0x0000_0008;
        const EXPLICIT_LAYOUT = 0x0000_0010;

        const CLASS_SEMANTIC_MASK = 0x0000_0020;
        const CLASS = 0x0000_0000;
        const INTERFACE = 0x0000_0020;

        const ABSTRACT = 0x0000_0080;
        const SEALED = 0x0000_0100;
        const SPECIAL_NAME = 0x0000_0400;

        const IMPORT = 0x0000_1000;
        const SERIALIZED = 0x0000_2000;

        const STRING_FORMAT_MASK = 0x0003_0000;
        const ANSI_CLASS = 0x0000_0000;
        const UNICODE_CLASS = 0x0001_0000;
        const AUTO_CLASS = 0x0002_0000;
        const CUSTOM_FORMAT_CLASS = 0x0003_0000;
        const CUSTOM_STRING_FORMAT_MASK = 0x00C0_0000;

        const BEFORE_FIELD_INIT = 0x0010_0000;
        const RT_SPECIAL_NAME = 0x0000_0800;
        const HAS_SECURITY = 0x0004_0000;
        const IS_TYPE_FORWARDER = 0x0020_0000;
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
#[derive(Debug, ClrPread, Clone, Copy, PartialEq, Eq, Hash)]
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

macro_rules! define_resolve_signature {
    ($ty:ty, $fn_name:ident, $ret_ty:ty, $def_field:ident) => {
        impl $ty {
            pub fn $fn_name(self, heap: Heap) -> $ret_ty {
                self.$def_field
                    .resolve(heap)
                    .unwrap()
                    .pread_with(0, scroll::LE)
                    .expect("Parse Signature")
            }
        }
    };
}

define_resolve_signature!(MemberRef, resolve_signature, MethodDefSig, signature);

define_resolve_signature!(MethodDef, resolve_signature, MethodDefSig, signature);

define_resolve_signature!(Field, resolve_signature, FieldSig, signature);

impl MethodDef {
    pub fn resolve_body(self, image: &Image) -> MethodBody {
        image.get_data(self.rva).expect("Parse MethodBody")
    }
}

macro_rules! define_resolve {
    ($ty:ty, $fn_name:ident, $ret_ty:ty, $ret_index:ty, $def_field:ident, $table_field:ident) => {
        impl $ty {
            pub fn $fn_name(
                self,
                table: &MetadataTable,
            ) -> impl Iterator<Item = ($ret_index, &$ret_ty)> {
                let mut index = self
                    .resolve_table(table)
                    .map(|d| d.$def_field)
                    .unwrap_or_default();
                let end = Self(self.0 + 1)
                    .resolve_table(table)
                    .map(|d| d.$def_field)
                    .unwrap_or_default();

                std::iter::from_fn(move || {
                    if index == end {
                        None
                    } else {
                        let r = index.resolve_table(table).map(|d| (index, d));
                        index.0 += 1;
                        r
                    }
                })
            }
        }
    };
}

define_resolve!(
    MethodDefIndex,
    resolve_params,
    Param,
    ParamIndex,
    param_list,
    param
);
define_resolve!(
    TypeDefIndex,
    resolve_fields,
    Field,
    FieldIndex,
    field_list,
    field
);
define_resolve!(
    TypeDefIndex,
    resolve_methods,
    MethodDef,
    MethodDefIndex,
    method_list,
    method_def
);
