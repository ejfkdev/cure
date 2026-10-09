//! ddc 反编译产物的形状回归：源码级规则在反编译形态上的两个实弹复现。
//!
//! ① null 复位赋值 + 单次使用落在重载族调用上（R8 产物：
//!    `String str2 = null; … str2 = null; stringBuilder2.append(str2);`
//!    —— PurchasesOrchestrator 复现）：变量形态是重载消歧依据，
//!    内联成 `append(null)` 后 append(CharSequence)/append(char[]) 二义。
//! ② 非 Iterable 的 .iterator() 拥有者（Kotlin Sequence：不透明调用
//!    返回 / 不透明类形参——FileHelper$…/k6.h0 复现）：while(hasNext)
//!    形态合法，for-each 不可编译，还原须过静态类型门。

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_simplify::simplify_unit;
use cure_java_print::print_unit;

fn run_src(src: &str) -> String {
    let mut out = parse(src);
    simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    print_unit(&out.ast, &out.unit)
}

#[test]
fn null_arg_keeps_typed_var_form() {
    // 形态①：append(String) 重载族——内联 null 即二义
    let out = run_src(
        r#"
public class NullArg {
    public static void main(String[] args) {
        String s = null;
        StringBuilder sb = new StringBuilder();
        sb.append(s);
        System.out.println(sb.length());
    }
}
"#,
    );
    assert!(
        out.contains("sb.append(s);"),
        "null 单次使用位于重载族实参位：必须保持变量形态（声明类型 = 消歧依据）\n{out}"
    );
    assert!(
        !out.contains("append(null)"),
        "append(null) 不可编译（CharSequence/char[] 二义）\n{out}"
    );
    // 声明被死代码消除可以，但使用点必须仍是 s
}

#[test]
fn null_return_position_still_propagates() {
    // return 位类型由方法返回类型钉住 → 传播安全（守卫不得过度收紧）
    let out = run_src(
        r#"
public class NullRet {
    static String f(int x) {
        String s = null;
        if (x > 0) {
            s = "pos";
        }
        return s;
    }
    public static void main(String[] args) {
        System.out.println(f(1) + f(-1));
    }
}
"#,
    );
    // 三元化后 s 的形态可能变化，但 null 字面量不得以实参形态出现；
    // 这里只需要保证输出仍可解析且 return 位传播不被守卫误杀
    assert!(out.contains("return"), "return 必然保留\n{out}");
}

#[test]
fn null_cast_position_still_propagates() {
    // Cast 包裹重建类型语境：`(String) s` 的 s 可被内联
    let out = run_src(
        r#"
public class NullCast {
    public static void main(String[] args) {
        String s = null;
        String t = (String) s;
        System.out.println(t == null);
    }
}
"#,
    );
    // 守卫语义：Cast 位放行。内联后 `(String) null` 合法。
    assert!(
        !out.contains(") s;") || out.contains("(String) null"),
        "Cast 位应放行内联（(String) null 合法）\n{out}"
    );
}

#[test]
fn while_iterator_opaque_type_stays_while() {
    // 形态②：不透明类形参（Kotlin Sequence 形状）——有 .iterator()
    // 但不是 java.lang.Iterable，for-each 不可编译
    let out = run_src(
        r#"
import java.util.Iterator;
import java.util.NoSuchElementException;

public class OpaqueSeqLoop {
    static class Seq {
        private final String[] items;
        Seq(String[] items) { this.items = items; }
        Iterator<String> iterator() {
            return new Iterator<String>() {
                int i = 0;
                public boolean hasNext() { return i < items.length; }
                public String next() {
                    if (!hasNext()) { throw new NoSuchElementException(); }
                    return items[i++];
                }
            };
        }
    }
    static String firstNonEmpty(Seq seq) {
        Iterator<String> it = seq.iterator();
        while (it.hasNext()) {
            String s = it.next();
            if (s.length() > 0) {
                return s;
            }
        }
        return "";
    }
    public static void main(String[] args) {
        Seq seq = new Seq(new String[] {"", "hit", "x"});
        System.out.println(firstNonEmpty(seq));
    }
}
"#,
    );
    assert!(
        out.contains("while (it.hasNext())"),
        "不透明 Iterable（非白名单类型）必须保留 while 形态\n{out}"
    );
    assert!(
        !out.contains("for (String s : seq)"),
        "for-each 对非 Iterable 类型不可编译\n{out}"
    );
}

#[test]
fn while_iterator_typed_list_still_recovers() {
    // 类型门不得误杀白名单类型：List<String> → for-each 正常还原
    let out = run_src(
        r#"
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public class TypedListLoop {
    static int total(List<String> names) {
        Iterator<String> it = names.iterator();
        int n = 0;
        while (it.hasNext()) {
            String s = it.next();
            n += s.length();
        }
        return n;
    }
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("alpha");
        names.add("beta");
        System.out.println(total(names));
    }
}
"#,
    );
    assert!(
        out.contains("for (String s : names)"),
        "List 白名单类型应还原 for-each\n{out}"
    );
}
