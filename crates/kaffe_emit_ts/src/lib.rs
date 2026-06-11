use kaffe_ast::*;

pub fn emit_module(module: &Module) -> String {
    let mut out = String::new();
    for (i, item) in module.items.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\n");
        }
        emit_item(&mut out, item, false);
    }
    out
}

fn emit_item(out: &mut String, item: &Item, exported: bool) {
    match item {
        Item::TypeAlias(type_alias) => emit_type_alias(out, type_alias, exported),
        Item::Function(function) => emit_function(out, function, exported),
        Item::Export(inner) => emit_item(out, inner, true),
    }
}

fn emit_type_alias(out: &mut String, type_alias: &TypeAlias, exported: bool) {
    if exported {
        out.push_str("export ");
    }
    out.push_str("type ");
    out.push_str(&type_alias.name);
    out.push_str(" = {\n");
    for field in &type_alias.fields {
        out.push_str("  ");
        out.push_str(&field.name);
        if field.optional {
            out.push('?');
        }
        out.push_str(": ");
        out.push_str(&emit_type_expr(&field.ty));
        out.push('\n');
    }
    out.push('}');
}

fn emit_function(out: &mut String, function: &FunctionDecl, exported: bool) {
    if exported {
        out.push_str("export ");
    }
    out.push_str("function ");
    out.push_str(&function.name);
    out.push('(');
    for (i, param) in function.params.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&param.name);
        out.push_str(": ");
        out.push_str(&emit_type_expr(&param.ty));
    }
    out.push(')');
    if let Some(return_type) = &function.return_type {
        out.push_str(": ");
        out.push_str(&emit_type_expr(return_type));
    }
    out.push_str(" {\n");
    emit_function_body(out, &function.body, 1);
    out.push('}');
}

fn emit_function_body(out: &mut String, body: &Expr, indent: usize) {
    match body {
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => emit_if_statement(out, condition, then_branch, else_branch.as_deref(), indent),
        _ => {
            write_indent(out, indent);
            out.push_str("return ");
            out.push_str(&emit_expr(body));
            out.push('\n');
        }
    }
}

fn emit_if_statement(
    out: &mut String,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    indent: usize,
) {
    write_indent(out, indent);
    out.push_str("if (");
    out.push_str(&emit_expr(condition));
    out.push_str(") {\n");
    emit_function_body(out, then_branch, indent + 1);
    write_indent(out, indent);
    out.push('}');
    if let Some(else_branch) = else_branch {
        out.push_str(" else {\n");
        emit_function_body(out, else_branch, indent + 1);
        write_indent(out, indent);
        out.push('}');
    }
    out.push('\n');
}

fn emit_type_expr(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::StringLit(value) => format!("\"{}\"", escape_string(value)),
        TypeExpr::Union(types) => types.iter().map(emit_type_expr).collect::<Vec<_>>().join(" | "),
        TypeExpr::Array(inner) => format!("{}[]", emit_type_expr(inner)),
    }
}

fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::StringLiteral(value) => format!("\"{}\"", escape_string(value)),
        Expr::TemplateLiteral(parts) => {
            let mut out = String::from("`");
            for part in parts {
                match part {
                    TemplatePart::Literal(value) => out.push_str(&escape_template_literal(value)),
                    TemplatePart::Interpolated(expr) => {
                        out.push_str("${");
                        out.push_str(&emit_expr(expr));
                        out.push('}');
                    }
                }
            }
            out.push('`');
            out
        }
        Expr::NumberLiteral(value) => value.to_string(),
        Expr::BoolLiteral(value) => value.to_string(),
        Expr::Identifier(name) => name.clone(),
        Expr::MemberAccess(target, field) => format!("{}.{}", emit_expr(target), field),
        Expr::BinaryOp(left, op, right) => {
            format!("{} {} {}", emit_expr(left), emit_bin_op(op), emit_expr(right))
        }
        Expr::In(left, right) => format!("{}.includes({})", emit_expr(right), emit_expr(left)),
        Expr::Array(items) => format!(
            "[{}]",
            items.iter().map(emit_expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut out = String::from("(() => {\n");
            emit_if_statement(&mut out, condition, then_branch, else_branch.as_deref(), 1);
            out.push_str("})()");
            out
        }
    }
}

fn emit_bin_op(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Eq => "===",
        BinOp::Neq => "!==",
        BinOp::Gt => ">",
        BinOp::Lt => "<",
        BinOp::Gte => ">=",
        BinOp::Lte => "<=",
    }
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_template_literal(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '$' if matches!(chars.peek(), Some('{')) => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }

    out
}
