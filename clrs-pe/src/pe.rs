use goblin::container::Endian;
use goblin::pe::data_directories::DataDirectory;
use scroll::ctx::{StrCtx, TryFromCtx};
use scroll::{Pread, LE};

mod raw;

pub use self::raw::*;

#[repr(C)]
#[derive(Debug, Pread)]
pub struct CliHeader {
    pub cb: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub metadata: DataDirectory,
    pub flags: u32,
    pub entry_point_token: MetadataToken,
    _empty: u64,
    pub strong_name_signature_hash: u32,
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
    pub strings: &'a str,
    pub blob: &'a [u8],
    pub guid: &'a [u8],
}
