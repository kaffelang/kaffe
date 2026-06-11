use kaffe_emit_ts::emit_module;
use kaffe_parser::parse;

#[test]
fn test_emit_type_alias() {
    let src = "type Product =\n  id: string\n  name: string\n  price: number";
    let module = parse(src).expect("parse failed");
    let ts = emit_module(&module);
    assert!(ts.contains("type Product = {"));
    assert!(ts.contains("id: string"));
    assert!(ts.contains("}"));
}

#[test]
fn test_emit_function() {
    let src = "fn add(a: number, b: number): number =>\n  a + b";
    let module = parse(src).expect("parse failed");
    let ts = emit_module(&module);
    assert!(ts.contains("function add(a: number, b: number): number {"));
    assert!(ts.contains("return a + b"));
}

#[test]
fn test_emit_template_string() {
    let src = "fn greet(name: string): string =>\n  \"Hello, #{name}\"";
    let module = parse(src).expect("parse failed");
    let ts = emit_module(&module);
    assert!(ts.contains("`Hello, ${name}`"));
}

#[test]
fn test_emit_template_string_escapes_literal_sequences() {
    let src = "fn greet(name: string): string =>\n  \"C:\\\\tmp ${name} #{name}\"";
    let module = parse(src).expect("parse failed");
    let ts = emit_module(&module);
    assert!(ts.contains("`C:\\\\tmp \\${name} ${name}`"));
}

#[test]
fn test_emit_in_operator() {
    let src = "fn canEdit(role: string): boolean =>\n  role in [\"admin\", \"editor\"]";
    let module = parse(src).expect("parse failed");
    let ts = emit_module(&module);
    assert!(ts.contains(".includes("));
}
