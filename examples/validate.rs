#![allow(clippy::uninlined_format_args)]

use hassle_rs::*;

#[repr(C)]
#[repr(packed)]
pub struct MinimalHeader {
    four_cc: u32,
    hash_digest: [u32; 4],
}

// zero_digest & get_digest from https://github.com/gwihlidal/dxil-signing/blob/master/rust/src/main.rs

fn zero_digest(buffer: &mut [u8]) {
    let header_ptr = buffer.as_mut_ptr().cast::<MinimalHeader>();
    let header_ref = unsafe { &mut *header_ptr };
    header_ref.hash_digest = [0; 4];
}

fn get_digest(buffer: &[u8]) -> [u32; 4] {
    let header_ptr = buffer.as_ptr().cast::<MinimalHeader>();
    let header_ref = unsafe { &*header_ptr };
    header_ref.hash_digest
}

fn main() {
    let source = include_str!("copy.hlsl");

    let mut dxil = match compile_hlsl("copy.hlsl", source, "copyCs", "cs_6_0", &[], &[]) {
        Ok(OperationOutput { messages, blob }) => {
            if let Some(m) = messages {
                eprintln!("Compiled to DXIL with warnings:\n{m}");
            }
            blob
        }
        // Could very well happen that one needs to recompile or download a dxcompiler.dll
        Err(e) => panic!("Failed to compile to DXIL: {:?}", e),
    };

    zero_digest(&mut dxil);

    let without_digest = get_digest(&dxil);
    println!("Before validation: {:?}", without_digest);

    let validated_dxil = match validate_dxil(&dxil) {
        Ok(OperationOutput { messages, blob }) => {
            if let Some(m) = messages {
                eprintln!("Validated DXIL with warnings:\n{m}");
            }
            blob
        }
        // Could very well happen that one needs to recompile or download a dxcompiler.dll
        Err(e) => panic!("Failed to validate DXIL: {:?}", e),
    };

    let with_digest = get_digest(&validated_dxil);

    println!("After validation: {:?}", with_digest);
}
