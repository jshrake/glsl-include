# glsl-include &emsp; [![Build Status]][travis] [![Build Status-win]][appveyor] [![Latest Version]][crates.io] [![Rustc Version 1.15+]][rustc]

[Build Status]: https://travis-ci.org/jshrake/glsl-include.svg?branch=master
[travis]: https://travis-ci.org/jshrake/glsl-include
[Build Status-win]: https://ci.appveyor.com/api/projects/status/3btnmxtpmabotj26?svg=true
[appveyor]: https://ci.appveyor.com/project/jshrake/glsl-include
[Latest Version]: https://img.shields.io/crates/v/glsl-include.svg
[crates.io]: https://crates.io/crates/glsl-include
[Rustc Version 1.15+]: https://img.shields.io/badge/rustc-1.15+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2017/02/02/Rust-1.15.html

**glsl-include is a rust library for expanding #include directives in GLSL source strings**

---

## Quick Start

Cargo.toml:

```toml
[dependencies]
glsl-include = "0.1.0"
```

## Benchmarks

Benchmarking makes use of the optional criterion dependency, which depends on rust 1.23

```
cargo bench --features "criterion"
```

The workflow I currently use for benchmarking a changeset:

```
git checkout master; cargo bench --features "criterion"
git checkout feature-branch; cargo bench --features "criterion"
```

For the best results with criterion, install `gnuplot` (macos: `brew install gnuplot`)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
