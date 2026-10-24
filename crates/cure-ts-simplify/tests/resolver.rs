use cure_engine::Lang;
use cure_ts_ast::TsId;
use cure_ts_parser::parse;

fn prepare(src: &str) -> (cure_ts_parser::ParseOutcome, TsId) {
    let mut out = parse(src);
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    let stmts: Vec<TsId> = out
        .unit
        .items
        .iter()
        .filter_map(|it| match it {
            cure_ts_ast::TopItem::Stmt(s) => Some(*s),
            _ => None,
        })
        .collect();
    let root = out.ast.module_block(stmts);
    Lang::prepare(&mut out.ast, root);
    (out, root)
}

fn refs(out: &cure_ts_parser::ParseOutcome, root: TsId, name: &str) -> Vec<TsId> {
    let mut v = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if let cure_ts_ast::NodeData::VarRef { name: sym } = out.ast.data(n) {
            if out.ast.sn(*sym) == name {
                v.push(n);
            }
        }
        for &c in out.ast.children(n) {
            stack.push(c);
        }
    }
    v
}

#[test]
fn block_shadowing_resolves_to_distinct_bindings() {
    let (out, root) = prepare(
        "let x = 1; { let x = 2; x; } x;",
    );
    let rs = refs(&out, root, "x");
    assert_eq!(rs.len(), 2, "{:?}", rs);
    let inner = rs.iter().find(|&&r| out.ast.is_local_var(r)).copied();
    let outer = rs.iter().find(|&&r| !out.ast.is_local_var(r)).copied();
    let (Some(inner), Some(outer)) = (inner, outer) else {
        panic!("expected one local and one top-level ref");
    };
    assert_ne!(out.ast.binding_id(inner), out.ast.binding_id(outer));
    assert!(out.ast.binding_id(inner).is_some());
    assert!(out.ast.binding_id(outer).is_some());
}

#[test]
fn top_level_bindings_are_not_local() {
    let (out, root) = prepare("var x = 1; x;");
    let r = refs(&out, root, "x")[0];
    let b = out.ast.binding_of(r).expect("module binding");
    assert!(b.module_level);
    assert!(!out.ast.is_local_var(r));
}

#[test]
fn closure_capture_marks_binding_and_blocks_locality() {
    let (out, root) = prepare(
        "var outer = function () { let y = 1; var inner = function () { return y; }; return inner; };",
    );
    let y_refs = refs(&out, root, "y");
    assert_eq!(y_refs.len(), 1);
    let b = out.ast.binding_of(y_refs[0]).expect("y binding");
    assert!(b.captured);
    assert!(!out.ast.is_local_var(y_refs[0]));
}

#[test]
fn named_function_expression_self_reference_is_not_capture() {
    let (out, root) = prepare(
        "var f = function g() { return g(); };",
    );
    let g_refs = refs(&out, root, "g");
    assert_eq!(g_refs.len(), 1);
    let b = out.ast.binding_of(g_refs[0]).expect("g binding");
    assert!(!b.captured, "self reference must not count as capture");
    assert_eq!(b.kind, cure_ts_ast::BindingKind::FunctionExprName);
    // 具名函数表达式的名字是不可变绑定：不能被当普通局部变量写删/传播。
    assert!(!out.ast.is_local_var(g_refs[0]));
}

#[test]
fn var_escaping_its_block_is_opaque() {
    let (out, root) = prepare(
        "var f = function () { { var x = 1; } return x; };",
    );
    let mut decl = None;
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if let cure_ts_ast::NodeData::VarDecl { name, .. } = out.ast.data(n) {
            if name == "x" {
                decl = Some(n);
            }
        }
        for &c in out.ast.children(n) {
            stack.push(c);
        }
    }
    let decl = decl.expect("var x");
    assert!(Lang::is_opaque(&out.ast, decl), "block-escaping var must be opaque");
}

#[test]
fn simple_var_stays_local_and_visible() {
    let (out, root) = prepare(
        "var f = function () { var x = 1; return x; };",
    );
    let r = refs(&out, root, "x")[0];
    assert!(out.ast.is_local_var(r));
    let b = out.ast.binding_of(r).expect("x binding");
    assert_eq!(b.ref_count, 1);
    assert_eq!(b.write_count, 0);
}

#[test]
fn top_level_decl_is_opaque_when_outside_root_writes_exist() {
    // 顶层声明在模块合成根里看不到 class 方法体（另一个顶层项）里的
    // 写——声明级规则必须拒绝触碰，否则 `let x=0` 会被当常量传播。
    let (out, root) = prepare(
        "let x = 0; class C { m() { ++x; } } x;",
    );
    let r = refs(&out, root, "x")[0];
    assert!(!out.ast.is_local_var(r));
    let mut decl = None;
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if let cure_ts_ast::NodeData::VarDecl { name, .. } = out.ast.data(n) {
            if name == "x" {
                decl = Some(n);
            }
        }
        for &c in out.ast.children(n) {
            stack.push(c);
        }
    }
    assert!(Lang::is_opaque(&out.ast, decl.expect("x decl")));
}
