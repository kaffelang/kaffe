use kaffe_emit_ts::emit_module;
use kaffe_parser::parse;

#[test]
fn example_compiles_to_expected_typescript() {
    let source = include_str!("../../../examples/main.kaf");
    let module = parse(source).expect("parse failed");
    let ts = emit_module(&module);
    let expected = r#"type User = {
  id: string
  name: string
  age?: number
  role: "admin" | "editor" | "reader"
}

export function canEdit(user: User): boolean {
  return ["admin", "editor"].includes(user.role)
}

export function greet(user: User): string {
  return `Hello, ${user.name}`
}"#;
    assert_eq!(ts, expected);
}
