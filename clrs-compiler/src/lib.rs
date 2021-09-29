use scroll::Pread;
use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

use goblin::pe::PE;
use goblin::pe::{data_directories::DataDirectory, utils::get_data};

use clrs_pe::cil::MethodBody;
use clrs_pe::pe::{CliHeader, Image, MetadataRoot, MethodDefSig, Param, RetType, TableIndex, Type};

struct WasmContext {
    module: Module,
    types: TypeSection,
    functions: FunctionSection,
    exports: ExportSection,
    codes: CodeSection,
}

const VAL_PTR: ValType = ValType::I32;

impl WasmContext {
    pub fn new() -> Self {
        WasmContext {
            module: Module::new(),
            types: TypeSection::new(),
            functions: FunctionSection::new(),
            exports: ExportSection::new(),
            codes: CodeSection::new(),
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.module
            .section(&self.types)
            .section(&self.functions)
            .section(&self.exports)
            .section(&self.codes);
        self.module.finish()
    }

    fn wasm_param(out: &mut Vec<ValType>, param: &Param) {
        match param {
            Param::Type { byref: true, .. } => {
                out.push(VAL_PTR);
            }
            Param::Type { byref: false, ty } => {
                match ty {
                    Type::I | Type::U => out.push(VAL_PTR),
                    Type::Boolean
                    | Type::Char
                    | Type::I1
                    | Type::I2
                    | Type::I4
                    | Type::U
                    | Type::U1
                    | Type::U2
                    | Type::U4 => out.push(ValType::I32),
                    Type::I8 | Type::U8 => out.push(ValType::I64),
                    Type::R4 => out.push(ValType::F32),
                    Type::R8 => out.push(ValType::F64),
                    Type::String => {
                        // 3 word PTR/LEN/CAP
                        out.push(VAL_PTR);
                        out.push(VAL_PTR);
                        out.push(VAL_PTR);
                    }
                    Type::SzArray { .. } => {
                        // 2 word PTR/LEN
                        out.push(VAL_PTR);
                        out.push(VAL_PTR);
                    }
                    _ => todo!(),
                }
            }
            Param::TypedByref => todo!(),
        }
    }

    fn wasm_return(ret: &RetType) -> Vec<ValType> {
        match ret {
            RetType::Void => vec![],
            _ => todo!(),
        }
    }

    pub fn wasm_function(&mut self, name: &str, signature: &MethodDefSig) {
        let mut params = Vec::new();
        signature
            .params
            .iter()
            .for_each(|p| Self::wasm_param(&mut params, p));
        let type_index = self.types.len();
        self.types
            .function(params, Self::wasm_return(&signature.ret));
        self.functions.function(type_index);
        self.exports.export(name, Export::Function(type_index));
    }
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
