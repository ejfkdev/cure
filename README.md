# cure

语言无关的代码语义简化 / 规范化引擎（Rust）。

目标：**在不改变程序语义的前提下**，把代码中冗余、重复、不自然的表达方式简化掉，
使代码接近人类手写风格。服务于反编译器（jcdc / ddc 等），但不依赖任何具体反编译器；
也可独立用于 AST 级别的代码清理与格式化。

不是 DCE、不是性能优化：默认不做死类/死方法删除，只做局部表达与控制流形态的规范化。

## crate 划分（可按需单独引用）

| crate | 职责 | 依赖 |
|---|---|---|
| `cure-engine` | 语言无关核心：`Lang` trait、Pattern 匹配、Rewrite 事务、Pass/fixed-point、成本模型、通用规则 | 无 |
| `cure-java-ast` | Java AST（arena + NodeId）+ 编译单元/成员结构、builder、`Lang` 实现（副作用/类型/作用域分析） | engine |
| `cure-java-parser` | **容错式** Java 源码解析器：语法错误跳过并原文保留，好代码照常解析 | ast |
| `cure-java-print` | 规范格式化打印器（方法体 `print` / 整文件 `print_unit`，优先级最小括号） | ast |
| `cure-java-simplify` | Java 规则 + `simplify`（方法体）/ `simplify_unit`（整文件）门面 | engine + ast |
| `cure-cli` | `cure` 命令行：源码文件 → 简化 + 格式化 | 以上全部 |

按需取用示例：

- 只要"解析成 AST"：`cure-java-ast` + `cure-java-parser`。
- 只要"格式化"：+ `cure-java-print`。
- 只要"优化"：`cure-engine` + `cure-java-ast` + `cure-java-simplify`。
- 全都要：用 CLI 或把五件套都引进去。

## 设计要点

- 语义保持是硬约束：任何 rewrite 必须通过语义检查（副作用、求值顺序、异常、类型），
  且**成本严格下降**才被应用 ⇒ fixed-point 天然收敛，不会振荡。
- 保守策略：语义未知（`Effect::Unknown`）一律不改写。
- 通用规则写在 engine（只依赖 `Lang` 抽象），语言特有规则写在各自 crate。
- 第一阶段范围：语句/表达式树层的 canonicalization；CFG 级 pass 与 DCE 不在本库。

## 容错（fault tolerance）

解析器**永不失败**：哪里坏跳哪里——

- 坏语句 → `Raw` 节点**原文保真**；
- 坏方法 → `Member::Raw`；坏顶层 → `unit.raws`；
- 其余部分照常解析、**照常参与简化与格式化**；
- 所有词法/语法错误带 1-based 行列号返回，可 `--strict` 升级为非零退出码。

## CLI

```text
cure [选项] <文件.java>... | -
  -w/--write          原地覆写
  -o/--output <文件>  输出路径（默认 stdout）
  --format-only       仅格式化
  --check             干跑：有可优化改写 → 退出码 2
  --report            stderr 打印逐规则统计
  --disable <规则>    禁用指定规则
  --strict            有解析错误 → 退出码 1
```

```bash
echo 'class A{int m(){int a=foo();int b=a;return b;}}' | cure -
# → class A {
#      int m() {
#          return foo();
#      }
#  }
```

## 语义验证体系（三层）

| 层 | 手段 | 验证什么 |
|---|---|---|
| 1. 属性测试 | 随机生成 toy 程序（80 个种子），解释器对拍**简化前后的返回值 + 调用副作用序列** | 引擎层语义保持（求值顺序/短路/副作用）+ fixed-point 幂等 |
| 2. 差分测试 | 原版与简化版各自 `javac` 编译、`java` 运行，**stdout/退出码/异常签名必须一致**（12 组：溢出、NaN、除零异常、StringBuilder、迭代器、装箱、短路副作用…） | Java 全链路语义保持 |
| 3. 语料库 | google-java-format 84 个真实文件跑全链路 | 鲁棒性（不 panic）、自洽性（输出可重新干净解析）、幂等性（第二轮 0 改写）+ 能力统计 |

`cargo test`：90 个测试。属性与差分测试在本轮开发中**实际抓到 3 个语义/解析 bug**（`x && true` 误折叠、`((Cast) x).method()` 后缀丢失、`new` 被当类型名）。

## 规则清单

引擎通用（19）：paren_removal、const_condition、boolean_return、if_else_empty、
bool_compare、double_not、bool_not_fold、bool_short_circuit、not_compare、
ternary_fold、ternary_bool、const_fold_bin（Java 回绕语义）、self_assign、
arith_identity、arith_zero、local_propagation、dead_store + 选配 unreachable_after_terminal。

Java 专属（6）：cast_simplify、self_compare、string_builder_fold（`new SB().append…` → `+` 拼接）、
box_unbox_chain（`Integer.valueOf(n).intValue()` → n）、iterator_to_for_each（迭代器模式 → for-each）。

## 参考

MLIR Canonicalization / LLVM InstCombine（规则组织）、Vineflower（反编译器自然化规则）、
egg / ast-grep / tree-sitter / google-java-format（本地 `/Users/e/Documents/github/` 下有源码可研读）。
