# cure

**[English](README.md) | [简体中文](README.zh-CN.md)**

[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust MSRV](https://img.shields.io/badge/rust-1.74%2B-orange)

**cure** 是容错式、语义保持的代码简化 / 格式化引擎，把反编译出来的代码
还原成「像人手写的」样子。引擎与 CLI 均语言无关——**当前实现的语言
后端：Java、TypeScript/JavaScript**（ts/tsx/mts/cts/js/jsx/mjs/cjs）与
**Python**（py/pyi，Python 2.7 与 3.x，覆盖至 3.15）；下文示例即 Java
后端的实际体验。TS/JS 后端同引擎、按 ECMAScript 语义：f64/ToInt32 折叠域、
±2^53 精确性、按文件的绑定/作用域解析（var 提升与闭包捕获守卫）、
可选链/解构不透明、ECMA 数字格式化。Python 后端同时接受两套语法——Python 2
的 print/exec/raise 语句、反引号、`<>`、long 与旧式八进制字面量、ur 字符串、
元组形参、`except E, e`，以及 PEP 701 f-string/t-string、match、海象、
except*、PEP 695 类型参数、lazy import——并按 Python 真值、整除/幂语义以及
生成器/作用域约束保持语义。Python 2 的 int/long 区分与 True/False 重绑定
也保持健全；两个版本语义不同的写法（print、exec）在歧义时逐字节保留。

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

- **容错设计** —— 语法错误不中断运行：错误区域原文保留，其余照常简化；
  超过递归上限的深嵌套输入整文件原文保真（防栈溢出）
  （`--strict` 可切换为 CI 用错误退出码）。
- **三层语义验证** —— 属性测试（对拍玩具语言解释器）+ 差分测试（对拍
  真实 `javac`/`java` 运行）+ 语料库重解析/幂等检查。这三层在开发中
  实际抓到过真实 bug，不是摆设。
- **简化效果指标**（`--stats`）：AST 节点数、判定点（圈复杂度）、最大
  嵌套的前后对比 + 文件分类（结构简化 / 仅格式 / 未变）——三项不受
  格式化影响，能区分「真简化」与「纯格式归一」，零简化文件直接可见。
- **可选源码保真** —— `--preserve-function-source` 把每个函数、类、对象
  字面量逐字节原样输出（保留注释、空白与 CRLF），`Function.prototype
  .toString()` 与原文一致；从保真区引用的绑定被冻结，模块其余部分仍正常
  简化。
- **复杂 TypeScript 类型** —— 泛型/映射/条件/元组/模板字面量类型别名，
  `interface`/`namespace`/ambient 声明，明确赋值 `x!: T`，可选成员，
  泛型调用 `f<T>()` 与 `new C<T>()`，带返回类型（含对象类型）与剩余
  参数的箭头/函数表达式，以及括号化 `as`/`satisfies` 类型。类型声明
  原文保真，值级代码仍正常简化。
- **最新 TS/JS 语法** —— 类/成员装饰器、`accessor`、私有字段/方法、
  static 块、`using`/`await using`、导入属性/`defer`/`source`、顶层
  await、泛型函数/箭头（含 `const` 类型参数）、`satisfies`/`as const`、
  `#x in o`、RegExp `v` 标志与可选链；未建模结构原文保真，不中断。
- **Python 后端（2.7 + 3.15）** —— `.py`/`.pyi` 全流程解析 → 简化 → 打印，
  容错缩进解析器同时接受两套语法：Python 2 的 print/exec/raise 语句、反引号、
  `<>`、`1L`、`0777`、`ur"..."`、元组形参、`except E, e`、无括号推导式元组，
  与 PEP 701、t-string、match、海象、except*、PEP 695、lazy import 共存。
  常量折叠遵循 Python 语义（i64 精确域整数、向下取整除法/取模、`and`/`or`
  返回操作数、真值）；Python 2 int/long 类型与 True/False 重绑定保持健全，
  版本相关写法在歧义时逐字节保留；含 `yield`/`global`/`nonlocal` 的死分支
  一律不删。
- **64+ 条简化规则**（引擎通用 + Java/TS/Python 规则包），含控制流扁平化
  还原、语句级 StringBuilder 链还原、XOR 噪声剥除、纯字面量 JDK 调用
  的部分求值（"虚拟执行"）。
- **真实代码库规模** —— OpenJDK 全源码语料（371,674 文件 / 4.7 GB）
  在 18 核笔记本上约 15 秒处理完，0 解析失败、0 panic。TS/JS 侧：
  13 MB 压缩 bundle 约 5 秒（增量 effect 缓存前约 32 秒），4,358 个
  npm 文件约 10.5 秒；Token 位置字段改为 u32，每个 token 48 字节。
- **跨平台 I/O 后端** —— Linux 可启用批量 `io_uring`（3,483 文件
  syscall 追踪：read 调用 6,977 → 10，syscall 时长 −53%）；macOS /
  Windows 用调优线程池。默认构建零运行时依赖。
- **零运行时依赖** —— 仅 `std`（一个可选的 feature 门控 Linux 专属
  crate）。

## 安装

```bash
# 从 crates.io
cargo install cure-cli

# 或经 cargo-binstall 安装预编译二进制
cargo binstall cure-cli

# 或从 GitHub Releases 直接下载裸二进制（无压缩包）：
# 资产命名为 cure-vX.Y.Z-<target>
# https://github.com/ejfkdev/cure/releases

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

```bash
echo 'const x = 1 + 2; console.log(x);' | cure --lang ts -
# const x = 3;
# console.log(3);
```

```bash
printf 'x = 1 + 2\nprint(x)\n' | cure --lang py -
# x = 3
# print(3)
```

```text
cure [选项] <文件>... | <目录> | -
```

| 选项 | 含义 |
|------|------|
| `-o, --output <路径>` | 输出路径（单文件）或输出根目录（目录模式） |
| `-w, --write` | 原地覆写输入文件 |
| `--check` | 干跑：有可优化改写 → 退出码 2，不写出任何内容 |
| `-j, --threads <N>` | 并行工作线程数（默认=CPU 核心数 × 1.5） |
| `--format-only` | 仅格式化，不做简化 |
| `--ext <后缀列表>` | 只处理指定后缀（逗号或空格分隔） |
| `--lang <名称>` | 强制后端：`java`、`ts`/`js`（TS/JS）或 `python`/`py`；默认按文件后缀推断 |
| `--copy-other` | 目录模式：不受处理的文件原样拷入输出树 |
| `--diff` | 打印每个被改写文件的统一 diff（Myers 算法） |
| `--stats` / `--report` | 汇总统计 / 逐文件改写统计 |
| `--disable <规则名>` | 禁用指定规则（可多次） |
| `--dead-code` | 选配：删除全单元零引用的 private 方法（反射场景慎用） |
| `--preserve-function-source` | TS/JS：函数/类/对象字面量逐字节原样输出（兼容 `Function.prototype.toString`），这些区域内部不做简化 |
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
| 4. 反编译 APK | 5 个真实 Android APK 经 [ddc](https://github.com/ejfkdev/ddc) 反编译共 144,592 文件，cure 完整 + `--dead-code` 双模式 | 同上三不变量 + javac 差分：ddc 原输出可编译的包在 cure 后必须仍可编译 |
| 5. ECMAScript 规范符合性 | [Test262](https://github.com/tc39/test262) 六层：(a) 同步用例原版与 cure 版各在全新 `node` vm 上下文执行，比较异常状态 + stdout（34,007 个可执行用例）；(b) **async 用例**执行并排空 `$DONE`/微任务/定时器（5,423 个执行）；(c) **module 用例**按 ESM 链接（相对 fixture 文件系统加载）并求值（358 个执行）；(d) **全部 53,910 个文件做编译差分**（含 async/module/负向）；(e) **打印往返**：解析 → 打印（不简化）→ 再解析，忽略 `Paren`/字面量拼写/`Group` 展开后 AST 结构等价（49,159 个检查）；(f) **简化幂等**：同一语料跑两遍 cure，输出逐字节一致（49,159 个检查） | ECMAScript 语义、打印器/括号/装饰器漂移、定点收敛；外加合法文件 cure 后可编译、输出幂等 |
| 6. 真实 JS/TS 语料 | 全局 npm 安装中的 4,358 个 `.js/.mjs/.cjs`（约 568 MB）与 1,334 个 `.ts/.tsx/.mts/.cts`（undici、zod、minisearch、pnpm、yarn、vite-plus、Lark CLI、bytenode…）：合法文件在 cure 后必须仍可编译；第二轮 cure 输出必须逐字节一致。两种语料均 **0 解析诊断 / 0 二轮差异**（默认与保真模式），cure 产物经 Oxc（`oxfmt`）解析 **0 语法错误** | 真实包语法（压缩 bundle、顶层 await、导入属性）、复杂类型语法与模块常量传播下的幂等性 |
| 7. 随机 JS 差分 | 确定性 PRNG 生成合法程序（声明/遮蔽/`var` 提升/闭包/循环/三元/短路/模板混合）；原版与 cure 版在 `node` 下比较 stdout + 退出码，并要求二次 cure 逐字节一致（`CURE_FUZZ_CASES` 控制样本数） | 规则组合语义与首轮/二轮收敛；开发中跑过 5,000 例 |
| 8. Python（2.7 + 3.13–3.15） | CPython 2.7 标准库（1,609 个 UTF-8 源文件）、4,135 文件 Python 2 包语料（Django 1.11、Twisted 20.3、pip 20.3、setuptools 44、requests、six、PyYAML）、CPython 3.13/3.14/3.15 标准库（2,188 / 2,288 / 2,456 源文件）、18,128 个 Python 3.14 site-packages 文件与 3.13/3.15 site 的 1,083 / 422 个文件：解析 → 打印后用对应解释器的 `ast.dump` 比较；二次 cure 必须逐字节一致；每个简化产物交给同一解释器编译；精选程序实际执行 | 所有语料 **0** 解析诊断；**32,289** 次 AST 等价往返（Python 2 5,736 + Python 3 26,553；已排除 20 个故意非法/异构夹具）；**35,548/35,548** 幂等、0 漂移；**32,274** 次编译检查 **0 破坏**（35 个原文件本就不可编译）；精选程序 stdout/退出码一致 |

`cargo test` 共 519 个测试。默认模式下 Test262 语义差分
**33,963/34,007（99.87%）执行结果逐字节一致**；仅 44 个用例被排除——它们比较
`Function.prototype.toString` 的**源码文本**（cure 会重排函数体格式），
或用箭头函数作计算属性名（键依赖同一文本）。使用
`--preserve-function-source` 时同一层 **34,007/34,007（100%）** 全一致：
函数/类/对象字面量按源字节输出，这 44 个用例不再有差异。async 层 **5,423/5,423
个用例执行结果一致（0 失配）**、module 层 **358/358（0 失配）**、编译层
**49,165/49,165 个合法文件在 cure 后仍可编译（0 破坏**）。打印往返与
简化幂等两层各 **49,159/49,159** 通过（按 Test262 的 script/module 目标
分别解析）。六层均有抽样常驻测试（`cargo test -p cure-ts-simplify --test
t262_sem|t262_async|t262_module|t262_compile|t262_roundtrip|t262_idem`）；语料或 `node` 缺失时
自动跳过（`T262_ROOT` / `T262_SAMPLE` 可覆盖）。第 1、2 层在本轮开发中**实际抓到 3 个
语义/解析 bug**（`x && true` 误折叠、`((Cast) x).method()` 后缀丢失、
`new` 被当类型名）。

Python 差分覆盖 CPython 2.7 与 3.13–3.15。2.7 标准库（1,609 个 UTF-8
源文件）与 4,135 文件的 Python 2 包语料（Django 1.11、Twisted 20.3、pip、
setuptools 44、requests、six、PyYAML）均为 **0 解析诊断**；1,607 + 4,129
个解析 → 打印往返在 CPython 2.7 `ast.dump` 下结构等价（其余为故意非法/
异构夹具），5,744 个文件全部幂等，原本可被 CPython 2.7 编译的文件
（1,600 + 4,129）在简化后仍全部可编译（另有 15 个原文件本就非法）。
3.13/3.14/3.15 标准库与 3.14 site-packages 再贡献 26,553 个 AST 等价
往返，0 诊断、0 幂等漂移、0 编译破坏；3.6 与 3.8 标准库（1,578 / 1,661
个文件）同样 0 诊断且幂等。全部语料合计 **32,289 次 AST 比较、35,548 个
幂等文件、32,274 次编译，0 破坏**；简化器共执行 **133,243 次改写**
（Python 2 标准库 5,501、Py2 包 4,864、Python 3.15 标准库 7,700、
Python 3.14 site-packages 97,951），docstring 与版本可见类型保持原样。另有 Python 2/3 精选程序（print/exec/
raise 形态、反引号、`<>`、long/int 类型、元组形参、带三引号参数的装饰器）
在对应解释器下与 cure 版对拍 stdout 与退出码，结果一致。CLI 目前只读取
UTF-8 源码；其他 PEP 263 编码文件会跳过（上面的 Python 2 标准库语料已
转码为 UTF-8 后测试）。

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

**引擎通用（34）**：`paren_removal`、`const_condition`、`boolean_return`、
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
`inverse_assign_pair`（`x+=K;x-=K` 寄存器噪声对抵消）、`assign_back_fold`（`x=e; x=x+K`
寄存器累加折回声明）、`dead_decl`（零使用死声明删除）、`dead_store`、`store_kill`、double_neg_fold（`-(-lit)` 折叠）、block_flatten（无谓嵌套块塌平）、选配 `unreachable_after_terminal`。

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
`twr_recover` + `string_switch_recover`（反编译形态还原）、
`try_unwrap_rethrow`（`catch (E e) { throw e; }` 纯重抛剥除——fernflower
最高频残迹，66 文件语料 ≥23 处）、new_string_char_array_fold（`new String(CHAR_ARRAY)`）、empty_finally_strip、try_unwrap_no_catch、static_array_index_fold（字面量数组下标）、const_method_inline（单 return helper 内联——解密器）、trailing_continue（标签感知尾 continue 删除）、

**Python 专属（12）**：`py_const_fold`（整数/布尔/字符串字面量、字符串
`+`/`*`、比较、真值、`and`/`or` 返回操作数；long/浮点/复杂字面量不折）、
`py_call_fold`（字面量 str 的 upper/lower/swapcase/capitalize/strip/
replace/startswith/endswith/join，以及防遮蔽的 len/ord/chr/str/int/abs/
bool/min/max）、`py_subscript_fold`（字面量字符串/列表/元组的下标与切片）、
`py_const_if`、`py_if_ternary`（单赋值/单 return 分支合成条件表达式）、
`py_nested_if`（`if a: if b:` → `if a and b:`）、
`py_else_after_terminal`（if 体必然终止时去掉 else）、`py_dead_store`
（相邻纯字面量覆盖）、`py_dead_expr`（裸数字/None/... 表达式）、
`py_return_none`（`return None` → `return`）、`py_unreachable`、
`py_pass_cleanup`。docstring、Python 2 int/long、被遮蔽的 True/False/
内建（含 exec 与 import *）以及版本歧义写法都有守卫。

## 性能

release 构建、18 核 Apple Silicon 笔记本、页缓存热、默认线程数（核心数 × 1.5）：

| 工作负载 | 文件数 | 耗时 | 峰值内存 |
|---|---|---|---|
| 单文件（stdin → stdout） | 1 | <10 ms | ~2 MiB |
| 入库测试语料 | 3,999 | 0.6 s | ~110 MiB |
| CPython 2.7 标准库，写出输出树 | 1,609 | 0.16 s | ~90 MiB |
| Python 2 包语料，写出输出树 | 4,135 | 0.40 s | ~90 MiB |
| CPython 3.15 标准库，写出输出树 | 2,456 | 0.3 s | ~150 MiB |
| Python 3.14 site-packages，写出输出树 | 18,128 | 1.8 s | ~370 MiB |
| OpenJDK 全源码语料，`--check` | 371,674 / 4.70 GB | 14.8 s | ~430 MiB |
| OpenJDK 全源码语料，写出输出树 | 371,674 / 4.70 GB | 27.5 s | ~430 MiB |

37 万文件两档均执行完整 parse → simplify → print，**0 解析失败、0 panic**。
3,999 文件入库语料的 `--stats`：结构简化 1032 / 仅格式 1943 / 未变 46
（行数 −53.4%、节点 −10.3%、判定点 −13.9%）。

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
cure-python-ast    Python NodeData + PyId arena
cure-python-parser 容错缩进词法/解析器（Python 2.7–3.15）
cure-python-print  规范化 4 空格 Python 打印器
cure-python-simplify Python 常量/控制流规则包（基于 cure-engine）
cure-cli           `cure` 二进制：并行管线、跨平台批量 I/O（io_uring/线程池）、目录模式
```

`Lang` trait 是语言边界。数值/字符串折叠语义全部走语言钩子
（`fold_lit_bin` / `fold_lit_cmp` / `fold_lit_neg` / `reassoc_delta`）——
引擎不硬编码任何语言的算术。每个语言前端在 `cure-engine` + `cure-tree`
之上提供自己的 AST + parser + printer + 规则包；TS/JS 与 Python 后端就是
两个这样的前端（另见 `cure-tree/tests/kit.rs` 里 ~100 行的玩具语言）。

## 状态与路线

- ✅ Java 后端达到生产级健壮性：37 万文件语料 0 失败
- ✅ TS/JS 后端：Test262 六层全绿，npm 语料 0 解析诊断
- ✅ Python 2.7 + 3.x 后端：35,548 个标准库/site-packages 文件，0 诊断、
  0 幂等漂移、0 编译破坏
- ✅ 12 套入库 Java 语料、三层验证、519 个测试
- 🔜 还原规则 V2（反射 addSuppressed 形态、嵌套双资源 try-with-resources）
- 🔜 Python 规则包扩展（局部传播、docstring/空白规范化、反编译形态还原）

## 许可证

MIT —— 见 [LICENSE](LICENSE)。版本历史见 [CHANGELOG.md](CHANGELOG.md)。
