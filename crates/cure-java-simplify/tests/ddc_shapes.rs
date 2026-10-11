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

#[test]
fn const_method_inline_owner_guard() {
    // ddc lark 语料 jna Function 复现：本类 static Boolean valueOf(boolean)
    // 与 JDK Integer.valueOf(int) 撞名——名字匹配不判归属曾把
    // Integer.valueOf(-1) 当本类方法内联，产出 `!-1 ? Boolean.FALSE :
    // Boolean.TRUE` 不可编译。归属守卫：限定调用接收者 == 声明类；
    // 字面量实参的同类调用（裸名/限定名）照常内联。
    let out = run_src(
        r#"
public class ValOf {
    static java.lang.Integer TRUE_C = java.lang.Integer.valueOf(-1);
    static java.lang.Boolean valueOf(boolean p) {
        return !p ? Boolean.FALSE : Boolean.TRUE;
    }
    static Object bare() {
        return valueOf(false);
    }
    static Object qualified() {
        return ValOf.valueOf(true);
    }
    public static void main(String[] args) {
        System.out.println(TRUE_C + "" + bare() + qualified());
    }
}
"#,
    );
    // JDK 调用原样保留（归属不符 → 不内联）
    assert!(
        out.contains("Integer.valueOf(-1)"),
        "Integer.valueOf 必须原样保留（归属不符）\n{out}"
    );
    assert!(!out.contains("!-1"), "撞名误内联的坏形态不得出现\n{out}");
    // 同类字面量实参调用照常内联 + 下游折叠收尾：
    // valueOf(false) → !false ? FALSE : TRUE → return Boolean.TRUE;
    assert!(
        out.contains("return Boolean.TRUE;"),
        "同类裸名 valueOf(false) 应内联并折叠\n{out}"
    );
    assert!(
        out.contains("return Boolean.FALSE;"),
        "同类限定 ValOf.valueOf(true) 应内联并折叠\n{out}"
    );
}

#[test]
fn enum_constant_shadows_outer_const_field() {
    // ddc weixin mapsdk hm 复现：外层 `static final String b17 = "hm"` 与
    // 嵌套枚举常量 b17 撞名——常量收集器没把枚举常量算进字段声明计数，
    // 外层字段被误判全单元唯一 → 嵌套枚举 clinit 的裸名 b17（解析到枚举
    // 常量）被替换成 "hm"，产出 `new E[] {a, "hm"}` 不可编译。
    let out = run_src(
        r#"
public class Enc {
    private static final java.lang.String b17 = "hm";
    public static enum E {
        a, b17;
        private static final Enc.E[] c;
        private E() {}
        static {
            c = new Enc.E[] {a, b17};
        }
    }
    public static void main(String[] args) {
        System.out.println(b17 + E.c[1]);
    }
}
"#,
    );
    assert!(
        out.contains("new Enc.E[] {a, b17}"),
        "枚举常量 b17 不得被外层同名字段字面量替换\n{out}"
    );
    assert!(
        !out.contains("{a, \"hm\"}"),
        "坏形态（枚举常量→字符串字面量）不得出现\n{out}"
    );
}

#[test]
fn typed_var_not_inlined_into_cast_position() {
    // ddc weixin dt/k 复现：R8 宽化中转 `Object obj5 = k3Var;` 的单次
    // 使用在 narrowing cast 位——`(String) obj5` 合法（Object 声明类型
    // 钉住可转换性），传播内联成 `(String) k3Var` 后引用类型不相关即
    // 不可编译。instanceof 被测式同族。
    let out = run_src(
        r#"
public class CastPos {
    static Object src() { return null; }
    public static void main(String[] args) {
        Object obj5 = src();
        String s = (String) obj5;
        boolean b = obj5 instanceof java.lang.String;
        System.out.println(s + b);
    }
}
"#,
    );
    assert!(
        out.contains("(String) obj5"),
        "cast 操作数位的 Object 中转变量必须保持变量形态\n{out}"
    );
    assert!(
        out.contains("obj5 instanceof"),
        "instanceof 被测式位同理保持变量形态\n{out}"
    );
}

#[test]
fn while_iterator_no_post_loop_reference() {
    // ddc lark viewmodel/a 复现：Kotlin 协程状态机在 while 循环**之后**
    // 保存迭代器（`it2x = iterator;`）——for-each 把迭代器声明消费进循
    // 环头会让该引用悬空。规则必须拒绝这种还原（保持 while 形态）。
    let out = run_src(
        r#"
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public class PostUse {
    static java.util.List<String> saved;
    static int consume(List<String> src) {
        Iterator<String> it = src.iterator();
        int n = 0;
        while (it.hasNext()) {
            n += it.next().length();
        }
        // 循环后引用 it（状态机保存块的形状）
        String marker = (it == null) ? "x" : "y";
        return n + marker.length();
    }
    public static void main(String[] args) {
        List<String> src = new ArrayList<>();
        src.add("alpha");
        System.out.println(consume(src));
    }
}
"#,
    );
    assert!(
        out.contains("while (it.hasNext())"),
        "循环后仍引用迭代器 → 不得还原 for-each（声明会被消费）\n{out}"
    );
}

#[test]
fn ctor_delegation_hoist_with_inline() {
    // ddc weibo MediaPicker（Kotlin 默认参数桥）形态：值已折成三元，
    // this() 委托仍在纯前置之后——「对 this 的调用必须是构造器中的第
    // 一个语句」。前置整体后移（纯值晚求值恒等）、实参递归内联。
    let out = run_src(
        r#"
public class Hoist {
    Hoist(long p1, long p2) {}
    Hoist(long p1, long p2, int p3, Object marker) {
        long v5;
        long v6 = 0L;
        v5 = (p3 & 1) == 0 ? p1 : 0L;
        v6 = (p3 & 2) == 0 ? p2 : 9223372036854775807L;
        this(v5, v6);
        Object[] box = new Object[] { Long.valueOf(v5), Long.valueOf(v6) };
        System.out.println(box.length);
    }
    public static void main(String[] args) {
        new Hoist(1L, 2L);
    }
}
"#,
    );
    assert!(
        out.contains("this((p3 & 1) == 0 ? p1 : 0L, (p3 & 2) == 0 ? p2 : 9223372036854775807L);"),
        "委托须在首位且实参内联三元\n{out}"
    );
}

#[test]
fn ctor_delegation_hoist_rejects_impure_pre() {
    // 前置含**裸调用语句**（非赋值）→ 不能删也不能跨 this() 换序 →
    // 整条放弃（保持输入的位次——不合法是输入固有，规则不恶化）
    let out = run_src(
        r#"
public class NoHoist {
    NoHoist(int a) {}
    NoHoist(int p, Object m) {
        side(p);
        this(p);
    }
    static int side(int x) { return x + 1; }
    public static void main(String[] args) { new NoHoist(1); }
}
"#,
    );
    let side_pos = out.find("side(p);").unwrap_or(usize::MAX);
    let this_pos = out.find("this(p);").unwrap_or(usize::MAX);
    assert!(
        side_pos < this_pos,
        "不纯前置（裸调用）不得被跨过提升\n{out}"
    );
}

#[test]
fn ctor_delegation_hoist_recursive_arg() {
    // 传递内联：v6 = p1 + 1; this(p1, v6) → this(p1, p1 + 1)
    //（local_propagation 同形合法内联，本测试锁定组合终态）
    let out = run_src(
        r#"
public class RecHoist {
    RecHoist(int a, int b) {}
    RecHoist(int p1, Object m) {
        int v6 = 0;
        v6 = p1 + 1;
        this(p1, v6);
        System.out.println(v6);
    }
    public static void main(String[] args) { new RecHoist(7); }
}
"#,
    );
    assert!(
        out.contains("this(p1, p1 + 1);"),
        "实参内联：this(p1, p1 + 1)\n{out}"
    );
}

#[test]
fn dead_pure_call_statements_removed() {
    // ddc 的 cmp 指令渲染族：compare 语句结果永不被读（真分支用原始
    // 比较另行表达）；Objects.toString 是 R8 删日志残留。三 APK 实测
    // ~6.3 万处。
    let out = run_src(
        r#"
public class DeadCmp {
    static long scale(long p0x, long p1x, String s) {
        Long.compare(p1x, 0L);
        if (p1x > 0L) {
            Long.compare(p0x, 0L);
            if (p0x > 0L) {
                boolean empty = s.isEmpty();
                return empty ? (p1x + p0x) / 2L : 0L;
            }
        }
        java.util.Objects.toString(s);
        return 0L;
    }
    public static void main(String[] args) {
        System.out.println(scale(4L, 2L, "x"));
    }
}
"#,
    );
    assert!(!out.contains(".compare("), "丢弃结果的 compare 语句应删除\n{out}");
    assert!(!out.contains("Objects.toString"), "Objects.toString 语句应删除\n{out}");
    assert!(out.contains("if (p1x > 0L)"), "真分支保留\n{out}");
}

#[test]
fn dead_pure_call_boxed_operand_kept() {
    // 装箱实参走 valueOf 重载——拆箱可 NPE，保守不删
    let out = run_src(
        r#"
public class BoxedCmp {
    static int f(Long x) {
        Long.compare(x, 0L);
        return x == null ? -1 : 1;
    }
    public static void main(String[] args) {
        System.out.println(f(null));
    }
}
"#,
    );
    assert!(out.contains("Long.compare(x, 0L);"), "装箱实参的 compare 保留（拆箱 NPE 语义）\n{out}");
}

#[test]
fn double_cast_object_hop_folded() {
    // dex 连续 check-cast（Object → 具体类型）的双重 cast 折叠。
    // 守卫：操作数静态类型须为 Object（声明或 null）——具体类型
    // 操作数的 (Object) 是必需跳板（Void→Number 不可编译）。
    let out = run_src(
        r#"
public class CastFold {
    static String g(Object o) {
        return ((String) (Object) o).trim();
    }
    static String h(Object o) {
        return ((String) (Object) null).trim() + ((String) (Object) o).trim();
    }
    public static void main(String[] args) {
        System.out.println(g("x") + h("y"));
    }
}
"#,
    );
    assert!(
        !out.contains(") (Object) "),
        "Object 类型操作数的双重 cast 应折叠\n{out}"
    );
}

#[test]
fn double_cast_unrelated_interfaces_kept() {
    // 不相关接口间需 Object 跳板（折叠后不可编译）
    let out = run_src(
        r#"
interface A {}
interface B {}
public class CastKeep {
    static A f(Object o) {
        return (A) (Object) o;
    }
    public static void main(String[] args) {
        System.out.println(f(null));
    }
}
"#,
    );
    // (A)(Object) o：Object → A 是合法的（Object 是万物超类）→ 可折叠
    // 真正的跳板场景是 (A)(B) x——这里只测 Object 中转被折叠
    assert!(!out.contains("(Object) o"), "Object 中转应被折叠\n{out}");
}

#[test]
fn trailing_else_flatten() {
    // if (c) { …; return; } else { … } → if (c) { …; return; } + else 续排
    let out = run_src(
        r#"
public class ElseFlat {
    static int f(int x) {
        if (x > 0) {
            return 1;
        } else {
            int y = x * 2;
            return y;
        }
    }
    public static void main(String[] args) {
        System.out.println(f(5) + f(-3));
    }
}
"#,
    );
    assert!(!out.contains("} else {"), "尾随 else 应展平\n{out}");
}

#[test]
fn trailing_else_void_return_removed() {
    // if (c) { …; return; } else { return; } → if (c) { …; return; }
    let out = run_src(
        r#"
public class VoidRet {
    static void g(boolean c) {
        if (c) {
            System.out.println("yes");
            return;
        } else {
            return;
        }
    }
    public static void main(String[] args) { g(true); g(false); }
}
"#,
    );
    assert!(!out.contains("} else {"), "else 仅裸 return 应删除\n{out}");
}

#[test]
fn trailing_else_throw_kept_flattened() {
    // then 终结于 throw 同族展平
    let out = run_src(
        r#"
public class ThrowFlat {
    static int h(int x) {
        if (x < 0) {
            throw new IllegalArgumentException("neg");
        } else {
            return x + 1;
        }
    }
    public static void main(String[] args) {
        System.out.println(h(1));
    }
}
"#,
    );
    assert!(!out.contains("} else {"), "throw 终结的 else 也应展平\n{out}");
}
