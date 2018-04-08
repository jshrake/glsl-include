#[cfg(feature = "benches")]
#[macro_use]
extern crate criterion;
extern crate glsl_include;
#[macro_use]
extern crate indoc;

use glsl_include::Preprocessor;
use criterion::Criterion;

fn no_include(c: &mut Criterion) {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let p = Preprocessor::new();
    c.bench_function("no includes", move |b| b.iter(|| p.run(src).unwrap()));
}

fn three_includes(c: &mut Criterion) {
    let a_src = indoc!(
        r"
        void A() {
        }"
    );
    let b_src = indoc!(
        r"
        void B() {
        }"
    );
    let c_src = indoc!(
        r"
        void C() {
        }"
    );
    let src = indoc!(
        r"
        #include <A.glsl>
        #include <B.glsl>
        #include <C.glsl>
        void main() {
        }"
    );
    let p = Preprocessor::new()
        .file("A.glsl", a_src)
        .file("B.glsl", b_src)
        .file("C.glsl", c_src);
    c.bench_function("three includes", move |b| b.iter(|| p.run(src).unwrap()));
}

fn recursive_includes(c: &mut Criterion) {
    let a_src = indoc!(
        r"
        #include <B.glsl>
        void A() {
        }"
    );
    let b_src = indoc!(
        r"
        #include <C.glsl>
        void B() {
        }"
    );
    let c_src = indoc!(
        r"
        void C() {
        }"
    );
    let src = indoc!(
        r"
        #include <A.glsl>
        void main() {
        }"
    );
    let p = Preprocessor::new()
        .file("A.glsl", a_src)
        .file("B.glsl", b_src)
        .file("C.glsl", c_src);
    c.bench_function("recursive includes", move |b| {
        b.iter(|| p.run(src).unwrap())
    });
}

criterion_group!(benches, no_include, three_includes, recursive_includes);
criterion_main!(benches);
