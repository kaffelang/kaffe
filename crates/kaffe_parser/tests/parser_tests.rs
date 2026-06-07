use kaffe_ast::Item;
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
    let src = "export fn hello(name: string): string =>\n  \"Hello\"";
    let module = parse(src).expect("parse failed");
    assert!(matches!(&module.items[0], Item::Export(_)));
}
