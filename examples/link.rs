//! Compile two HLSL sources to `lib_6_8` DXIL libraries, register them with
//! [`DxcLinker`], and link them into a single `cs_6_8` shader binary.

use hassle_rs::wrapper::{Dxc, DxcIncludeHandler};
use hassle_rs::OperationOutput;

struct NoopIncludes;
impl DxcIncludeHandler for NoopIncludes {
    fn load_source(&mut self, _filename: String) -> Option<String> {
        None
    }
}

fn compile_lib(dxc: &Dxc, name: &str, source: &str) -> Vec<u8> {
    let compiler = dxc.create_compiler().unwrap();
    let library = dxc.create_library().unwrap();
    let blob = library.create_blob_with_encoding_from_str(source).unwrap();

    let result = compiler
        .compile(
            &blob,
            name,
            "", // entry-point ignored for lib targets
            "lib_6_8",
            &["-HV", "2021"],
            Some(&mut NoopIncludes),
            &[],
        )
        .expect("compile call");

    let OperationOutput { messages, blob } =
        OperationOutput::from_operation_result(result).expect("compile success");
    if let Some(m) = messages {
        eprintln!("[{name}] compile messages:\n{m}");
    }
    blob
}

fn main() {
    let dxc = Dxc::new(None).expect("load dxcompiler");
    let library = dxc.create_library().unwrap();

    // Compile each source to a DXIL library blob.
    let add_dxil = compile_lib(&dxc, "link-add.hlsl", include_str!("link-add.hlsl"));
    let main_dxil = compile_lib(&dxc, "link-main.hlsl", include_str!("link-main.hlsl"));

    // Wrap as DxcBlobs.
    let add_blob = library.create_blob_with_encoding(&add_dxil).unwrap();
    let main_blob = library.create_blob_with_encoding(&main_dxil).unwrap();

    // Register and link.
    let linker = dxc.create_linker().expect("create linker");
    linker
        .register_library("add", &add_blob)
        .expect("register add");
    linker
        .register_library("main", &main_blob)
        .expect("register main");

    let result = linker
        .link("main", "cs_6_8", &["add", "main"], &["-HV", "2021"])
        .expect("link call");

    let OperationOutput { messages, blob } =
        OperationOutput::from_operation_result(result).expect("link success");
    if let Some(m) = messages {
        eprintln!("link messages:\n{m}");
    }

    let bytes: &[u8] = blob.as_ref();
    println!("linked DXIL: {} bytes", bytes.len());
    assert!(!bytes.is_empty(), "linker produced an empty blob");
    // DXIL containers start with "DXBC" magic.
    assert_eq!(&bytes[..4], b"DXBC", "expected DXBC magic at start of blob");
    println!("DXBC magic present — linker output looks valid");
}
