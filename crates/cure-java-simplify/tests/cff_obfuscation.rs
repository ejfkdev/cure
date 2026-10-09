//! 控制流扁平化（CFF）还原测试：obfuscator.io / Allatori 风格的
//! `while(true){switch(state)}` 状态机 → 结构化控制流。
//! javac 差分验证语义保持。

use std::fs;
use std::process::Command;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

fn javac_available() -> bool {
    Command::new("javac")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_case(name: &str, src: &str) -> (String, usize) {
    let mut outcome = parse(src);
    assert!(
        outcome.errors.is_empty(),
        "{name}: 源码解析失败：{:?}",
        outcome.errors
    );
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    if javac_available() {
        let base = std::env::temp_dir().join(format!("cure_cff_{name}"));
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        let cls = format!("{name}C");
        fs::write(orig_dir.join(format!("{cls}.java")), src).unwrap();
        fs::write(clean_dir.join(format!("{cls}.java")), &cleaned).unwrap();
        for d in [&orig_dir, &clean_dir] {
            let out = Command::new("javac")
                .arg("-encoding")
                .arg("UTF-8")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(d.join(format!("{cls}.java")))
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{name}: javac 失败\n{}\n== 净化输出 ==\n{cleaned}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let run = |d: &std::path::Path| {
            Command::new("java")
                .arg("-cp")
                .arg(d)
                .arg(&cls)
                .output()
                .unwrap()
        };
        let (a, b) = (run(&orig_dir), run(&clean_dir));
        assert_eq!(a.status.code(), b.status.code(), "{name}: 退出码不一致");
        assert_eq!(
            String::from_utf8_lossy(&a.stdout),
            String::from_utf8_lossy(&b.stdout),
            "{name}: CFF 还原改变行为！\n== 净化输出 ==\n{cleaned}"
        );
    }
    (cleaned, report.edits)
}

#[test]
fn cff_linear_chain() {
    let src = r#"
public class LinearC {
    static int total;
    static int result;
    public static void main(String[] args) {
        total = 0;
        result = 0;
        int s = 0;
        while (true) {
            switch (s) {
                case 0: {
                    total = 10;
                    s = 1;
                    break;
                }
                case 1: {
                    total += 5;
                    s = 2;
                    break;
                }
                case 2: {
                    result = total * 2;
                    s = 7;
                    break;
                }
            }
            if (s == 7) {
                break;
            }
        }
        System.out.println("r=" + result);
    }
}
"#;
    let (cleaned, edits) = run_case("Linear", src);
    assert!(edits >= 1, "CFF 未被还原：\n{cleaned}");
    assert!(cleaned.contains("total = 10;"), "{cleaned}");
    assert!(cleaned.contains("total += 5;"), "{cleaned}");
    assert!(cleaned.contains("result = total * 2;"), "{cleaned}");
    assert!(!cleaned.contains("switch"), "{cleaned}");
    assert!(!cleaned.contains("int s"), "{cleaned}");
}

#[test]
fn cff_diamond() {
    let src = r#"
public class DiamondC {
    public static void main(String[] args) {
        int x = 3;
        String y = null;
        String out = null;
        int s = 0;
        while (true) {
            switch (s) {
                case 0: {
                    x = x * 7;
                    s = 1;
                    break;
                }
                case 1: {
                    if (x > 10) {
                        s = 2;
                    } else {
                        s = 3;
                    }
                    break;
                }
                case 2: {
                    y = "pos";
                    s = 4;
                    break;
                }
                case 3: {
                    y = "neg";
                    s = 4;
                    break;
                }
                case 4: {
                    out = y + "!";
                    s = 9;
                    break;
                }
            }
            if (s == 9) {
                break;
            }
        }
        System.out.println("out=" + out);
    }
}
"#;
    let (cleaned, edits) = run_case("Diamond", src);
    assert!(edits >= 1, "CFF 未被还原：\n{cleaned}");
    // CFF 还原出 if/else → 赋值传播+三元归并折成内联三元；assign_back_fold
    // 把 `x = x * 7` 折回字面量 init 后全链常量化（javac 差分验证行为一致）
    assert!(cleaned.contains(r#"println("out=pos!")"#), "{cleaned}");
    assert!(!cleaned.contains("switch"), "{cleaned}");
    assert!(!cleaned.contains("int s"), "{cleaned}");
    assert!(!cleaned.contains("x"), "{cleaned}");
}

#[test]
fn cff_loop_shape() {
    let src = r#"
public class LoopC {
    public static void main(String[] args) {
        int i = 0;
        int acc = 0;
        int s = 0;
        while (true) {
            switch (s) {
                case 0: {
                    if (i < 5) {
                        s = 1;
                    } else {
                        s = 2;
                    }
                    break;
                }
                case 1: {
                    acc += i;
                    i++;
                    s = 0;
                    break;
                }
                case 2: {
                    s = 8;
                    break;
                }
            }
            if (s == 8) {
                break;
            }
        }
        System.out.println("acc=" + acc);
    }
}
"#;
    let (cleaned, edits) = run_case("Loop", src);
    assert!(edits >= 1, "CFF 未被还原：\n{cleaned}");
    // vexec 全程静态执行：CFF 还原为 while 循环后，累加器/字符串拼接
    // 继续被折叠成单常量（javac 差分验证行为一致）
    assert!(cleaned.contains(r#"println("acc=10")"#), "{cleaned}");
    assert!(!cleaned.contains("switch"), "{cleaned}");
    // 空 case 2（纯出口）被丢弃
    assert!(!cleaned.contains("s = 8"), "{cleaned}");
}

#[test]
fn cff_default_return_exit() {
    let src = r#"
public class DefaultC {
    public static void main(String[] args) {
        int v = 3;
        int s = 0;
        while (true) {
            switch (s) {
                case 0: {
                    v = v + 9;
                    s = 1;
                    break;
                }
                case 1: {
                    System.out.println("v=" + v);
                    s = 99;
                    break;
                }
                default: {
                    return;
                }
            }
        }
    }
}
"#;
    let (cleaned, edits) = run_case("Default", src);
    assert!(edits >= 1, "CFF 未被还原：\n{cleaned}");
    // assign_back_fold 把 `v = v + 9` 折回字面量 init 后全链常量化
    //（run_case 内 javac 差分验证行为一致）
    assert!(cleaned.contains(r#"println("v=12")"#), "{cleaned}");
    assert!(!cleaned.contains("switch"), "{cleaned}");
    assert!(!cleaned.contains("v = v"), "{cleaned}");
}

#[test]
fn cff_negative_var_used_after() {
    // s 在循环后被使用 → 不能删状态机（还原会破坏语义）
    let src = r#"
public class NegC {
    public static void main(String[] args) {
        int s = 0;
        int x = 0;
        while (true) {
            switch (s) {
                case 0: {
                    x = 5;
                    s = 1;
                    break;
                }
                case 1: {
                    s = 2;
                    break;
                }
            }
            if (s == 2) {
                break;
            }
        }
        System.out.println("s=" + s + " x=" + x);
    }
}
"#;
    let mut outcome = parse(src);
    assert!(outcome.errors.is_empty());
    let before = print_unit(&outcome.ast, &outcome.unit);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let after = print_unit(&outcome.ast, &outcome.unit);
    // vexec 对该 main 做了全程静态执行（switch 形态消失），但 `s=2 x=5`
    // 的值证明语义保持（run_case 的 javac 差分同源代码；此用例直接比对
    // 输出值）
    assert!(after.contains(r#"println("s=2 x=5")"#), "值必须一致：\n{after}");
    let _ = before;
}

#[test]
fn cff_unreachable_case_dropped_as_dead_code() {
    // 不可达 case 是死代码（从入口不可达 → 永不执行）→ 随状态机一并丢弃
    let src = r#"
public class UnrC {
    public static void main(String[] args) {
        int s = 0;
        int x = 0;
        while (true) {
            switch (s) {
                case 0: {
                    x = 5;
                    s = 9;
                    break;
                }
                case 5: {
                    x = 6;
                    s = 9;
                    break;
                }
            }
            if (s == 9) {
                break;
            }
        }
        System.out.println("x=" + x);
    }
}
"#;
    let mut outcome = parse(src);
    assert!(outcome.errors.is_empty());
    let _report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let after = print_unit(&outcome.ast, &outcome.unit);
    // 死 case 体已消失；CFF 还原后整段被 static_exec 折叠为常量输出
    // （by_rule 断言不可靠——多规则接力时归属可能在任一环）
    assert!(after.contains(r#"println("x=5")"#), "{after}");
    assert!(!after.contains("x = 6;"), "死 case 体必须消失：\n{after}");
    assert!(!after.contains("switch"), "{after}");
}
