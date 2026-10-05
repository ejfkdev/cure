//! 端到端测试：源码文本 → 解析 → 简化 → 格式化输出。
//!
//! 重点覆盖容错语义：语法错误区域原文保留，其余代码照常优化。

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

fn process(src: &str) -> (String, usize) {
    let mut out = parse(src);
    let report = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    (print_unit(&out.ast, &out.unit), report.edits)
}

#[test]
fn doc_example_through_real_source() {
    // 设计文档示例 1：真实 Java 源码进出
    let src = r#"
class A {
    int m() {
        int a = foo();
        int b = a;
        return b;
    }
}
"#;
    let (printed, edits) = process(src);
    assert!(edits > 0);
    assert!(printed.contains("return foo();"), "{printed}");
    assert!(!printed.contains("int b = a;"), "{printed}");
}

#[test]
fn broken_method_skipped_good_method_optimized() {
    // 坏方法原文保留，好方法照常优化 —— 容错语义的核心用例
    let src = r#"
class A {
    void broken() {
        int x = ;
        foo(= dead);
    }

    int good() {
        int a = foo();
        int b = a;
        return b;
    }
}
"#;
    let (printed, _edits) = process(src);
    // 坏的保留
    assert!(printed.contains("int x = ;"), "{printed}");
    assert!(printed.contains("foo(= dead);"), "{printed}");
    // 好的优化
    assert!(printed.contains("return foo();"), "{printed}");
    assert!(!printed.contains("int b = a;"), "{printed}");
}

#[test]
fn broken_statement_skipped_rest_optimized() {
    let src = r#"
class A {
    int m(boolean flag) {
        int a = 1;
        bar(= broken);
        if (flag) {
            return true;
        } else {
            return false;
        }
    }
}
"#;
    let (printed, _edits) = process(src);
    assert!(printed.contains("bar(= broken);"), "{printed}");
    // if/return 折叠发生在坏语句之后 —— 照常工作
    assert!(printed.contains("return flag;"), "{printed}");
}

#[test]
fn decompiler_artifacts_cleaned() {
    // 反编译产物大杂烩
    let src = r#"
class Order {
    private static final int MAX = 100;

    boolean check(int value) {
        boolean b = value > 10;
        if (b) {
            return true;
        } else {
            return false;
        }
    }

    String describe(int x) {
        if (x == 0) {
            return "zero";
        }
        int t = x * 1;
        int u = t;
        int self = 5;
        self = self;
        return "value=" + u;
    }
}
"#;
    let (printed, edits) = process(src);
    assert!(edits >= 4, "expect multiple edits, got {edits}");
    // 布尔返回折叠 + 传播
    assert!(printed.contains("return value > 10;"), "{printed}");
    // x * 1 → x, int u = t → int t 传播
    assert!(printed.contains("return \"value=\" + x;"), "{printed}");
    // 自赋值删除
    assert!(!printed.contains("self = self;"), "{printed}");
}

#[test]
fn idiomatic_code_left_alone() {
    // 已经自然的代码：只有确有收益的改写发生
    let src = r#"
class Ok {
    int sum(int[] xs) {
        int total = 0;
        for (int x : xs) {
            total += x;
        }
        return total;
    }
}
"#;
    let (printed, edits) = process(src);
    assert_eq!(edits, 0, "should not touch idiomatic code: {printed}");
    assert!(printed.contains("total += x;"), "{printed}");
}

#[test]
fn formatting_only_still_valid() {
    // 乱格式输入 → 规范输出（不优化路径）
    let src = "class A{void m(int x){if(x>0){System.out.println(\"hi\");}}}";
    let (printed, _) = process(src);
    assert!(printed.contains("if (x > 0) {"), "{printed}");
    assert!(printed.contains("    System.out.println(\"hi\");"), "{printed}");
}
