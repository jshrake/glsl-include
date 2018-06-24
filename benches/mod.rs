#[macro_use]
extern crate criterion;
extern crate glsl_include;
#[macro_use]
extern crate indoc;

use criterion::Criterion;
use glsl_include::Context;

fn no_include(c: &mut Criterion) {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let p = Context::new();
    c.bench_function("no includes", move |b| b.iter(|| p.expand(src).unwrap()));
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
    let mut p = Context::new();
    p.include("A.glsl", a_src);
    p.include("B.glsl", b_src);
    p.include("C.glsl", c_src);
    c.bench_function("three includes", move |b| b.iter(|| p.expand(src).unwrap()));
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
    let mut p = Context::new();
    p.include("A.glsl", a_src);
    p.include("B.glsl", b_src);
    p.include("C.glsl", c_src);
    c.bench_function("recursive includes", move |b| {
        b.iter(|| p.expand(src).unwrap())
    });
}

criterion_group!(benches, no_include, three_includes, recursive_includes);
criterion_main!(benches);
