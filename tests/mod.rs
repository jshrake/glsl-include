extern crate glsl_include;
#[macro_use]
extern crate indoc;

use glsl_include::{FileLine, Preprocessor, SourceMap};

#[test]
fn no_include() {
    let src = indoc!(
        r"
        void main() {
        }"
    );
    let (processed_src, _) = Preprocessor::new().run(src).unwrap();
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
    let (processed_src, _) = Preprocessor::new().run(src).unwrap();
    assert_eq!(src, processed_src);
}

#[test]
fn angle_bracket_include() {
    let src = indoc!(
        r#"
        #include <A.glsl>
        void main() {}"#
    );
    let (processed_src, _) = Preprocessor::new()
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
    let (processed_src, _) = Preprocessor::new()
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

fn source_map_compare(left: &SourceMap, right: &SourceMap) -> bool {
    for (l, r) in left.iter().zip(right) {
        if l.file != r.file || l.line != r.line {
            return false;
        }
    }
    return true;
}

#[test]
fn no_source_map() {
    let src = indoc!(
        r#"
        void main() {}"#
    );
    let (_, source_map) = Preprocessor::new().run(src).unwrap();
    assert_eq!(source_map.is_none(), true);
}

#[test]
fn with_source_map() {
    let src = indoc!(
        r#"
        void main() {}"#
    );
    let (_, source_map) = Preprocessor::new().generate_source_map().run(src).unwrap();
    assert_eq!(source_map.is_some(), true);
}

#[test]
fn source_map_1() {
    let src = indoc!(
        r#"
        #include "A.glsl"
        void main() {}"#
    );
    let (_, source_map) = Preprocessor::new()
        .file("A.glsl", "void A() {}")
        .generate_source_map()
        .run(src)
        .unwrap();
    let expected = vec![
        FileLine {
            file: Some("A.glsl"),
            line: 1,
        },
        FileLine {
            file: None,
            line: 2,
        },
    ];
    let source_map = source_map.unwrap();
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
    let (_, source_map) = Preprocessor::new()
        .file("A.glsl", "void A() {}\nvoid A2() {}")
        .generate_source_map()
        .run(src)
        .unwrap();
    let expected = vec![
        FileLine {
            file: None,
            line: 1,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 2,
        },
        FileLine {
            file: Some("A.glsl"),
            line: 3,
        },
        FileLine {
            file: None,
            line: 4,
        },
        FileLine {
            file: None,
            line: 5,
        },
    ];
    let source_map = source_map.unwrap();
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
    let (processed_src, _) = Preprocessor::new()
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
    let (processed_src, _) = Preprocessor::new()
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
    let result = Preprocessor::new()
        .file("A.glsl", a_src)
        .file("B.glsl", b_src)
        .file("C.glsl", c_src)
        .run(src);
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
    let result = Preprocessor::new().run(src);
    match result {
        Err(ref e) => println!("{}", e),
        Ok(_) => (),
    };
    result.unwrap();
}
