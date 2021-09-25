use goblin::pe::PE;
use goblin::pe::{data_directories::DataDirectory, utils::get_data};

use clrs::cil::MethodBody;
use clrs::pe::{raw::*, CliHeader, MetadataRoot};

fn main() {
    let file = include_bytes!("../assets/HelloWorld.dll");
    let file = &file[..];
    let pe = PE::parse(file).unwrap();
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

    let cli_header_value: CliHeader = get_data(file, sections, cli_header, file_alignment).unwrap();
    let metadata_root: MetadataRoot =
        get_data(file, sections, cli_header_value.metadata, file_alignment).unwrap();

    assert_eq!(
        metadata_root.signature, 0x424a5342,
        "Invalid CLR metadata signature"
    );
    assert_eq!(metadata_root.major_version, 1);
    assert_eq!(metadata_root.minor_version, 1);

    let heap = metadata_root.heap;
    let metadata_stream = metadata_root.metadata_stream.unwrap();
    let metadata_table = metadata_stream.table;
    let entry_point = cli_header_value
        .entry_point_token
        .as_method_def()
        .expect("EntryPoint is not Method")
        .resolve_table(&metadata_table)
        .expect("Entry method not found");

    dbg!(entry_point);

    let body: MethodBody = get_data(
        file,
        sections,
        DataDirectory {
            virtual_address: entry_point.rva,
            size: 0,
        },
        file_alignment,
    )
    .unwrap();

    println!("EntryPoint({}): {:?}", entry_point.name.resolve(heap), body);
}
