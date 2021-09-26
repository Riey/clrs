use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

use goblin::pe::PE;
use goblin::pe::{data_directories::DataDirectory, utils::get_data};

use clrs_pe::cil::MethodBody;
use clrs_pe::pe::{CliHeader, MetadataRoot, TableIndex};

pub fn compile(root: &MetadataRoot, file: &[u8]) -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();

    let table = &root.metadata_stream.table;

    let mut methods = table.method_def.iter().enumerate().peekable();

    while let Some((method_index, method)) = methods.next() {
        let next_param_index = methods.peek().map(|(_, m)| m.param_list.0 as usize - 1).unwrap_or(table.param.len());
        let params = &table.param[method.param_list.0 as usize - 1..next_param_index];
    }

    types.function([], []);

    module.finish()
}
