#![allow(dead_code)]
#![allow(non_upper_case_globals)]

//! # Hassle
//!
//! This crate provides an FFI layer and idiomatic rust wrappers for the new [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) library.
//!
//! ## Simple example
//!
//! ```rust
//!
//! let code = "
//!     Texture2D<float4> g_input    : register(t0, space0);
//!     RWTexture2D<float4> g_output : register(u0, space0);
//!
//!     [numthreads(8, 8, 1)]
//!     void copyCs(uint3 dispatchThreadId : SV_DispatchThreadID)
//!     {
//!         g_output[dispatchThreadId.xy] = g_input[dispatchThreadId.xy];
//!     }";
//!
//! let ir = compile_hlsl(
//!     "shader_filename.hlsl",
//!     code,
//!     "copyCs",
//!     "cs_6_1",
//!     &vec!["-spirv"],
//!     &vec![
//!         ("MY_DEFINE", Some("Value")),
//!         ("OTHER_DEFINE", None)
//!     ],
//! );
//! ```

#[macro_use]
extern crate bitflags;

pub mod ffi;
pub mod os;
pub mod utils;

#[macro_use]
pub mod wrapper;

#[cfg(windows)]
pub mod intellisense;

pub use crate::ffi::*;
pub use crate::utils::{compile_hlsl, validate_dxil};
pub use crate::wrapper::*;
