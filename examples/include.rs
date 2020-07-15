use hassle_rs::*;
use rspirv::binary::Disassemble;
use rspirv::dr::load_bytes;

fn main() {
    let source = include_str!("include.hlsl");

    match compile_hlsl("include.hlsl", source, "copyCs", "cs_6_0", &["-spirv"], &[]) {
        Ok(spirv) => {
            let module = load_bytes(spirv).unwrap();
            println!("{}", module.disassemble());
        }
        // Could very well happen that one needs to recompile or download a dxcompiler.dll
        Err(s) => panic!("Failed to compile to SPIR-V: {}", s),
    }
}
