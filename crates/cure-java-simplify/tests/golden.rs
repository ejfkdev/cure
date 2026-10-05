//! Java 侧 golden 测试：builder 构建 → simplify → printer 快照。
//!
//! 这些用例直接对应设计文档里的示例（§2、§9）。
//! 注意：builder 均为 `&mut self`，测试里一律平铺绑定，不做嵌套调用。

use cure_engine::kind::{BinOp, UnOp};
use cure_engine::Config;
use cure_java_ast::{JType, JavaAst, JavaId, Lit};
use cure_java_print::print;
use cure_java_simplify::simplify;

fn run(build: impl FnOnce(&mut JavaAst) -> JavaId) -> String {
    let mut ast = JavaAst::new();
    let root = build(&mut ast);
    simplify(&mut ast, root, &Config::default());
    print(&ast, root)
}

#[test]
fn doc_example_1_local_chain() {
    // int a = foo(); int b = a; return b;  →  return foo();
    let out = run(|a| {
        let foo = a.plain_call("foo", vec![]);
        let da = a.var_decl("a", JType::Int, Some(foo));
        let av = a.var("a");
        let db = a.var_decl("b", JType::Int, Some(av));
        let bv = a.var("b");
        let r = a.ret(Some(bv));
        a.block(vec![da, db, r])
    });
    assert_eq!(out, "return foo();");
}

#[test]
fn doc_example_2_boolean() {
    // boolean b = x > 10; if (b) { return true; } else { return false; }
    // → return x > 10;
    let out = run(|a| {
        let xv = a.var("x");
        let ten = a.lit(Lit::Int(10));
        let gt = a.bin(BinOp::Gt, xv, ten);
        let db = a.var_decl("b", JType::Bool, Some(gt));
        let bt = a.lit(Lit::Bool(true));
        let tr = a.ret(Some(bt));
        let then = a.block(vec![tr]);
        let bf = a.lit(Lit::Bool(false));
        let er = a.ret(Some(bf));
        let els = a.block(vec![er]);
        let bv = a.var("b");
        let i = a.if_(bv, then, Some(els));
        a.block(vec![db, i])
    });
    assert_eq!(out, "return x > 10;");
}

#[test]
fn doc_example_3_negated_return() {
    // 条件未知：if (c) { return false; } else { return true; }  →  return !c;
    let out = run(|a| {
        let dc = a.var_decl("c", JType::Bool, None);
        let bf = a.lit(Lit::Bool(false));
        let tr = a.ret(Some(bf));
        let then = a.block(vec![tr]);
        let bt = a.lit(Lit::Bool(true));
        let er = a.ret(Some(bt));
        let els = a.block(vec![er]);
        let cv = a.var("c");
        let i = a.if_(cv, then, Some(els));
        a.block(vec![dc, i])
    });
    assert_eq!(out, "boolean c;\nreturn !c;");
}

#[test]
fn known_const_condition_folds_fully() {
    // 条件已知（c = false）：折叠到 return false;（声明被传播消费）
    let out = run(|a| {
        let bf0 = a.lit(Lit::Bool(false));
        let dc = a.var_decl("c", JType::Bool, Some(bf0));
        let bf1 = a.lit(Lit::Bool(false));
        let tr = a.ret(Some(bf1));
        let then = a.block(vec![tr]);
        let bt = a.lit(Lit::Bool(true));
        let er = a.ret(Some(bt));
        let els = a.block(vec![er]);
        let cv = a.var("c");
        let i = a.if_(cv, then, Some(els));
        a.block(vec![dc, i])
    });
    assert_eq!(out, "return true;");
}

#[test]
fn redundant_double_cast() {
    let out = run(|a| {
        let xv = a.var("x");
        let inner = a.cast(JType::Ref("String".into()), xv);
        let outer = a.cast(JType::Ref("String".into()), inner);
        let r = a.ret(Some(outer));
        a.block(vec![r])
    });
    assert_eq!(out, "return (String) x;");
}

#[test]
fn redundant_typed_cast() {
    // String s = value; return (String) s;  →  return (String) value;
    //（value 类型未知 → cast 下沉到 value 上后保留）
    let out = run(|a| {
        let value = a.var("value");
        let ds = a.var_decl("s", JType::Ref("String".into()), Some(value));
        let sv = a.var("s");
        let c = a.cast(JType::Ref("String".into()), sv);
        let r = a.ret(Some(c));
        a.block(vec![ds, r])
    });
    assert_eq!(out, "return (String) value;");
}

#[test]
fn self_assign_removed() {
    let out = run(|a| {
        let one = a.lit(Lit::Int(1));
        let di = a.var_decl("i", JType::Int, Some(one));
        let iv = a.var("i");
        let iv2 = a.var("i");
        let sa = a.assign(iv, iv2);
        let keep = a.plain_call("keep", vec![]);
        let ks = a.expr_stmt(keep);
        a.block(vec![di, sa, ks])
    });
    assert_eq!(out, "int i = 1;\nkeep();");
}

#[test]
fn formatting_only_no_change() {
    let out = run(|a| {
        let xv = a.var("x");
        let zero = a.lit(Lit::Int(0));
        let cond = a.bin(BinOp::Gt, xv, zero);
        let da = a.plain_call("doA", vec![]);
        let sa = a.expr_stmt(da);
        let db = a.plain_call("doB", vec![]);
        let sb = a.expr_stmt(db);
        let then = a.block(vec![sa, sb]);
        let two = a.lit(Lit::Int(2));
        let er = a.ret(Some(two));
        let els = a.block(vec![er]);
        let i = a.if_(cond, then, Some(els));
        a.block(vec![i])
    });
    assert_eq!(
        out,
        "if (x > 0) {\n    doA();\n    doB();\n} else {\n    return 2;\n}"
    );
}

#[test]
fn while_and_arith() {
    let out = run(|a| {
        let iv = a.var("i");
        let inc = a.un(UnOp::PostInc, iv);
        let inc_s = a.expr_stmt(inc);
        let body = a.block(vec![inc_s]);
        let iv2 = a.var("i");
        let ten = a.lit(Lit::Int(10));
        let cond = a.bin(BinOp::Lt, iv2, ten);
        let w = a.while_(cond, body);
        a.block(vec![w])
    });
    assert_eq!(out, "while (i < 10) {\n    i++;\n}");
}

#[test]
fn nested_call_formatting() {
    let out = run(|a| {
        let obj = a.var("obj");
        let get = a.method_call(obj, "get", vec![]);
        let sv = a.var("s");
        let cat = a.method_call(sv, "concat", vec![get]);
        let r = a.ret(Some(cat));
        a.block(vec![r])
    });
    assert_eq!(out, r#"return s.concat(obj.get());"#);
}

#[test]
fn paren_removed_and_precedence_kept() {
    // 冗余括号删除
    let out = run(|a| {
        let xv = a.var("x");
        let one = a.lit(Lit::Int(1));
        let add = a.bin(BinOp::Add, xv, one);
        let p = a.paren(add);
        let r = a.ret(Some(p));
        a.block(vec![r])
    });
    assert_eq!(out, "return x + 1;");

    // 优先级需要的括号保留
    let out2 = run(|a| {
        let xv = a.var("x");
        let one = a.lit(Lit::Int(1));
        let add = a.bin(BinOp::Add, xv, one);
        let two = a.lit(Lit::Int(2));
        let mul = a.bin(BinOp::Mul, add, two);
        let r = a.ret(Some(mul));
        a.block(vec![r])
    });
    assert_eq!(out2, "return (x + 1) * 2;");
}

#[test]
fn const_condition_blocks() {
    let out = run(|a| {
        let live = a.plain_call("live", vec![]);
        let ls = a.expr_stmt(live);
        let then = a.block(vec![ls]);
        let dead = a.plain_call("dead", vec![]);
        let ds = a.expr_stmt(dead);
        let els = a.block(vec![ds]);
        let cond = a.lit(Lit::Bool(true));
        let i = a.if_(cond, then, Some(els));
        a.block(vec![i])
    });
    assert_eq!(out, "live();");
}

#[test]
fn self_compare_ints() {
    let out = run(|a| {
        let three = a.lit(Lit::Int(3));
        let di = a.var_decl("i", JType::Int, Some(three));
        let iv = a.var("i");
        let iv2 = a.var("i");
        let cmp = a.bin(BinOp::Eq, iv, iv2);
        let r = a.ret(Some(cmp));
        a.block(vec![di, r])
    });
    // i == i → true 后 i 成为死声明；死存储删除不在范围内，声明保留
    assert_eq!(out, "int i = 3;\nreturn true;");
}

#[test]
fn arith_identity_ints_only() {
    // int x; x + 0 → x
    let out = run(|a| {
        let dx = a.var_decl("x", JType::Int, None);
        let xv = a.var("x");
        let zero = a.lit(Lit::Int(0));
        let add = a.bin(BinOp::Add, xv, zero);
        let r = a.ret(Some(add));
        a.block(vec![dx, r])
    });
    assert_eq!(out, "int x;\nreturn x;");

    // 未声明类型的变量（无法证明是整数）→ 不化简
    let out2 = run(|a| {
        let yv = a.var("y");
        let zero = a.lit(Lit::Int(0));
        let add = a.bin(BinOp::Add, yv, zero);
        let r = a.ret(Some(add));
        a.block(vec![r])
    });
    assert_eq!(out2, "return y + 0;");
}

#[test]
fn decompiler_artifact_chain() {
    // 典型反编译产物：invoke → 存临时 → 返回前转型
    // Object r = obj.m(x); return ((String) r).length();
    let out = run(|a| {
        let xv = a.var("x");
        let obj = a.var("obj");
        let m = a.method_call(obj, "m", vec![xv]);
        let dr = a.var_decl("r", JType::Ref("Object".into()), Some(m));
        let rv = a.var("r");
        let cast = a.cast(JType::Ref("String".into()), rv);
        let len = a.method_call(cast, "length", vec![]);
        let ret = a.ret(Some(len));
        a.block(vec![dr, ret])
    });
    // r 传播后 cast 落到调用上；r 是 Object，cast 必须保留
    assert_eq!(out, "return ((String) obj.m(x)).length();");
}
