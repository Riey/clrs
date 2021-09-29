use std::convert::TryInto;

use goblin::container::Endian;
use goblin::pe::data_directories::DataDirectory;
use goblin::pe::utils::get_data;
use goblin::pe::PE;
use scroll::ctx::{StrCtx, TryFromCtx};
use scroll::{Pread, LE};

mod raw;

pub use self::raw::*;

pub struct Image<'a> {
    bytes: &'a [u8],
    cli_header: CliHeader,
    metadata_root: MetadataRoot<'a>,
}

impl<'a> Image<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> goblin::error::Result<Self> {
        let pe = PE::parse(bytes).unwrap();
        if pe.header.coff_header.machine != 0x14c {
            panic!("Is not a .Net executable");
        }
        let optional_header = pe.header.optional_header.expect("No optional header");
        let file_alignment = optional_header.windows_fields.file_alignment;
        let cli_header = optional_header
            .data_directories
            .get_clr_runtime_header()
            .expect("No CLI header");
        let sections = &pe.sections;

        let cli_header_value: CliHeader =
            get_data(bytes, sections, cli_header, file_alignment).unwrap();
        let metadata_root: MetadataRoot =
            get_data(bytes, sections, cli_header_value.metadata, file_alignment).unwrap();
        Ok(Self {
            bytes,
            cli_header: cli_header_value,
            metadata_root,
        })
    }

    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub fn cli_header(&self) -> &CliHeader {
        &self.cli_header
    }
    pub fn metadata_root(&self) -> &MetadataRoot<'a> {
        &self.metadata_root
    }
}

#[repr(C)]
#[derive(Debug, Pread)]
pub struct CliHeader {
    pub cb: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub metadata: DataDirectory,
    pub flags: u32,
    pub entry_point_token: MetadataToken,
    pub resources: DataDirectory,
    pub strong_name_signature_hash: DataDirectory,
    pub code_manager_table: u64,
    pub vtable_fixups: DataDirectory,
    pub export_address_table_jumps: u64,
    pub managed_native_header: u64,
}

#[derive(Debug)]
pub struct MetadataRoot<'a> {
    pub signature: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub version: &'a str,
    pub heap: Heap<'a>,
    pub metadata_stream: MetadataStream,
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

        let mut heap = Heap::default();
        let mut metadata_stream = None;

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
                    metadata_stream = Some(stream_src.pread(0)?);
                }
                "#Strings" => {
                    heap.strings =
                        std::str::from_utf8(stream_src).expect("#Strings is invalid UTF-8");
                }
                "#Blob" => {
                    heap.blob = stream_src;
                }
                "#GUID" => {
                    heap.guid = stream_src;
                }
                "#US" => {}
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
                metadata_stream: metadata_stream.unwrap(),
                heap,
                version,
            },
            *offset,
        ))
    }
}

/// #~
#[repr(C)]
#[derive(Debug)]
pub struct MetadataStream {
    pub major_version: u8,
    pub minor_version: u8,
    pub table: MetadataTable,
    pub ctx: PeCtx,
}

impl<'a> TryFromCtx<'a> for MetadataStream {
    type Error = scroll::Error;

    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let ctx = raw::PeCtx {};

        let reserved: u32 = src.gread_with(offset, LE)?;
        debug_assert_eq!(reserved, 0);

        let major_version = src.gread_with(offset, LE)?;
        let minor_version = src.gread_with(offset, LE)?;

        let heap_size: u8 = src.gread_with(offset, LE)?;
        assert_eq!(heap_size, 0);

        let reserved: u8 = src.gread_with(offset, LE)?;
        debug_assert_eq!(reserved, 1);

        let table = src.gread_with(offset, ctx)?;

        Ok((
            Self {
                major_version,
                minor_version,
                table,
                ctx,
            },
            *offset,
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Heap<'a> {
    strings: &'a str,
    blob: &'a [u8],
    guid: &'a [u8],
}

const GUID_SIZE: usize = 128 / 8;

impl<'a> Heap<'a> {
    pub fn ref_string(self, index: usize) -> Option<&'a str> {
        if index == 0 {
            return None;
        }

        let ret = self.strings.get(index..)?;
        ret.split('\0').next()
    }

    pub fn ref_blob(self, mut index: usize) -> Option<&'a [u8]> {
        if index == 0 {
            return None;
        }

        let length: U = self.blob.gread_with(&mut index, scroll::LE).ok()?;

        self.blob.get(index..index + length.0 as usize)
    }

    pub fn ref_guid(self, index: usize) -> Option<&'a [u8; GUID_SIZE]> {
        if index == 0 {
            return None;
        }

        self.guid.get(index..index + GUID_SIZE)?.try_into().ok()
    }
}
