extern crate glsl_include;
#[macro_use]
extern crate indoc;

use glsl_include::Preprocessor;

#[test]
fn no_include() {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let processed_src = Preprocessor::new().run(src).unwrap();
    assert_eq!(src, processed_src);
}

#[test]
fn single_line_comments() {
    let src = indoc!(
        r#"
        //#include <A.glsl>
        // #include "A.glsl"
        void main() {}"#
    );
    let processed_src = Preprocessor::new().run(src).unwrap();
    assert_eq!(src, processed_src);
}

#[test]
fn angle_bracket_include() {
    let src = indoc!(
        r#"
        #include <A.glsl>
        void main() {}"#
    );
    let processed_src = Preprocessor::new()
        .file("A.glsl", "void A() {}")
        .run(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, processed_src);
}

#[test]
fn quote_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let processed_src = Preprocessor::new()
        .file("A.glsl", "void A() {}")
        .run(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, processed_src);
}

#[test]
fn duplicate_includes() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        #include "A.glsl"
        #include <A.glsl>
        void main() {}"#
    );
    let processed_src = Preprocessor::new()
        .file("A.glsl", "void A() {}")
        .run(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, processed_src);
}

#[test]
fn recursive_duplicate_includes() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        #include "A.glsl"
        #include <A.glsl>
        void main() {}"#
    );
    let a_src = indoc!(
        r#"
        #include "B.glsl"
        #include "B.glsl"
        void A() {}"#
    );
    let b_src = indoc!(
        r#"
        #include "C.glsl"
        #include "C.glsl"
        void B() {}"#
    );
    let c_src = indoc!(
        r#"
        void C() {}"#
    );
    let processed_src = Preprocessor::new()
        .file("A.glsl", a_src)
        .file("B.glsl", b_src)
        .file("C.glsl", c_src)
        .run(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void C() {}
        void B() {}
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, processed_src);
}

#[test]
#[should_panic]
fn recursive_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let a_src = indoc!(
        r#"
        #include "A.glsl"
        void A() {}"#
    );
    Preprocessor::new().file("A.glsl", a_src).run(src).unwrap();
}

#[test]
#[should_panic]
fn deep_recursive_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let a_src = indoc!(
        r#"
        #include <B.glsl>
        void A() {}"#
    );
    let b_src = indoc!(
        r#"
        #include "C.glsl"
        void B() {}"#
    );
    let c_src = indoc!(
        r#"
        #include "A.glsl"
        void C() {}"#
    );
    Preprocessor::new()
        .file("A.glsl", a_src)
        .file("B.glsl", b_src)
        .file("C.glsl", c_src)
        .run(src)
        .unwrap();
}

#[test]
#[should_panic]
fn non_existent_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    Preprocessor::new().run(src).unwrap();
}
