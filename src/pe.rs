use enum_map::EnumMap;
use enumset::EnumSet;
use goblin::container::Endian;
use goblin::pe::data_directories::DataDirectory;
use scroll::ctx::{StrCtx, TryFromCtx};
use scroll::{Pread, LE};

mod raw;

#[repr(C)]
#[derive(Debug, Pread)]
pub struct CliHeader {
    pub cb: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub metadata: DataDirectory,
    pub flags: u32,
    pub entry_point_token: u32,
    _empty: u64,
    pub strong_name_signature_hash: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MetadataRoot<'a> {
    pub signature: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub version: &'a str,
    pub metadata_stream: Option<MetadataStream>,
}

impl<'a> TryFromCtx<'a, Endian> for MetadataRoot<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let signature = src.gread_with(offset, LE)?;
        let major_version = src.gread_with(offset, LE)?;
        let minor_version = src.gread_with(offset, LE)?;
        let _reserved: u32 = src.gread_with(offset, LE)?;
        let length: u32 = src.gread_with(offset, LE)?;
        let version: &str = src.gread_with(offset, StrCtx::Length(length as usize))?;
        let version = version.trim_end_matches('\0');
        let _reserved: u16 = src.gread_with(offset, LE)?;
        let num_streams: u16 = src.gread_with(offset, LE)?;

        let mut metadata_stream = None;
        let mut heap = Heap::default();

        for _ in 0..num_streams {
            let stream_offset: u32 = src.gread_with(offset, LE)?;
            let size: u32 = src.gread_with(offset, LE)?;
            let name = src.gread(offset)?;

            let pad = 4 - *offset % 4;

            if pad < 4 {
                *offset += pad;
            }

            let stream_src = &src[stream_offset as usize..(stream_offset + size) as usize];

            match name {
                "#~" => {
                    metadata_stream = Some(src.gread(offset)?);
                }
                "#Strings" => {
                    heap.strings = stream_src;
                }
                "#Blob" => {
                    heap.blob = stream_src;
                    dbg!(String::from_utf8_lossy(heap.blob));
                }
                other => {
                    eprintln!("Unknown stream header: {}", other);
                }
            }
        }

        Ok((
            Self {
                signature,
                major_version,
                minor_version,
                metadata_stream,
                version,
            },
            *offset,
        ))
    }
}

#[derive(Debug, Default)]
pub struct Heap<'a> {
    strings: &'a [u8],
    blob: &'a [u8],
}

impl<'a> Heap<'a> {
    pub fn read_string(&self, index: u16) -> Result<&'a str, scroll::Error> {
        self.strings.pread(index as usize)
    }

    pub fn read_blob(&self, _index: u16) -> Result<&'a [u8], scroll::Error> {
        Ok(b"")
        // TODO
        // self.blob.pread(index as usize)
    }
}

/// ECMA-335 II.22
#[repr(u64)]
#[derive(enumset::EnumSetType, enum_map::Enum, Debug)]
pub enum MetadataStreamItem {
    Assembly = 0x20,
    AssemblyOS = 0x22,
    AssemblyProcessor = 0x21,
    AssemblyRef = 0x23,
    AssemblyRefOS = 0x25,
    AssemblyRefProcessor = 0x24,

    ClassLayout = 0x0F,
    Constant = 0x0B,
    CustomAttribute = 0x0C,

    DeclSecurity = 0x0E,
    EventMap = 0x12,
    Event = 0x14,
    ExportedType = 0x27,

    Field = 0x04,
    FieldLayout = 0x10,
    FieldMarshal = 0x0D,
    FieldRVA = 0x1D,

    File = 0x26,

    GenericParam = 0x2A,
    GenericParamConstraint = 0x2C,

    ImplMap = 0x1C,

    InterfaceImpl = 0x09,

    ManifestResource = 0x28,

    MemberRef = 0x0A,

    MethodDef = 0x06,
    MethodImpl = 0x19,
    MethodSemantics = 0x18,
    MethodSpec = 0x2B,

    Module = 0x00,
    ModuleRef = 0x1A,

    NestedClass = 0x29,

    Param = 0x08,

    Property = 0x17,
    PropertyMap = 0x15,

    StandAloneSig = 0x11,

    TypeDef = 0x02,
    TypeRef = 0x01,
    TypeSpec = 0x1B,
}

impl MetadataStreamItem {
    pub fn column_size(self) -> usize {
        match self {
            Self::Assembly => 22,
            Self::AssemblyOS => 12,
            Self::AssemblyProcessor => 4,
            Self::AssemblyRef => 20,
            Self::AssemblyRefOS => 14,
            Self::AssemblyRefProcessor => 6,
            Self::ClassLayout => 8,
            Self::Module => 10,
            Self::MethodDef => 14,
            other => todo!("{:?}", other),
        }
    }
}

/// #~
#[repr(C)]
#[derive(Debug)]
pub struct MetadataStream {
    pub major_version: u8,
    pub minor_version: u8,
    pub table: raw::MetadataTable,
}

impl<'a> TryFromCtx<'a> for MetadataStream {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let ctx = raw::PeCtx {};

        let _reserved: u32 = src.gread_with(offset, LE)?;
        let major_version = src.gread_with(offset, LE)?;
        let minor_version = src.gread_with(offset, LE)?;

        let _heap_size: u8 = src.gread_with(offset, LE)?;
        let _reserved: u8 = src.gread_with(offset, LE)?;

        let table = src.gread_with(offset, ctx)?;

        Ok((
            Self {
                major_version,
                minor_version,
                table,
            },
            *offset,
        ))
    }
}
