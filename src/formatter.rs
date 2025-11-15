use cel::common::ast::{CallExpr, ComprehensionExpr, EntryExpr, Expr, IdedExpr, ListExpr, MapExpr, SelectExpr, StructExpr};
use cel::common::value::CelVal;
use cel::parser::Parser;

use crate::doc::Doc;
use crate::options::FormatOptions;

/// Format a CEL expression string
pub fn format_cel(source: &str, options: &FormatOptions) -> anyhow::Result<String> {
    // Parse the CEL expression
    let parser = Parser::new();
    let ast = parser.parse(source).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // Format the AST
    let doc = format_expr(&ast);

    // Render to string
    let indent_str = if options.use_spaces {
        " ".repeat(options.indent_width)
    } else {
        "\t".to_string()
    };

    Ok(doc.render(options.max_width, &indent_str))
}

/// Format an IdedExpr
fn format_expr(expr: &IdedExpr) -> Doc {
    format_expr_inner(&expr.expr)
}

/// Format the inner Expr
fn format_expr_inner(expr: &Expr) -> Doc {
    match expr {
        Expr::Unspecified => Doc::text(""),

        Expr::Ident(name) => Doc::text(name.clone()),

        Expr::Literal(val) => format_literal(val),

        Expr::Select(select) => format_select(select),

        Expr::Call(call) => format_call(call),

        Expr::List(list) => format_list(list),

        Expr::Map(map) => format_map(map),

        Expr::Struct(s) => format_struct(s),

        Expr::Comprehension(comp) => format_comprehension(comp),
    }
}

/// Format a literal value
fn format_literal(val: &CelVal) -> Doc {
    match val {
        CelVal::Boolean(b) => Doc::text(b.to_string()),
        CelVal::Int(i) => Doc::text(i.to_string()),
        CelVal::UInt(u) => Doc::text(format!("{}u", u)),
        CelVal::Double(d) => {
            let s = d.to_string();
            // Ensure doubles always have a decimal point
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                Doc::text(format!("{}.0", s))
            } else {
                Doc::text(s)
            }
        }
        CelVal::String(s) => Doc::text(format!("\"{}\"", escape_string(s))),
        CelVal::Bytes(b) => Doc::text(format!("b\"{}\"", escape_bytes(b))),
        CelVal::Null => Doc::text("null"),
        CelVal::Duration(d) => Doc::text(format!("duration(\"{}s\")", d.as_secs())),
        CelVal::Timestamp(ts) => {
            // Format timestamp as RFC3339
            Doc::text(format!("timestamp({:?})", ts))
        }
        _ => Doc::text(format!("{:?}", val)),
    }
}

/// Format a select expression (field access)
fn format_select(select: &SelectExpr) -> Doc {
    let operand = format_expr(&select.operand);

    if select.test {
        // This is a has() macro
        Doc::concat(vec![
            Doc::text("has("),
            operand,
            Doc::text("."),
            Doc::text(select.field.clone()),
            Doc::text(")"),
        ])
    } else {
        // Regular field access
        Doc::concat(vec![
            operand,
            Doc::text("."),
            Doc::text(select.field.clone()),
        ])
    }
}

/// Format a function call
fn format_call(call: &CallExpr) -> Doc {
    let func_name = &call.func_name;

    // Check if this is a binary operator
    if is_binary_op(func_name) {
        return format_binary_op(func_name, &call.args);
    }

    // Check if this is a unary operator
    if is_unary_op(func_name) {
        return format_unary_op(func_name, &call.args);
    }

    // Check if this is a ternary conditional
    if func_name == "_?_:_" {
        return format_ternary(&call.args);
    }

    // Check if this is an index operation
    if func_name == "_[_]" {
        return format_index(&call.args);
    }

    // Regular function call or method call
    if let Some(target) = &call.target {
        // Method call: target.func(args)
        let target_doc = format_expr(target);
        let args_doc = format_args(&call.args);

        Doc::concat(vec![
            target_doc,
            Doc::text("."),
            Doc::text(func_name.clone()),
            Doc::wrap_parens(args_doc),
        ])
    } else {
        // Regular function call: func(args)
        let args_doc = format_args(&call.args);

        Doc::concat(vec![
            Doc::text(func_name.clone()),
            Doc::wrap_parens(args_doc),
        ])
    }
}

/// Check if a function name is a binary operator
fn is_binary_op(name: &str) -> bool {
    matches!(
        name,
        "_+_" | "_-_" | "_*_" | "_/_" | "_%_" | "_==_" | "_!=_" | "_<_" | "_<=_" | "_>_" | "_>=_"
            | "_&&_" | "_||_" | "@in"
    )
}

/// Check if a function name is a unary operator
fn is_unary_op(name: &str) -> bool {
    matches!(name, "!_" | "-_")
}

/// Format a binary operator
fn format_binary_op(op: &str, args: &[IdedExpr]) -> Doc {
    if args.len() != 2 {
        return Doc::text(format!("<invalid binary op: {}>", op));
    }

    let left = format_expr(&args[0]);
    let right = format_expr(&args[1]);
    let op_str = match op {
        "_+_" => "+",
        "_-_" => "-",
        "_*_" => "*",
        "_/_" => "/",
        "_%_" => "%",
        "_==_" => "==",
        "_!=_" => "!=",
        "_<_" => "<",
        "_<=_" => "<=",
        "_>_" => ">",
        "_>=_" => ">=",
        "_&&_" => "&&",
        "_||_" => "||",
        "@in" => "in",
        _ => op,
    };

    // Add parentheses for complex expressions
    let left_doc = if needs_parens(&args[0].expr, op) {
        Doc::parens(left)
    } else {
        left
    };

    let right_doc = if needs_parens(&args[1].expr, op) {
        Doc::parens(right)
    } else {
        right
    };

    Doc::group(Doc::concat(vec![
        left_doc,
        Doc::text(" "),
        Doc::text(op_str),
        Doc::line(),
        right_doc,
    ]))
}

/// Format a unary operator
fn format_unary_op(op: &str, args: &[IdedExpr]) -> Doc {
    if args.len() != 1 {
        return Doc::text(format!("<invalid unary op: {}>", op));
    }

    let operand = format_expr(&args[0]);
    let op_str = match op {
        "!_" => "!",
        "-_" => "-",
        _ => op,
    };

    Doc::concat(vec![Doc::text(op_str), operand])
}

/// Format a ternary conditional (a ? b : c)
fn format_ternary(args: &[IdedExpr]) -> Doc {
    if args.len() != 3 {
        return Doc::text("<invalid ternary>");
    }

    let cond = format_expr(&args[0]);
    let then_expr = format_expr(&args[1]);
    let else_expr = format_expr(&args[2]);

    Doc::group(Doc::concat(vec![
        cond,
        Doc::line(),
        Doc::text("? "),
        then_expr,
        Doc::line(),
        Doc::text(": "),
        else_expr,
    ]))
}

/// Format an index operation (a[b])
fn format_index(args: &[IdedExpr]) -> Doc {
    if args.len() != 2 {
        return Doc::text("<invalid index>");
    }

    let target = format_expr(&args[0]);
    let index = format_expr(&args[1]);

    Doc::concat(vec![
        target,
        Doc::text("["),
        index,
        Doc::text("]"),
    ])
}

/// Format function arguments
fn format_args(args: &[IdedExpr]) -> Doc {
    if args.is_empty() {
        return Doc::nil();
    }

    let arg_docs: Vec<Doc> = args.iter().map(format_expr).collect();
    Doc::join_comma(arg_docs, false)
}

/// Format a list literal
fn format_list(list: &ListExpr) -> Doc {
    if list.elements.is_empty() {
        return Doc::text("[]");
    }

    let elem_docs: Vec<Doc> = list.elements.iter().map(format_expr).collect();
    Doc::wrap_brackets(Doc::join_comma(elem_docs, true))
}

/// Format a map literal
fn format_map(map: &MapExpr) -> Doc {
    if map.entries.is_empty() {
        return Doc::text("{}");
    }

    let entry_docs: Vec<Doc> = map
        .entries
        .iter()
        .filter_map(|ided_entry| {
            match &ided_entry.expr {
                EntryExpr::MapEntry(entry) => {
                    let key = format_expr(&entry.key);
                    let value = format_expr(&entry.value);
                    Some(Doc::concat(vec![key, Doc::text(": "), value]))
                }
                _ => None,
            }
        })
        .collect();

    Doc::wrap_braces(Doc::join_comma(entry_docs, true))
}

/// Format a struct literal
fn format_struct(s: &StructExpr) -> Doc {
    let name = Doc::text(s.type_name.clone());

    if s.entries.is_empty() {
        return Doc::concat(vec![name, Doc::text("{}")]);
    }

    let field_docs: Vec<Doc> = s
        .entries
        .iter()
        .filter_map(|ided_entry| {
            match &ided_entry.expr {
                EntryExpr::StructField(field) => {
                    let key = Doc::text(field.field.clone());
                    let value = format_expr(&field.value);
                    Some(Doc::concat(vec![key, Doc::text(": "), value]))
                }
                _ => None,
            }
        })
        .collect();

    Doc::concat(vec![
        name,
        Doc::wrap_braces(Doc::join_comma(field_docs, true)),
    ])
}

/// Format a comprehension expression
fn format_comprehension(comp: &ComprehensionExpr) -> Doc {
    // Comprehensions are the result of macro expansion
    // Try to detect common patterns and format them back to macro form

    // Detect map() pattern:
    // accu_var = "@result", accu_init = [], loop_cond = true,
    // loop_step = @result + [expr], result = @result
    if comp.accu_var == "@result" {
        if let Expr::List(list) = &comp.accu_init.expr {
            if list.elements.is_empty() {
                // Check if loop_cond is true
                if is_literal_true(&comp.loop_cond.expr) {
                    // Check if loop_step is @result + [expr]
                    if let Some(map_expr) = extract_map_pattern(comp) {
                        let range = format_expr(&comp.iter_range);
                        let var = Doc::text(comp.iter_var.clone());
                        return Doc::concat(vec![
                            range,
                            Doc::text(".map("),
                            var,
                            Doc::text(", "),
                            map_expr,
                            Doc::text(")"),
                        ]);
                    }
                }

                // Check for filter() pattern:
                // loop_cond = predicate, loop_step = @result + [iter_var]
                if let Some(filter_expr) = extract_filter_pattern(comp) {
                    let range = format_expr(&comp.iter_range);
                    let var = Doc::text(comp.iter_var.clone());
                    return Doc::concat(vec![
                        range,
                        Doc::text(".filter("),
                        var,
                        Doc::text(", "),
                        filter_expr,
                        Doc::text(")"),
                    ]);
                }
            }
        }

        // Check for all() pattern:
        // accu_init = true, loop_step = @result && predicate
        if is_literal_true(&comp.accu_init.expr) {
            if let Some(all_expr) = extract_all_pattern(comp) {
                let range = format_expr(&comp.iter_range);
                let var = Doc::text(comp.iter_var.clone());
                return Doc::concat(vec![
                    range,
                    Doc::text(".all("),
                    var,
                    Doc::text(", "),
                    all_expr,
                    Doc::text(")"),
                ]);
            }
        }

        // Check for exists() pattern:
        // accu_init = false, loop_step = @result || predicate
        if is_literal_false(&comp.accu_init.expr) {
            if let Some(exists_expr) = extract_exists_pattern(comp) {
                let range = format_expr(&comp.iter_range);
                let var = Doc::text(comp.iter_var.clone());
                return Doc::concat(vec![
                    range,
                    Doc::text(".exists("),
                    var,
                    Doc::text(", "),
                    exists_expr,
                    Doc::text(")"),
                ]);
            }
        }
    }

    // Fallback: couldn't detect a macro pattern
    Doc::text("<comprehension>")
}

/// Check if an expression is the literal true
fn is_literal_true(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(CelVal::Boolean(true)))
}

/// Check if an expression is the literal false
fn is_literal_false(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(CelVal::Boolean(false)))
}

/// Extract map() pattern: @result + [expr]
fn extract_map_pattern(comp: &ComprehensionExpr) -> Option<Doc> {
    if let Expr::Call(call) = &comp.loop_step.expr {
        if call.func_name == "_+_" && call.args.len() == 2 {
            // Check if first arg is @result
            if let Expr::Ident(name) = &call.args[0].expr {
                if name == "@result" {
                    // Check if second arg is [expr]
                    if let Expr::List(list) = &call.args[1].expr {
                        if list.elements.len() == 1 {
                            return Some(format_expr(&list.elements[0]));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract filter() pattern
/// Pattern: loop_step = predicate ? (@result + [var]) : @result
fn extract_filter_pattern(comp: &ComprehensionExpr) -> Option<Doc> {
    // loop_step should be a ternary: predicate ? (@result + [var]) : @result
    if let Expr::Call(call) = &comp.loop_step.expr {
        if call.func_name == "_?_:_" && call.args.len() == 3 {
            let predicate = &call.args[0];
            let then_branch = &call.args[1];
            let else_branch = &call.args[2];

            // Check then_branch is @result + [var]
            if let Expr::Call(add_call) = &then_branch.expr {
                if add_call.func_name == "_+_" && add_call.args.len() == 2 {
                    if let Expr::Ident(name) = &add_call.args[0].expr {
                        if name == "@result" {
                            if let Expr::List(list) = &add_call.args[1].expr {
                                if list.elements.len() == 1 {
                                    if let Expr::Ident(var) = &list.elements[0].expr {
                                        if var == &comp.iter_var {
                                            // Check else_branch is @result
                                            if let Expr::Ident(else_name) = &else_branch.expr {
                                                if else_name == "@result" {
                                                    // This is a filter!
                                                    return Some(format_expr(predicate));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract all() pattern: @result && predicate
fn extract_all_pattern(comp: &ComprehensionExpr) -> Option<Doc> {
    if let Expr::Call(call) = &comp.loop_step.expr {
        if call.func_name == "_&&_" && call.args.len() == 2 {
            if let Expr::Ident(name) = &call.args[0].expr {
                if name == "@result" {
                    return Some(format_expr(&call.args[1]));
                }
            }
        }
    }
    None
}

/// Extract exists() pattern: @result || predicate
fn extract_exists_pattern(comp: &ComprehensionExpr) -> Option<Doc> {
    if let Expr::Call(call) = &comp.loop_step.expr {
        if call.func_name == "_||_" && call.args.len() == 2 {
            if let Expr::Ident(name) = &call.args[0].expr {
                if name == "@result" {
                    return Some(format_expr(&call.args[1]));
                }
            }
        }
    }
    None
}

/// Check if an expression needs parentheses based on operator precedence
fn needs_parens(expr: &Expr, parent_op: &str) -> bool {
    match expr {
        Expr::Call(call) if is_binary_op(&call.func_name) => {
            let child_prec = op_precedence(&call.func_name);
            let parent_prec = op_precedence(parent_op);
            child_prec < parent_prec
        }
        _ => false,
    }
}

/// Get operator precedence (higher = tighter binding)
fn op_precedence(op: &str) -> i32 {
    match op {
        "_||_" => 1,
        "_&&_" => 2,
        "_==_" | "_!=_" => 3,
        "_<_" | "_<=_" | "_>_" | "_>=_" | "@in" => 4,
        "_+_" | "_-_" => 5,
        "_*_" | "_/_" | "_%_" => 6,
        "!_" | "-_" => 7,
        _ => 0,
    }
}

/// Escape a string for CEL string literals
fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            c => vec![c],
        })
        .collect()
}

/// Escape bytes for CEL byte literals
fn escape_bytes(b: &[u8]) -> String {
    b.iter()
        .flat_map(|&byte| match byte {
            b'"' => vec![b'\\', b'"'],
            b'\\' => vec![b'\\', b'\\'],
            b'\n' => vec![b'\\', b'n'],
            b'\r' => vec![b'\\', b'r'],
            b'\t' => vec![b'\\', b't'],
            32..=126 => vec![byte],
            _ => format!("\\x{:02x}", byte).into_bytes(),
        })
        .map(|b| b as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_expr_str(input: &str) -> String {
        let options = FormatOptions::default();
        format_cel(input, &options).unwrap()
    }

    #[test]
    fn test_literals() {
        assert_eq!(format_expr_str("true"), "true");
        assert_eq!(format_expr_str("false"), "false");
        assert_eq!(format_expr_str("42"), "42");
        assert_eq!(format_expr_str("3.14"), "3.14");
        assert_eq!(format_expr_str(r#""hello""#), r#""hello""#);
        assert_eq!(format_expr_str("null"), "null");
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(format_expr_str("1 + 2"), "1 + 2");
        assert_eq!(format_expr_str("10 - 5"), "10 - 5");
        assert_eq!(format_expr_str("3 * 4"), "3 * 4");
        assert_eq!(format_expr_str("20 / 4"), "20 / 4");
        assert_eq!(format_expr_str("17 % 5"), "17 % 5");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(format_expr_str("5 > 3"), "5 > 3");
        assert_eq!(format_expr_str("10 >= 10"), "10 >= 10");
        assert_eq!(format_expr_str("2 < 7"), "2 < 7");
        assert_eq!(format_expr_str("4 <= 4"), "4 <= 4");
        assert_eq!(format_expr_str(r#""foo" == "foo""#), r#""foo" == "foo""#);
        assert_eq!(format_expr_str(r#""bar" != "baz""#), r#""bar" != "baz""#);
    }

    #[test]
    fn test_logical() {
        assert_eq!(format_expr_str("true && false"), "true && false");
        assert_eq!(format_expr_str("true || false"), "true || false");
        assert_eq!(format_expr_str("!true"), "!true");
    }

    #[test]
    fn test_lists() {
        assert_eq!(format_expr_str("[]"), "[]");
        assert_eq!(format_expr_str("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(format_expr_str(r#"["a", "b", "c"]"#), r#"["a", "b", "c"]"#);
    }

    #[test]
    fn test_maps() {
        assert_eq!(format_expr_str("{}"), "{}");
        assert_eq!(format_expr_str(r#"{"a": 1, "b": 2}"#), r#"{"a": 1, "b": 2}"#);
    }

    #[test]
    fn test_field_access() {
        assert_eq!(format_expr_str("user.name"), "user.name");
        assert_eq!(format_expr_str("user.profile.email"), "user.profile.email");
    }

    #[test]
    fn test_function_calls() {
        assert_eq!(format_expr_str("size([1, 2, 3])"), "size([1, 2, 3])");
        assert_eq!(format_expr_str(r#""hello".startsWith("h")"#), r#""hello".startsWith("h")"#);
    }

    #[test]
    fn test_index() {
        assert_eq!(format_expr_str("list[0]"), "list[0]");
        assert_eq!(format_expr_str(r#"map["key"]"#), r#"map["key"]"#);
    }

    #[test]
    fn test_ternary() {
        assert_eq!(
            format_expr_str(r#"x > 0 ? "positive" : "negative""#),
            r#"x > 0 ? "positive" : "negative""#
        );
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(format_expr_str("1 + 2 * 3"), "1 + 2 * 3");
        assert_eq!(format_expr_str("(1 + 2) * 3"), "(1 + 2) * 3");
        assert_eq!(format_expr_str("a && b || c"), "a && b || c");
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(
            format_expr_str("x > 5 && y < 10"),
            "x > 5 && y < 10"
        );
        assert_eq!(
            format_expr_str(r#"user.age >= 18 && user.active == true"#),
            r#"user.age >= 18 && user.active == true"#
        );
    }

    #[test]
    fn test_struct_literals() {
        let result = format_expr_str(r#"Person{name: "Alice", age: 25}"#);
        assert!(result.contains("Person"));
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_uint() {
        assert_eq!(format_expr_str("42u"), "42u");
        assert_eq!(format_expr_str("100u + 50u"), "100u + 50u");
    }

    #[test]
    fn test_map_macro() {
        assert_eq!(
            format_expr_str("[1, 2, 3].map(x, x * 2)"),
            "[1, 2, 3].map(x, x * 2)"
        );
        assert_eq!(
            format_expr_str("[1, 2, 3].map(x, x + 1)"),
            "[1, 2, 3].map(x, x + 1)"
        );
    }

    #[test]
    fn test_filter_macro() {
        assert_eq!(
            format_expr_str("[1, 2, 3, 4, 5].filter(x, x > 2)"),
            "[1, 2, 3, 4, 5].filter(x, x > 2)"
        );
        assert_eq!(
            format_expr_str("[1, 2, 3].filter(x, x % 2 == 0)"),
            "[1, 2, 3].filter(x, x % 2 == 0)"
        );
    }

    #[test]
    fn test_all_macro() {
        assert_eq!(
            format_expr_str("[1, 2, 3].all(x, x > 0)"),
            "[1, 2, 3].all(x, x > 0)"
        );
        assert_eq!(
            format_expr_str("[1, 2, 3].all(x, x < 10)"),
            "[1, 2, 3].all(x, x < 10)"
        );
    }

    #[test]
    fn test_exists_macro() {
        assert_eq!(
            format_expr_str("[1, 2, 3].exists(x, x == 2)"),
            "[1, 2, 3].exists(x, x == 2)"
        );
        assert_eq!(
            format_expr_str("[1, 2, 3].exists(x, x > 5)"),
            "[1, 2, 3].exists(x, x > 5)"
        );
    }

    #[test]
    fn test_nested_macros() {
        assert_eq!(
            format_expr_str("[[1, 2], [3, 4]].map(x, x.map(y, y * 2))"),
            "[[1, 2], [3, 4]].map(x, x.map(y, y * 2))"
        );
    }
}
