# glsl-include &emsp; [![Build Status]][travis] [![Build Status-win]][appveyor] [![Latest Version]][crates.io]

[Build Status]: https://travis-ci.org/jshrake/glsl-include.svg?branch=master
[travis]: https://travis-ci.org/jshrake/glsl-include
[Build Status-win]: https://ci.appveyor.com/api/projects/status/3btnmxtpmabotj26?svg=true
[appveyor]: https://ci.appveyor.com/project/jshrake/glsl-include
[Latest Version]: https://img.shields.io/crates/v/glsl-include.svg
[crates.io]: https://crates.io/crates/glsl-include

**glsl-include is a rust library for expanding #include directives in GLSL source strings**

---

## Quick Start

Cargo.toml:
```toml
[dependencies]
glsl-include = "0.2"
```

main.rs:

```rust
extern crate glsl_include;
use glsl_include::Context;

fn main () {
    let main = r"
        #version 410
        #include <platform.glsl>
        #include <common.glsl>
        out vec4 fragColor;
        void main () {
            fragColor = vec4(1.0);
        }";
    let platform = "void platform_fn() {}";
    let common = "uniform float iTime;";
    let (expanded_src, source_map) = Context::new()
        .include("platform.glsl", platform)
        .include("common.glsl",common)
        .expand_to_string(main).unwrap();
}
```

## #pragma include

The library also expands `#pragma include` statements with no additonal configuration required.

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
