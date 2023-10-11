use hassle_rs::utils::DefaultIncludeHandler;
use hassle_rs::*;
use rspirv::binary::Disassemble;

fn main() {
    let dxc = Dxc::new(None).unwrap();
    let compiler = dxc.create_compiler().unwrap();
    let library = dxc.create_library().unwrap();
    let spirv = false;

    let args = &if spirv {
        vec!["-spirv", "-fspv-target-env=vulkan1.1spirv1.4"]
    } else {
        vec![]
    };

    let exports = compiler.compile(
        &library
            .create_blob_with_encoding_from_str(include_str!("exports.hlsl"))
            .unwrap(),
        "exports.hlsl",
        "",
        "lib_6_6",
        args,
        Some(&mut DefaultIncludeHandler {}),
        &[],
    );
    let use_exports = compiler.compile(
        &library
            .create_blob_with_encoding_from_str(include_str!("use-export.hlsl"))
            .unwrap(),
        "use-exports.hlsl",
        "",
        "lib_6_6",
        args,
        Some(&mut DefaultIncludeHandler {}),
        &[],
    );

    let exports = exports.ok().unwrap().get_result().unwrap();
    let use_exports = use_exports.ok().unwrap().get_result().unwrap();

    if spirv {
        let mut exports = rspirv::dr::load_bytes(exports).unwrap();
        let mut use_exports = rspirv::dr::load_bytes(use_exports).unwrap();
        let linked =
            spirv_linker::link(&mut [&mut exports, &mut use_exports], &Default::default()).unwrap();
        println!("{}", exports.disassemble());
        println!("{}", use_exports.disassemble());
        println!("{}", linked.disassemble());
    } else {
        let linker = dxc.create_linker().unwrap();

        linker.register_library("exports", &exports).unwrap();
        linker.register_library("useExports", &use_exports).unwrap();

        let binary = linker.link("copyCs", "cs_6_6", &["exports", "useExports"], &[]);
        match binary {
            Ok(spirv) => {
                let spirv = spirv.get_result().unwrap().to_vec::<u8>();

                println!("Outputting `linked.dxil`");
                println!("run `dxc -dumpbin linked.dxil` to disassemble");
                let _ = std::fs::write("./linked.dxil", spirv);
            }
            // Could very well happen that one needs to recompile or download a dxcompiler.dll
            Err(result) => {
                let error_blob = result.0.get_error_buffer().unwrap();

                panic!(
                    "Failed to link to SPIR-V: {}",
                    library.get_blob_as_string(&error_blob.into()).unwrap()
                );
            }
        }
    }
}
