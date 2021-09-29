use scroll::Pread;
use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

use goblin::pe::PE;
use goblin::pe::{data_directories::DataDirectory, utils::get_data};

use clrs_pe::cil::MethodBody;
use clrs_pe::pe::{CliHeader, Image, MetadataRoot, MethodDefSig, TableIndex};

struct WasmContext {
    module: Module,
    types: TypeSection,
}

impl WasmContext {
    pub fn new() -> Self {
        WasmContext {
            module: Module::new(),
            types: TypeSection::new(),
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.module.section(&self.types);
        self.module.finish()
    }

    pub fn wasm_function(&mut self, name: &str, signature: &MethodDefSig) {}
}

pub fn compile(image: &Image) -> Vec<u8> {
    let mut ctx = WasmContext::new();
    let root = image.metadata_root();
    let table = &root.metadata_stream.table;

    for (_index, method) in table.list_method_def() {
        let name = method.name.resolve(root.heap);
        let signature: MethodDefSig = method
            .signature
            .resolve(root.heap)
            .pread_with(0, scroll::LE)
            .unwrap();

        ctx.wasm_function(name, &signature);
    }

    ctx.finish()
}

pub fn dump(image: &Image) {
    let root = image.metadata_root();
    let table = &root.metadata_stream.table;

    for (_index, method) in table.list_method_def() {
        let name = method.name.resolve(root.heap);
        let signature: MethodDefSig = method
            .signature
            .resolve(root.heap)
            .pread_with(0, scroll::LE)
            .unwrap();

        println!("{}: {:?}", name, signature);
    }
}
