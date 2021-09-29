use std::collections::HashMap;

use scroll::Pread;
use wasm_encoder::{
    CodeSection, DataSection, Export, ExportSection, Function, FunctionSection,
    Instruction as WasmInst, MemorySection, MemoryType, Module, TypeSection, ValType,
};

use clrs_pe::cil::{Instruction, MethodBody};
use clrs_pe::pe::{
    Image, MetadataRoot, MethodDefIndex, MethodDefSig, Param, RetType, Type, UserStringIndex,
};

struct WasmContext {
    types: TypeSection,
    functions: FunctionSection,
    exports: ExportSection,
    codes: CodeSection,
    data: DataSection,
    memory: MemorySection,
    string_cache: HashMap<UserStringIndex, i32>,
    method_lookup: HashMap<MethodDefIndex, u32>,
}

const VAL_PTR: ValType = ValType::I32;

impl WasmContext {
    pub fn new(root: &MetadataRoot) -> Self {
        let mut memory = MemorySection::new();

        memory.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
        });

        let mut data = DataSection::new();
        let mut string_cache = HashMap::new();

        let mut offset = 0;

        for (index, s) in root.heap.list_user_string() {
            string_cache.insert(index, offset);
            data.active(0, WasmInst::I32Const(offset), s.iter().copied());
            offset += s.len() as i32 + 1;
        }

        WasmContext {
            types: TypeSection::new(),
            functions: FunctionSection::new(),
            exports: ExportSection::new(),
            data,
            memory,
            codes: CodeSection::new(),
            string_cache,
            method_lookup: HashMap::new(),
        }
    }

    pub fn finish(&self) -> Vec<u8> {
        let mut module = Module::new();
        module
            .section(&self.types)
            .section(&self.functions)
            .section(&self.exports)
            .section(&self.memory)
            .section(&self.data)
            .section(&self.codes);
        module.finish()
    }

    fn convert_wasm_param(&self, out: &mut Vec<ValType>, param: &Param) {
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

    fn convert_wasm_return(&self, ret: &RetType) -> Vec<ValType> {
        match ret {
            RetType::Void => vec![],
            _ => todo!(),
        }
    }

    fn convert_wasm_function(&self, body: &MethodBody) -> Function {
        dbg!(body);

        // TODO: arg, locals
        let locals = vec![];
        let mut f = Function::new(locals);

        for inst in body.instructions.iter() {
            match inst {
                Instruction::Nop => {
                    f.instruction(WasmInst::Nop);
                }
                Instruction::LdStr(s) => {
                    let s = s.as_userstring().unwrap();

                    // TODO: handling null
                    // if s.0 == 0 {
                    // }

                    f.instruction(WasmInst::I32Const(self.string_cache[&s]));
                    // TODO: alloc literal
                    // f.instruction(WasmInst::Call(0));
                }
                Instruction::Call(method) => {
                    if let Some(member) = method.as_member_ref() {
                    } else if let Some(method) = method.as_method_def() {
                        f.instruction(WasmInst::Call(self.method_lookup[&method]));
                    } else if let Some(spec) = method.as_method_spec() {
                    } else {
                        panic!("Invalid Call argument")
                    }
                }
                Instruction::Ret => {
                    f.instruction(WasmInst::Return);
                }
                Instruction::LdArg(n) => {
                    f.instruction(WasmInst::LocalGet(*n));
                }
                _ => todo!("{:?}", inst),
            }
        }

        f
    }

    pub fn emit_wasm_function_header(
        &mut self,
        name: &str,
        index: MethodDefIndex,
        signature: &MethodDefSig,
    ) {
        let mut params = Vec::new();
        signature
            .params
            .iter()
            .for_each(|p| self.convert_wasm_param(&mut params, p));
        let type_index = self.types.len();
        self.types
            .function(params, self.convert_wasm_return(&signature.ret));
        let func_index = self.functions.len();
        self.method_lookup.insert(index, func_index);
        self.functions.function(type_index);
        self.exports.export(name, Export::Function(type_index));
    }

    pub fn emit_wasm_function_body(&mut self, body: &MethodBody) {
        let func = self.convert_wasm_function(body);
        self.codes.function(&func);
    }
}

pub fn compile(image: &Image) -> Vec<u8> {
    let root = image.metadata_root();
    let mut ctx = WasmContext::new(root);
    let table = &root.metadata_stream.table;

    for (index, method) in table.list_method_def() {
        let name = method.name.resolve(root.heap);
        let signature = method.resolve_signature(root.heap);

        ctx.emit_wasm_function_header(name, index, &signature);
    }

    for (_, method) in table.list_method_def() {
        let body = method.resolve_body(image);
        ctx.emit_wasm_function_body(&body);
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
