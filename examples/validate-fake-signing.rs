#![allow(clippy::uninlined_format_args)]

#[repr(C)]
pub struct MinimalHeader {
    four_cc: u32,
    hash_digest: [u32; 4],
}

fn get_digest(buffer: &[u8]) -> [u32; 4] {
    let header_ptr = buffer.as_ptr().cast::<MinimalHeader>();
    let header_ref = unsafe { &*header_ptr };
    header_ref.hash_digest
}

use hassle_rs::{compile_hlsl, fake_sign_dxil_in_place, validate_dxil};

fn main() {
    let sources = [
        include_str!("copy-over-56.hlsl"),
        include_str!("copy-under-56.hlsl"),
    ];

    let mut all_matches = true;

    for (idx, source) in sources.iter().enumerate() {
        println!("Testing file: {}", idx);
        let mut dxil = compile_hlsl("copy.hlsl", source, "copyCs", "cs_6_0", &[], &[]).unwrap();

        let without_digest = get_digest(&dxil);

        let result = fake_sign_dxil_in_place(&mut dxil);
        assert!(result);

        let fake_signed_digest = get_digest(&dxil);

        if cfg!(windows) {
            let validated_dxil = validate_dxil(&dxil).unwrap();

            let with_digest = get_digest(&validated_dxil);

            println!(
                "\tAfter compilation: {:?}\n\tAfter dxil.dll: {:?}\n\tAfter fake signing: {:?}",
                without_digest, with_digest, fake_signed_digest
            );

            if fake_signed_digest != with_digest {
                println!("---- Mismatch in file {} ----", idx);
                all_matches &= false;
            }
        } else {
            println!(
                "\tAfter compilation: {:?}\n\tAfter fake signing: {:?}",
                without_digest, fake_signed_digest
            );
        }
    }

    if cfg!(windows) {
        if all_matches {
            println!("Success");
        }
    } else {
        println!("Warning: Signatures not validated against `dxil.dll` - this is only possible on Windows");
    }
}
