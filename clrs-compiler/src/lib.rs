use std::collections::HashMap;
use std::rc::Rc;
use wasm_encoder::{
    CodeSection, DataSection, EntityType, Export, ExportSection, Function, FunctionSection,
    ImportSection, Instruction as WasmInst, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};

use clrs_pe::cil::{Instruction, MethodBody};
use clrs_pe::pe::{
    Heap, Image, MemberRef, MemberRefIndex, MemberRefParent, MetadataRoot, MetadataTable,
    MethodCallingConvension, MethodDefIndex, MethodDefSig, Param, RetType, TableIndex, Type,
    TypeDef, TypeDefIndex, UserStringIndex,
};

#[derive(Clone)]
struct MethodCacheData {
    pub fn_index: u32,
}

#[derive(Clone)]
struct SignatureCacheData {
    pub type_index: u32,
    pub param_types: Rc<Vec<ValType>>,
}

#[derive(Clone)]
struct StringCacheData {
    pub data_index: i32,
    pub str_len: u32,
}

struct MemberRefCacheData {
    pub fn_index: u32,
}

struct WasmContext {
    types: TypeSection,
    functions: FunctionSection,
    exports: ExportSection,
    imports: ImportSection,
    codes: CodeSection,
    data: DataSection,
    memory: MemorySection,
    signature_cache: HashMap<MethodDefSig, SignatureCacheData>,
    string_cache: HashMap<UserStringIndex, StringCacheData>,
    method_cache: HashMap<MethodDefIndex, MethodCacheData>,
    member_ref_cache: HashMap<MemberRefIndex, MemberRefCacheData>,
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
            string_cache.insert(
                index,
                StringCacheData {
                    data_index: offset,
                    str_len: s.len() as u32,
                },
            );
            data.active(0, WasmInst::I32Const(offset), s.iter().copied());
            offset += s.len() as i32;
        }

        WasmContext {
            types: TypeSection::new(),
            functions: FunctionSection::new(),
            exports: ExportSection::new(),
            imports: ImportSection::new(),
            data,
            memory,
            codes: CodeSection::new(),
            string_cache,
            signature_cache: HashMap::new(),
            method_cache: HashMap::new(),
            member_ref_cache: HashMap::new(),
        }
    }

    fn compute_fn_index(&self, is_import: bool) -> u32 {
        if is_import {
            self.imports.len()
        } else {
            self.imports.len() + self.functions.len()
        }
    }

    pub fn finish(&self) -> Vec<u8> {
        let mut module = Module::new();
        module
            .section(&self.types)
            .section(&self.imports)
            .section(&self.functions)
            .section(&self.memory)
            .section(&self.exports)
            .section(&self.codes)
            .section(&self.data);
        module.finish()
    }

    fn convert_wasm_param(out: &mut Vec<ValType>, param: &Param) {
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

    fn convert_wasm_return(ret: &RetType) -> Vec<ValType> {
        match ret {
            RetType::Void => vec![],
            _ => todo!(),
        }
    }

    fn convert_wasm_function(&self, body: &MethodBody) -> Function {
        // TODO: locals
        let mut f = Function::new(vec![]);

        for inst in body.instructions.iter() {
            match inst {
                Instruction::Nop => {
                    f.instruction(WasmInst::Nop);
                }
                Instruction::LdStr(s) => {
                    let s = s.as_userstring().unwrap();
                    let str_data = &self.string_cache[&s];

                    // TODO: handling null
                    // if s.0 == 0 {
                    // }

                    f.instruction(WasmInst::I32Const(str_data.data_index));
                    f.instruction(WasmInst::I32Const(str_data.str_len as i32));
                    f.instruction(WasmInst::I32Const(0));
                }
                Instruction::Call(method) => {
                    if let Some(member) = method.as_member_ref() {
                        f.instruction(WasmInst::Call(self.member_ref_cache[&member].fn_index));
                    } else if let Some(method) = method.as_method_def() {
                        f.instruction(WasmInst::Call(self.method_cache[&method].fn_index));
                    } else if let Some(_spec) = method.as_method_spec() {
                        todo!("Call via MethodSpec");
                    } else {
                        panic!("Invalid Call argument");
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

        f.instruction(WasmInst::End);

        f
    }

    fn wasm_method_sig(&mut self, signature: MethodDefSig) -> SignatureCacheData {
        let types = &mut self.types;

        self.signature_cache
            .entry(signature)
            .or_insert_with_key(|signature| {
                let mut params = Vec::new();
                if signature
                    .calling_convension
                    .contains(MethodCallingConvension::HAS_THIS)
                {
                    params.push(VAL_PTR);
                }
                signature
                    .params
                    .iter()
                    .for_each(|p| Self::convert_wasm_param(&mut params, p));
                let type_index = types.len();
                let params = Rc::new(params);
                types.function(
                    params.iter().copied(),
                    Self::convert_wasm_return(&signature.ret),
                );
                SignatureCacheData {
                    type_index,
                    param_types: params,
                }
            })
            .clone()
    }

    fn get_method_full_name(
        type_namespace: Option<&str>,
        type_name: &str,
        method_name: &str,
    ) -> String {
        match type_namespace {
            Some(namespace) => format!("[{}]{}::{}", namespace, type_name, method_name),
            None => format!("{}::{}", type_name, method_name),
        }
    }

    pub fn emit_wasm_member_ref(
        &mut self,
        member_ref_index: MemberRefIndex,
        member_ref: &MemberRef,
        table: &MetadataTable,
        heap: Heap,
    ) {
        let member_sig = member_ref.resolve_signature(heap);
        let member_func_name = member_ref.name.resolve(heap).unwrap();
        let member_func_ty = self.wasm_method_sig(member_sig);
        match member_ref.class {
            MemberRefParent::TypeRefIndex(ty_ref) => {
                let ty_ref = ty_ref.resolve_table(table).unwrap();

                let namespace = ty_ref.type_namespace.resolve(heap);
                let name = ty_ref.type_name.resolve(heap);

                let fn_index = self.compute_fn_index(true);
                self.imports.import(
                    "env",
                    Some(&Self::get_method_full_name(
                        namespace,
                        name.unwrap(),
                        member_func_name,
                    )),
                    EntityType::Function(member_func_ty.type_index),
                );
                self.member_ref_cache
                    .insert(member_ref_index, MemberRefCacheData { fn_index });
            }
            other => todo!("{:?}", other),
        }
    }

    pub fn emit_wasm_type_header(
        &mut self,
        ty_index: TypeDefIndex,
        ty_def: &TypeDef,
        table: &MetadataTable,
        heap: Heap,
    ) {
        let namespace = ty_def.type_namespace.resolve(heap);
        let ty_name = ty_def.type_name.resolve(heap).unwrap();
        // TODO: create field type
        // let fields = ty_index.resolve_fields(table).unwrap();
        for (method_index, method_def) in ty_index.resolve_methods(table) {
            let full_name = Self::get_method_full_name(
                namespace,
                ty_name,
                method_def.name.resolve(heap).unwrap(),
            );
            let signature = method_def.resolve_signature(heap);
            self.emit_wasm_function_header(&full_name, method_index, signature);
        }
    }

    fn emit_wasm_function_header(
        &mut self,
        name: &str,
        index: MethodDefIndex,
        signature: MethodDefSig,
    ) {
        let sig_data = self.wasm_method_sig(signature);

        let fn_index = self.compute_fn_index(false);
        self.method_cache
            .insert(index, MethodCacheData { fn_index });
        self.functions.function(sig_data.type_index);
        self.exports.export(name, Export::Function(fn_index));
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

    for (index, member_ref) in table.list_member_ref() {
        ctx.emit_wasm_member_ref(index, member_ref, table, root.heap);
    }

    for (ty_index, ty_def) in table.list_type_def() {
        ctx.emit_wasm_type_header(ty_index, ty_def, table, root.heap);
    }

    for (_method_index, method_def) in table.list_method_def() {
        ctx.emit_wasm_function_body(&method_def.resolve_body(image));
    }

    ctx.finish()
}

pub fn dump(image: &Image) {
    let root = image.metadata_root();
    let table = &root.metadata_stream.table;

    for (_index, method) in table.list_method_def() {
        let name = method.name.resolve(root.heap).unwrap();
        let signature = method.resolve_signature(root.heap);
        println!("{}: {:?}", name, signature);
    }
}
