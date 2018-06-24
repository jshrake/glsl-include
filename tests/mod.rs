extern crate glsl_include;
#[macro_use]
extern crate indoc;

use glsl_include::Context;

#[test]
fn no_include() {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let expand_src = Context::new().expand(src).unwrap();
    assert_eq!(src, &expand_src);
}

#[test]
fn single_line_comments() {
    let src = indoc!(
        r#"
        //#include <A.glsl>
        // #include "A.glsl"
        void main() {}"#
    );
    let expand_src = Context::new().expand(src).unwrap();
    assert_eq!(src, &expand_src);
}

#[test]
fn angle_bracket_include() {
    let src = indoc!(
        r#"
        #include <A.glsl>
        void main() {}"#
    );
    let expand_src = Context::new()
        .include("A.glsl", "void A() {}")
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        #line 1 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
}

#[test]
fn quote_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let expand_src = Context::new()
        .include("A.glsl", "void A() {}")
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        #line 1 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
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
    let expand_src = Context::new()
        .include("A.glsl", "void A() {}")
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        #line 3 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
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
    let expand_src = Context::new()
        .include("A.glsl", a_src)
        .include("B.glsl", b_src)
        .include("C.glsl", c_src)
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void C() {}
        #line 2 0
        void B() {}
        #line 2 0
        void A() {}
        #line 3 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
}

#[test]
fn pragma_include() {
    let src = indoc!(
        r#"
        #pragma  include "A.glsl"
        void main() {}"#
    );
    let expand_src = Context::new()
        .include("A.glsl", "void A() {}")
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        #line 1 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
}

#[test]
fn weird_pragma_include() {
    let src = indoc!(
        r#"
        #pragmapragma include "A.glsl"
        #pragma pragma include "A.glsl"
        # pragma  include "A.glsl"
        void main() {}"#
    );
    let expand_src = Context::new()
        .include("A.glsl", "void A() {}")
        .expand(src)
        .unwrap();
    let expected = indoc!(
        r#"
        #pragmapragma include "A.glsl"
        #pragma pragma include "A.glsl"
        void A() {}
        #line 3 0
        void main() {}"#
    );
    assert_eq!(expected, expand_src);
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
    Context::new().include("A.glsl", a_src).expand(src).unwrap();
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
    let result = Context::new()
        .include("A.glsl", a_src)
        .include("B.glsl", b_src)
        .include("C.glsl", c_src)
        .expand(src);
    match result {
        Err(ref e) => println!("{}", e),
        Ok(_) => (),
    };
    result.unwrap();
}

#[test]
#[should_panic]
fn non_existent_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let result = Context::new().expand(src);
    match result {
        Err(ref e) => println!("{}", e),
        Ok(_) => (),
    };
    result.unwrap();
}
