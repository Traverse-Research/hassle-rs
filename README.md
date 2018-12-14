hassle
========
This crate provides an FFI layer and idiomatic rust wrappers for the new [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) library.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hassle = "0.1.0"
```

and add this to your crate root:

```rust
extern crate hassle;
```

Then acquire `dxcompiler.dll` directly from [AppVeyor](https://ci.appveyor.com/project/antiagainst/directxshadercompiler/branch/master/artifacts) or compile it from source according to the instructions in the [DirectXShaderCompiler](https://github.com/Microsoft/DirectXShaderCompiler) GitHub repository.

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/Jasper-Bekkers/hassle-rs/issues) to see what known improvements are documented.
