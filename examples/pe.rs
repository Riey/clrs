use goblin::pe::utils::get_data;
use goblin::pe::PE;

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

    println!("{:b}", cli_header_value.entry_point_token);

    let ty = (cli_header_value.entry_point_token & 0xFF000000) >> 24;
    let row = cli_header_value.entry_point_token & 0x00FFFFFF;

    assert_eq!(ty, 6, "EntryPoint is not Method");

    let heap = metadata_root.heap;
    let metadata_table = metadata_root.metadata_stream.unwrap().table;
    let method_def_index = MethodDefIndex(row as _);
    let entry_point = method_def_index.resolve_table(&metadata_table).unwrap();

    dbg!(entry_point);

    let name = entry_point.name.resolve(heap);

    dbg!(name);
}
