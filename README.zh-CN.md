# cure

**[English](README.md) | [简体中文](README.zh-CN.md)**

[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust MSRV](https://img.shields.io/badge/rust-1.74%2B-orange)

**cure** 是容错式、语义保持的代码简化 / 格式化引擎，把反编译出来的 Java
还原成「像人手写的」样子。

```java
// 反编译输入                                 // cure 处理后
class B {                                     class B {
    void n() {                                    void n() {
        int x = 1 + 2;                                System.out.println("y1");
        boolean c = (x > 2) && true;              }
        if (c) { System.out.println("y"+1); }  }
        return;
    }
}
```

它**不是**死代码消除（DCE），**不是**压缩器：只做局部表达式与控制流
**形态**的规范化——折叠常量与不透明谓词、内联拷贝、消解编译器产物
（寄存器噪声、StringBuilder 链、控制流扁平化状态机），并以干净的惯用
格式输出——同时保证程序含义不变。

## 特性

- **容错设计** —— 语法错误不中断运行：错误区域原文保留，其余照常简化
  （`--strict` 可切换为 CI 用错误退出码）。
- **三层语义验证** —— 属性测试（对拍玩具语言解释器）+ 差分测试（对拍
  真实 `javac`/`java` 运行）+ 语料库重解析/幂等检查。这三层在开发中
  实际抓到过真实 bug，不是摆设。
- **54 条简化规则**（引擎通用 31 + Java 专属 23），含控制流扁平化
  还原、语句级 StringBuilder 链还原、XOR 噪声剥除、纯字面量 JDK 调用
  的部分求值（"虚拟执行"）。
- **真实代码库规模** —— OpenJDK 全源码语料（371,674 文件 / 4.7 GB）
  在 18 核笔记本上约 15 秒处理完，0 解析失败、0 panic。
- **跨平台 I/O 后端** —— Linux 可启用批量 `io_uring`（3,483 文件
  syscall 追踪：read 调用 6,977 → 10，syscall 时长 −53%）；macOS /
  Windows 用调优线程池。默认构建零运行时依赖。
- **零运行时依赖** —— 仅 `std`（一个可选的 feature 门控 Linux 专属
  crate）。

## 安装

```bash
# 从 crates.io
cargo install cure-cli

# 或从 GitHub Releases 下载预编译二进制
curl -L https://github.com/ejfkdev/cure/releases/latest/download/cure-<target>.tar.gz | tar xz

# 或从源码
git clone https://github.com/ejfkdev/cure
cargo install --path crates/cure-cli
```

## 用法

```bash
echo 'class A{int m(){int a=foo();int b=a;return b;}}' | cure -
```
```java
class A {
    int m() {
        return foo();
    }
}
```

```text
cure [选项] <文件.java>... | <目录> | -
```

| 选项 | 含义 |
|------|------|
| `-o, --output <路径>` | 输出路径（单文件）或输出根目录（目录模式） |
| `-w, --write` | 原地覆写输入文件 |
| `--check` | 干跑：有可优化改写 → 退出码 2，不写出任何内容 |
| `-j, --threads <N>` | 并行工作线程数（默认=CPU 核心数 × 1.5） |
| `--format-only` | 仅格式化，不做简化 |
| `--ext <后缀列表>` | 只处理指定后缀（逗号或空格分隔） |
| `--copy-other` | 目录模式：不受处理的文件原样拷入输出树 |
| `--diff` | 打印每个被改写文件的统一 diff（Myers 算法） |
| `--stats` / `--report` | 汇总统计 / 逐文件改写统计 |
| `--disable <规则名>` | 禁用指定规则（可多次） |
| `--dead-code` | 选配：删除全单元零引用的 private 方法（反射场景慎用） |
| `--strict` | 有解析错误 → 退出码 1 |

目录模式递归处理、多核并行（动态取号队列，天然均衡大文件聚集），
默认镜像到同级 `<目录名>-cure-out/`。

无参数运行 `cure` 直接打印帮助；`cure rules` 列出全部规则名
（`--disable` 的取值）；帮助双语——`CURE_LANG=zh|en` 强制指定，
否则按 locale 自动检测。完整选项见 `cure --help`。

## 语义验证体系（三层）

| 层 | 手段 | 验证什么 |
|---|---|---|
| 1. 属性测试 | 随机生成 toy 程序（80 个种子），解释器对拍**简化前后的返回值 + 调用副作用序列** | 引擎层语义保持（求值顺序/短路/副作用）+ fixed-point 幂等 |
| 2. 差分测试 | 原版与简化版各自 `javac` 编译、`java` 运行，**stdout/退出码/异常签名必须一致**（12 组：溢出、NaN、除零异常、StringBuilder、迭代器、装箱、短路副作用…） | Java 全链路语义保持 |
| 3. 语料库 | 3,999 个入库真实文件（12 套自动发现）+ 5,596 文件外部 mega 语料 + 37 万文件 OpenJDK 源码语料 | 鲁棒性（不 panic）、自洽性（输出可重新干净解析）、幂等性（第二轮 0 改写） |

`cargo test` 共 170 个测试。第 1、2 层在本轮开发中**实际抓到 3 个
语义/解析 bug**（`x && true` 误折叠、`((Cast) x).method()` 后缀丢失、
`new` 被当类型名）。

### 对抗性案例

以下全部通过 javac 差分逐字节一致 + 副作用调用序列完整保留：

- **终极组合混淆** —— 跨方法字符串解密器（string-array + Base64
  helper）× CFF 状态机 × 寄存器噪声 × 不透明谓词 × SB 语句链：
  24 次改写，非空行 87 → 32（−64%），`d(0)+d(1)+"!"` → `println("super!")`。
- **控制流扁平化还原**（`cff_recover`）—— obfuscator.io / Allatori
  风格 `while(true){switch(s)}` 状态机 → 结构化控制流：线性链、菱形
  分叉、循环回环、`default: return` 出口；不安全形态（状态变量循环后
  被使用 / 不可达 case）保守拒绝。
- **刁钻混淆** —— 多层常量隐藏、嵌套不透明谓词、SB × new String ×
  valueOf × 分散常量：44 次改写，非空行 76 → 33（−57%）。
- **双工具链** —— 混淆源 → javac → d8 → ddc（寄存器伪影）→ cure：
  输出与 ddc / 原始双重一致。

## 规则清单

**引擎通用（31）**：`paren_removal`、`const_condition`、`boolean_return`、
`if_to_ternary`、`if_assign_ternary`、`if_else_empty`、`bool_compare`、
`double_not`、`bool_not_fold`、`bool_short_circuit`、`not_compare`、
`ternary_fold`、`ternary_bool`、`ternary_bool_op`、`const_fold_bin`
（Java i32/i64 环绕 + 移位掩码）、`cmp_const_fold`（`1 < 2 → true`，
击穿不透明谓词）、`self_assign`、`arith_identity`、`arith_zero`、
`bit_identity`、`arith_reassoc`（双异或/加减重结合/字符串常量合并
`("a"+x)+"b"+"c" → "a"+x+"bc"`）、`local_propagation`、
`decl_assign_merge`、`assign_propagation`（拷贝赋值内联，块内声明锚点
防逃逸）、`store_kill`（支配路径击杀远距死存储/寄存器预声明）、
`multi_use_copy`、`trailing_return`、`trailing_continue`（标签感知）、
`dead_store`、`store_kill`、double_neg_fold（`-(-lit)` 折叠）、block_flatten（无谓嵌套块塌平）、选配 `unreachable_after_terminal`。

**Java 专属（23）**：`cast_simplify`、`self_compare`、
`string_builder_fold`、`box_unbox_chain`、`iterator_to_for_each`、
`while_iterator_to_for_each`、`new_string_fold`、`loop_head_break`
（含 do-while 形态）、`concat_value_of_drop`、
`string_builder_statements`（语句级 SB 链还原：重赋值 + 新变量混合
形态）、`xor_noise`（Java 语义下 XOR 操作数必整型，副作用调用也能剥
`(mark(5) ^ 0x5A) ^ 0x5A → mark(5)`）、`str_len_fold`、`literal_eval`
（部分求值器：`"HelloWorld".substring(0,5)`、`String.format`、
`Integer.parseInt`、`Math.abs/max/min`、`Character.isXxx/toXxx`、字面量
数组下标、cast 字面量——**只在求值成功时折叠**，异常路径保持原样）、
`base64_new_string_fold`、`cff_recover`（控制流扁平化还原）、
`twr_recover` + `string_switch_recover`（反编译形态还原）、new_string_char_array_fold（`new String(CHAR_ARRAY)`）、empty_finally_strip、try_unwrap_no_catch、static_array_index_fold（字面量数组下标）、const_method_inline（单 return helper 内联——解密器）、trailing_continue（标签感知尾 continue 删除）、

## 性能

release 构建、18 核 Apple Silicon 笔记本、页缓存热、默认线程数（核心数 × 1.5）：

| 工作负载 | 文件数 | 耗时 | 峰值内存 |
|---|---|---|---|
| 单文件（stdin → stdout） | 1 | <10 ms | ~2 MiB |
| 入库测试语料 | 3,999 | 0.6 s | ~110 MiB |
| OpenJDK 全源码语料，`--check` | 371,674 / 4.70 GB | 14.8 s | ~430 MiB |
| OpenJDK 全源码语料，写出输出树 | 371,674 / 4.70 GB | 27.5 s | ~430 MiB |

37 万文件两档均执行完整 parse → simplify → print，**0 解析失败、0 panic**。

- 并行度：动态取号队列，默认线程 = 核心数 × 1.5（P/E 异构实测比 1:1 快 7%；CPU 与内存持平）。
- Linux 批量 I/O（`--features io-uring`）：三阶段批量 open→read→close；3,483 文件 read 6,977 → 10 次，syscall 时长 −53%。
- macOS/Windows I/O：线程池逐文件读，实测聚合 ~1 GB/s（接近逐文件 syscall 上限）。

## 架构

```text
cure-engine        语言无关核心：模式、改写、pass、fixed-point 驱动、代价模型、
                   字面量折叠语义钩子
cure-tree          通用 arena 树工具包（每个语言复用）：子节点内联容器、
                   名字 intern + 节点键、区域事件索引（B3 惰性重建）、效果表重建
cure-java-ast      Java NodeData + 签名层；基于 cure-tree 实现 Lang trait
cure-java-parser   容错 Java 词法/解析器（token 全借用、源零拷贝）
cure-java-print    规范化 Java 打印器/格式化器
cure-java-simplify Java 规则包 + cure-engine 门面
cure-cli           `cure` 二进制：并行管线、跨平台批量 I/O（io_uring/线程池）、目录模式
```

`Lang` trait 是语言边界。数值/字符串折叠语义全部走语言钩子
（`fold_lit_bin` / `fold_lit_cmp` / `fold_lit_neg` / `reassoc_delta`）——
引擎不硬编码任何语言的算术。接入第二语言只需在 `cure-engine` +
`cure-tree` 之上提供自己的 AST + parser + printer + 规则包
（见 `cure-tree/tests/kit.rs` 里 ~100 行的玩具语言）。

## 状态与路线

- ✅ Java 后端达到生产级健壮性：37 万文件语料 0 失败
- ✅ 12 套入库语料、三层验证、170 个测试
- 🔜 还原规则 V2（反射 addSuppressed 形态、嵌套双资源 try-with-resources）
- 🔜 第二语言接入，验证 Lang trait 边界的泛化性

## 许可证

MIT —— 见 [LICENSE](LICENSE)。版本历史见 [CHANGELOG.md](CHANGELOG.md)。
