use clrs_pe::pe::Image;

fn main() {
    let file = include_bytes!("../assets/HelloWorld.dll");
    let image = Image::from_bytes(file).unwrap();
    let wasm = clrs_compiler::compile(&image);
    println!("{}", wasmprinter::print_bytes(wasm).unwrap());
}
