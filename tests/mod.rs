extern crate glsl_include;

use glsl_include::Preprocessor;

#[test]
fn no_include() {
    let src = include_str!("no_include.glsl").trim_right();
    let p = Preprocessor::new();
    let processed_src = p.run(src).unwrap();
    assert_eq!(src, processed_src);
}

#[test]
fn single_line_comments() {
    let src = include_str!("single_line_comments.glsl").trim_right();
    let p = Preprocessor::new();
    let processed_src = p.run(src).unwrap();
    assert_eq!(src.lines().count(), processed_src.lines().count());
}

#[test]
fn angle_bracket_include() {
    let src = include_str!("angle_bracket.glsl").trim_right();
    let p = Preprocessor::new().file("A.glsl", "void A() {}");
    let processed_src = p.run(src).unwrap();
    assert_eq!(src.lines().count(), processed_src.lines().count());
}

#[test]
fn quote_include() {
    let src = include_str!("quote.glsl").trim_right();
    let p = Preprocessor::new().file("A.glsl", "void A() {}");
    let processed_src = p.run(src).unwrap();
    assert_eq!(src.lines().count(), processed_src.lines().count());
}

#[test]
#[should_panic]
fn recursive_include() {
    let src = include_str!("quote.glsl").trim_right();
    Preprocessor::new()
        .file("A.glsl", "#include <A.glsl>\nvoid A() {}")
        .run(src)
        .unwrap();
}

#[test]
#[should_panic]
fn deep_recursive_include() {
    let src = include_str!("quote.glsl").trim_right();
    Preprocessor::new()
        .file("A.glsl", "#include <B.glsl>\nvoid A() {}")
        .file("B.glsl", "#include <C.glsl>\nvoid B() {}")
        .file("C.glsl", "#include <A.glsl>\nvoid C() {}")
        .run(src)
        .unwrap();
}

#[test]
#[should_panic]
fn non_existent_include() {
    let src = include_str!("quote.glsl").trim_right();
    let p = Preprocessor::new();
    p.run(src).unwrap();
}
