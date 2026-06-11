use kaffe_ast::{Expr, Item};
use kaffe_parser::parse;

#[test]
fn test_parse_type_alias() {
    let src = "type Product =\n  id: string\n  name: string\n  price: number";
    let module = parse(src).expect("parse failed");
    assert_eq!(module.items.len(), 1);
    if let Item::TypeAlias(ta) = &module.items[0] {
        assert_eq!(ta.name, "Product");
        assert_eq!(ta.fields.len(), 3);
    } else {
        panic!("expected TypeAlias");
    }
}

#[test]
fn test_parse_function() {
    let src = "fn add(a: number, b: number): number =>\n  a + b";
    let module = parse(src).expect("parse failed");
    assert_eq!(module.items.len(), 1);
    if let Item::Function(f) = &module.items[0] {
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
    } else {
        panic!("expected Function");
    }
}

#[test]
fn test_parse_export_fn() {
    let src = "+fn hello(name: string): string =>\n  \"Hello\"";
    let module = parse(src).expect("parse failed");
    assert!(matches!(&module.items[0], Item::Export(_)));
}

#[test]
fn test_parse_if_with_in_condition() {
    let src = "fn canEdit(role: string): boolean =>\n  if role in [\"admin\"]\n    true\n  else\n    false";
    let module = parse(src).expect("parse failed");

    let Item::Function(function) = &module.items[0] else {
        panic!("expected Function");
    };

    match &function.body {
        Expr::If { condition, .. } => {
            assert!(matches!(condition.as_ref(), Expr::In(_, _)));
        }
        _ => panic!("expected If expression"),
    }
}

#[test]
fn test_reject_unterminated_template_interpolation() {
    let src = "fn greet(name: string): string =>\n  \"Hello, #{name\"";
    let err = parse(src).expect_err("expected parse error");
    assert!(err.to_string().contains("unterminated template interpolation"));
}

#[test]
fn test_reject_empty_template_interpolation() {
    let src = "fn greet(name: string): string =>\n  \"Hello, #{ }\"";
    let err = parse(src).expect_err("expected parse error");
    assert!(err.to_string().contains("empty template interpolation"));
}

#[test]
fn test_reject_invalid_template_interpolation() {
    let src = "fn greet(name: string): string =>\n  \"Hello, #{.name}\"";
    let err = parse(src).expect_err("expected parse error");
    assert!(err.to_string().contains("invalid template interpolation"));
}
