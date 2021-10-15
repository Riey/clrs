use clrs_pe::pe::Image;

fn main() {
    let file = include_bytes!("../HelloWorld/bin/Release/net5.0/mscorlib.dll");
    let image = Image::from_bytes(file).unwrap();
    let wasm = clrs_compiler::compile(&image);
    println!("{}", wasmprinter::print_bytes(&wasm).unwrap());
    wasmparser::validate(&wasm).unwrap();
}
