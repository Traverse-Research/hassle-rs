use hassle_rs::*;

#[repr(C)]
#[repr(packed)]
pub struct MinimalHeader {
    four_cc: u32,
    hash_digest: [u32; 4],
}

// zero_digest & get_digest from https://github.com/gwihlidal/dxil-signing/blob/master/rust/src/main.rs

fn zero_digest(buffer: &mut [u8]) {
    let buffer_ptr: *mut u8 = buffer.as_mut_ptr();
    let header_ptr: *mut MinimalHeader = buffer_ptr as *mut _;
    let header_mut: &mut MinimalHeader = unsafe { &mut *header_ptr };
    header_mut.hash_digest[0] = 0x0;
    header_mut.hash_digest[1] = 0x0;
    header_mut.hash_digest[2] = 0x0;
    header_mut.hash_digest[3] = 0x0;
}

fn get_digest(buffer: &[u8]) -> [u32; 4] {
    let buffer_ptr: *const u8 = buffer.as_ptr();
    let header_ptr: *const MinimalHeader = buffer_ptr as *const _;
    let header_ref: &MinimalHeader = unsafe { &*header_ptr };
    let digest: [u32; 4] = [
        header_ref.hash_digest[0],
        header_ref.hash_digest[1],
        header_ref.hash_digest[2],
        header_ref.hash_digest[3],
    ];
    digest
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
