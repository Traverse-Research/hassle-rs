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

    let mut dxil = compile_hlsl("copy.hlsl", source, "copyCs", "cs_6_0", &[], &[]).unwrap();

    zero_digest(&mut dxil);

    let without_digest = get_digest(&dxil);
    println!("Before validation: {:?}", without_digest);

    let validated_dxil = validate_dxil(&dxil).unwrap();

    let with_digest = get_digest(&validated_dxil);

    println!("After validation: {:?}", with_digest);
}
