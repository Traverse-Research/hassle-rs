🌤 hassle-rs
========
[![Actions Status](https://github.com/Traverse-Research/hassle-rs/workflows/Continuous%20integration/badge.svg)](https://github.com/Traverse-Research/hassle-rs/actions)
[![Latest version](https://img.shields.io/crates/v/hassle-rs.svg)](https://crates.io/crates/hassle-rs)
[![Documentation](https://docs.rs/hassle-rs/badge.svg)](https://docs.rs/hassle-rs)
[![Lines of code](https://tokei.rs/b1/github/Traverse-Research/hassle-rs)](https://github.com/Traverse-Research/hassle-rs)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](../master/CODE_OF_CONDUCT.md)

[![Banner](banner.png)](https://traverseresearch.nl)

This crate provides an FFI layer and idiomatic rust wrappers for the new [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) library.

- [Documentation](https://docs.rs/hassle-rs)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hassle-rs = "0.5.4"
```

Then acquire `dxcompiler.dll` on Windows or `libdxcompiler.so` on Linux directly from [AppVeyor](https://ci.appveyor.com/project/antiagainst/directxshadercompiler/branch/master/artifacts), or compile it from source according to the instructions in the [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) GitHub repository and make sure it's in the executable environment.

DxcValidator also requires `dxil.dll` which can be grabbed from any recent Windows 10 SDK flight.
More info: https://www.wihlidal.com/blog/pipeline/2018-09-16-dxil-signing-post-compile/

## Compile HLSL into SPIR-V:

```rust
let spirv = compile_hlsl(
    "shader_filename.hlsl",
    code,
    "copyCs",
    "cs_6_5",
    &vec!["-spirv"],
    &vec![
        ("MY_DEFINE", Some("Value")),
        ("OTHER_DEFINE", None)
    ],
);
```

## Compile HLSL into DXIL and validate it:

```rust
let dxil = compile_hlsl("test.cs.hlsl", test_cs, "main", "cs_6_5", args, &[]).unwrap();
let result = validate_dxil(&dxil); // Only a Windows machine in Developer Mode can run non-validated DXIL

if let Some(err) = result.err() {
    println!("validation failed: {}", err);
}
```

## macOS support

One can build `libdxcompiler.dynlib` from source with [this commit](https://github.com/microsoft/DirectXShaderCompiler/pull/3062/commits/9f2b30aa333f22eed00bf37b3a9b94f5ff5d23fe) for `clang` or [the entire PR](https://github.com/microsoft/DirectXShaderCompiler/pull/3062) for `GCC`, by following [the DXC Unix build guide](https://github.com/microsoft/DirectXShaderCompiler/blob/master/docs/DxcOnUnix.rst#building-dxc). These patches [have been merged](https://github.com/microsoft/DirectXShaderCompiler/commit/af14220b45d3ce46e0bad51ce79655e41d07c478) to DXC and are available since `release-1.6.2012`.

## Linux support

Linux shares the same requirements as [macOS support](#macOS-support) when compiling from source but DXC provides prebuilt libraries for every commit on [AppVeyor](https://ci.appveyor.com/project/dnovillo/directxshadercompiler/history). These are linked from individual commits on GitHub too.

Furthermore semantics around `BSTR` (only used in `intellisense`) have [changed since hassle-rs 0.5.1](https://github.com/Traverse-Research/hassle-rs/commit/94670248cc01614c3a9c9f4d5288afb4040544bd) and require `libdxcompiler.so` compiled from at least [this commit](https://github.com/microsoft/DirectXShaderCompiler/commit/2ade6f84d6b95bfd96eec1d6d15e3aa3b519d180), also included in `release-1.6.2012`. `Intellisense` on hassle-rs `0.5.0` and below is not compatible with these newer libraries.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

## Contibutions

 - Graham Wihlidal
 - Tiago Carvalho
 - Marijn Suijten
 - Tomasz Stachowiak

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/Traverse-Research/hassle-rs/issues) to see what known improvements are documented.
