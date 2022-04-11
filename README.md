# Fractal Generation App

This is a simple Rust application made with Druid which provides the tools to create complex fractal images. It uses a highly-performant multi-threaded and SIMD-enabled backend to compute fractals in near-real-time on the CPU.

## Compiling

Currently, the backend requires the target feature `avx2` to be available.

```sh
# debug [non-release] builds are significantly slower, useful for development only
cargo build --release
```

## Running

```sh
cargo run --release
```
