extern crate glsl_include;
#[macro_use]
extern crate indoc;

use glsl_include::{Context, FileLine, SourceMap};

#[test]
fn no_include() {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let (expand_to_stringed_src, _) = Context::new().expand_to_string(src).unwrap();
    assert_eq!(src, expand_to_stringed_src);
}

#[test]
fn single_line_comments() {
    let src = indoc!(
        r#"
        //#include <A.glsl>
        // #include "A.glsl"
        void main() {}"#
    );
    let (expand_to_stringed_src, _) = Context::new().expand_to_string(src).unwrap();
    assert_eq!(src, expand_to_stringed_src);
}

#[test]
fn angle_bracket_include() {
    let src = indoc!(
        r#"
        #include <A.glsl>
        void main() {}"#
    );
    let (expand_to_stringed_src, _) = Context::new()
        .include("A.glsl", "void A() {}")
        .expand_to_string(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, expand_to_stringed_src);
}

#[test]
fn quote_include() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let (expand_to_stringed_src, _) = Context::new()
        .include("A.glsl", "void A() {}")
        .expand_to_string(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, expand_to_stringed_src);
}

fn source_map_compare(left: &SourceMap, right: &SourceMap) -> bool {
    for (l, r) in left.iter().zip(right) {
        if l.file != r.file || l.line != r.line {
            return false;
        }
    }
    return true;
}

#[test]
fn source_map_1() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let (_, source_map) = Context::new()
        .include("A.glsl", "void A() {}")
        .expand_to_string(src)
        .unwrap();
    let expected = vec![
        FileLine {
            file: Some("A.glsl"),
            line: 0,
        },
        FileLine {
            file: None,
            line: 1,
        },
    ];
    println!("Expected {:?}, got {:?}", expected, source_map);
    assert_eq!(expected.len(), source_map.len());
    assert_eq!(source_map_compare(&expected, &source_map), true);
}

#[test]
fn source_map_2() {
    let src = indoc!(
        r#"
        #version 410 core
        #include "A.glsl"

        void main() {}"#
    );
    let (_, source_map) = Context::new()
        .include("A.glsl", "void A() {}\nvoid A2() {}")
        .expand_to_string(src)
        .unwrap();
    let expected = vec![
        FileLine {
            file: None,
            line: 0,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 0,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 1,
        },
        FileLine {
            file: None,
            line: 2,
        },
        FileLine {
            file: None,
            line: 3,
        },
    ];
    println!("Expected {:?}, got {:?}", expected, source_map);
    assert_eq!(expected.len(), source_map.len());
    assert_eq!(source_map_compare(&expected, &source_map), true);
}

#[test]
fn source_map_3() {
    let src = indoc!(
        r#"
        #version 410 core
        #include <A.glsl>
        #include <C.glsl>

        void main() {}"#
    );
    let (_, source_map) = Context::new()
        .include(
            "A.glsl",
            "#include <B.glsl>\nvoid A1(){}\nvoid A2(){}\n#include <C.glsl>",
        )
        .include("B.glsl", "void B1(){}\nvoid B2(){}")
        .include("C.glsl", "void C1(){}")
        .expand_to_string(src)
        .unwrap();
    let expected = vec![
        FileLine {
            file: None,
            line: 0,
        },
        FileLine {
            file: Some("B.glsl"),
            line: 0,
        },
        FileLine {
            file: Some("B.glsl"),
            line: 1,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 1,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 2,
        },
        FileLine {
            file: Some("C.glsl"),
            line: 0,
        },
        FileLine {
            file: None,
            line: 3,
        },
        FileLine {
            file: None,
            line: 4,
        },
    ];
    println!("Expected {:?}, got {:?}", expected, source_map);
    assert_eq!(expected.len(), source_map.len());
    assert_eq!(source_map_compare(&expected, &source_map), true);
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
    let (expand_to_stringed_src, _) = Context::new()
        .include("A.glsl", "void A() {}")
        .expand_to_string(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, expand_to_stringed_src);
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
    let (expand_to_stringed_src, _) = Context::new()
        .include("A.glsl", a_src)
        .include("B.glsl", b_src)
        .include("C.glsl", c_src)
        .expand_to_string(src)
        .unwrap();
    let expected = indoc!(
        r#"
        void C() {}
        void B() {}
        void A() {}
        void main() {}"#
    );
    assert_eq!(expected, expand_to_stringed_src);
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
    Context::new()
        .include("A.glsl", a_src)
        .expand_to_string(src)
        .unwrap();
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
        .expand_to_string(src);
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
    let result = Context::new().expand_to_string(src);
    match result {
        Err(ref e) => println!("{}", e),
        Ok(_) => (),
    };
    result.unwrap();
}
