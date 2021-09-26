use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

use clrs_pe::cil::{Instruction as CilInstruction, MethodBody};
use clrs_pe::pe::{MetadataRoot, MethodDef};

pub fn compile_cil_method(root: &MetadataRoot, method: &MethodDef) -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();

    module.finish()
}
