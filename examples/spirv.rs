use hassle_rs::*;
use rspirv::binary::Disassemble;
use rspirv::dr::load_bytes;

fn main() {
    let source = include_str!("copy.hlsl");

    let spirv = match compile_hlsl("copy.hlsl", source, "copyCs", "cs_6_0", &["-spirv"], &[]) {
        Ok(OperationOutput { messages, blob }) => {
            if let Some(m) = messages {
                eprintln!("Compiled to SPIR-V with warnings:\n{m}");
            }
            blob
        }
        // Could very well happen that one needs to recompile or download a dxcompiler.dll
        Err(e) => panic!("Failed to compile to SPIR-V: {e:?}"),
    };
    let module = load_bytes(spirv).unwrap();
    println!("{}", module.disassemble());
}
