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
hassle-rs = "0.11.0"
```

Then acquire `dxcompiler.dll` on Windows or `libdxcompiler.so` on Linux directly from [AppVeyor](https://ci.appveyor.com/project/antiagainst/directxshadercompiler/branch/master/artifacts), or compile it from source according to [the instructions](https://github.com/microsoft/DirectXShaderCompiler/blob/main/docs/DxcOnUnix.rst#building-dxc) in the [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) GitHub repository and make sure it's in the executable environment. See our [support table](##Supported-DXC-versions-on-non-Windows) below for specific compatibility notes on non-Windows OSes.

DxcValidator also requires `dxil.dll` which can be grabbed from any recent DXC release: https://github.com/microsoft/DirectXShaderCompiler/releases/latest
More info: https://www.wihlidal.com/blog/pipeline/2018-09-16-dxil-signing-post-compile/

## Supported DXC versions on non-Windows

Outside of Windows (e.g. Unix) the emulated COM API needed its fair share of fixes to match the layout on Windows. This results in repetitive API breakage that is hard to detect from within `hassle-rs`: be sure to math the `hassle-rs` release below to a minimum DXC commit to prevent runtime failures outside of Windows!

| Since `hassle-rs` | DXC release | Git commit |
|-|-|-|
| 0.10.0 | [v1.7.2212](https://github.com/microsoft/DirectXShaderCompiler/releases/tag/v1.7.2212) | https://github.com/microsoft/DirectXShaderCompiler/commit/47f31378a9b51894b0465b33ac1d10ce6349a468 |
| 0.5.1 (if using `intellisense`) | [release-1.6.2012](https://github.com/microsoft/DirectXShaderCompiler/tree/release-1.6.2012) | https://github.com/microsoft/DirectXShaderCompiler/commit/2ade6f84d6b95bfd96eec1d6d15e3aa3b519d180 |

When compiling on MacOS with `clang`, or Linux with `gcc`, be sure to have at least https://github.com/microsoft/DirectXShaderCompiler/commit/af14220b45d3ce46e0bad51ce79655e41d07c478 (also included in `release-1.6.2012`).

<details>
<summary>Interesting DXC commits pertaining Unix support</summary>

These patches have had an effect on `hassle-rs` compatibility over time:

- [`[Linux] WinAdapter: Remove virtual dtors from IUnknown to fix vtable ABI`](https://github.com/microsoft/DirectXShaderCompiler/commit/47f31378a9b51894b0465b33ac1d10ce6349a468)
- [`Linux: Implement prefix-counted BSTR allocation in SysAllocStringLen`](https://github.com/microsoft/DirectXShaderCompiler/commit/2ade6f84d6b95bfd96eec1d6d15e3aa3b519d180)
- [`[linux-port] Support full IID comparison on GCC`](https://github.com/microsoft/DirectXShaderCompiler/commit/af14220b45d3ce46e0bad51ce79655e41d07c478)

</details>

## Usage examples

### Compile HLSL into SPIR-V

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

### Compile HLSL into DXIL and validate it

```rust
let dxil = compile_hlsl("test.cs.hlsl", test_cs, "main", "cs_6_5", args, &[]).unwrap();
let result = validate_dxil(&dxil); // Only a Windows machine in Developer Mode can run non-validated DXIL

if let Some(err) = result.err() {
    println!("validation failed: {}", err);
}
```

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

## Contributions

 - Graham Wihlidal
 - Tiago Carvalho
 - Marijn Suijten
 - Tomasz Stachowiak
 - Manon Oomen

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/Traverse-Research/hassle-rs/issues) to see what known improvements are documented.
