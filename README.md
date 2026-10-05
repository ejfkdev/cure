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

引擎通用（32）：paren_removal、const_condition、boolean_return、if_to_ternary、
if_assign_ternary、if_else_empty、bool_compare、double_not、bool_not_fold、
bool_short_circuit、not_compare、ternary_fold、ternary_bool、const_fold_bin、
**cmp_const_fold**（`1 < 2 → true`，击穿不透明谓词）、self_assign、
arith_identity、**arith_zero**、**bit_identity**、**arith_reassoc**（双异或/
加减重结合/字符串拼接常量合并 `("a"+x)+"b"+"c" → "a"+x+"bc"`）、
**ternary_bool_op**（`c ? a : false → c && a`）、
local_propagation、**decl_assign_merge**、**assign_propagation**（拷贝赋值内联，
块内声明锚点防逃逸；使用语句写排除：`target = use` 的 RHS 先于写求值）、
**store_kill**（远距死存储/寄存器预声明清理：首个事件在**支配路径**上是写时
击杀——字面量提升进 init / 剥 init / 删语句；条件分支、循环体、try 内的事件
不算必经，副作用 init 永不丢），
**multi_use_copy**（多用途拷贝传播：`x = y; …N 处读 x` → 全部替换为 y）、
**trailing_return**（void 方法尾部裸 `return;` 删除）、
**trailing_continue**（标签感知：循环体尾部 continue，标签指向本循环才删）、
dead_store + 选配 unreachable_after_terminal。

Java 专属（15）：cast_simplify、self_compare、string_builder_fold、box_unbox_chain、
iterator_to_for_each、while_iterator_to_for_each（支持 Cast/Paren 包裹的 next()）、
new_string_fold、loop_head_break、concat_value_of_drop、
**string_builder_statements**（语句级 SB 链还原：重赋值+新变量混合形态
`sb = sb.append(x); sb2 = sb.append(y); s = sb2.toString()` → `s = …拼接…`）、
**xor_noise**（Java 语义下 XOR 操作数必为整型 → 含副作用调用也能剥
`(mark(5) ^ 0x5A) ^ 0x5A → mark(5)`）、
**str_len_fold**（`"abc".length() → 3`）、
**literal_eval**（部分求值器/"虚拟执行"：纯 JDK 方法 + 全字面量实参 →
编译期求值——`"HelloWorld".substring(0,5)`、`String.format("%s=%d",…)`、
`Integer.parseInt("42")`、`Math.abs/max/min`、`Character.isXxx/toXxx/toString`、
字面量数组下标 `{"a","b"}[1]`、cast 字面量 `(char)('a'+2) → 'c'`；
**只在求值成功时折叠**——解析失败/越界/异常路径保持原样；
不折 toUpperCase/toLowerCase（locale 敏感））、
**base64_new_string_fold**（`new String(Base64.getDecoder().decode("aGVsbG8=")) → "hello"`，
标准/URL 字母表，UTF-8 合法时折叠）、
**cff_recover**（**控制流扁平化还原**：`while(true){switch(s)}` 状态机 →
结构化控制流——线性链顺序拼接、菱形找公共后继 if/else{前缀}+单次续接、
分支回环 → while(cond){体}、链内后向边 → while(true){后缀}；
支持哨兵出口与 default: return 两种出口形态；嵌套条件/发散/外部跳转保守拒绝）。
引擎侧 char 参与算术按 Java 语义提升为 int 折叠；Str+Char/Int/Long/Bool
字面量拼接直接折成字符串（浮点除外——Double.toString 算法不保证逐位一致）。
引擎侧 char 参与算术按 Java 语义提升为 int 折叠；Str+Char/Int/Long/Bool
字面量拼接直接折成字符串（浮点除外——Double.toString 算法不保证逐位一致）。

## 虚拟执行验证（tests/eval_obfuscation.rs）

混淆器把常量藏进方法调用（substring/format/parseInt/Base64/字符算术/
字面量表/字符串杂项/Math/Character）——40 次改写，非空行 44 → 20（-55%），
javac 差分逐字节一致。模式来源：obfuscator.io / javascript-obfuscator
的 string-array 与字符串编码家族的 Java 等价形态（跨方法解密器与
控制流扁平化需要过程间分析，暂不覆盖）。

## 控制流扁平化还原验证（tests/cff_obfuscation.rs）

obfuscator.io / Allatori 风格的状态机混淆——线性链（32 → 12 行）、菱形分叉
（还原出 if/else 后继续被接力折叠成内联三元）、循环形态（`case 0: if(i<5)...`
回环 → `while (i < 5) {...}`）、default: return 出口，全部 javac 差分逐字节一致；
安全负例（状态变量循环后被使用 / 存在不可达 case）正确拒绝还原。

## 刁钻混淆验证（tests/hard_obfuscation.rs）

多层常量隐藏（声明链×异或对×位噪声×死赋值）、嵌套不透明谓词、循环混合断路+
寄存器回拷、多层字符串混淆（SB×new String×valueOf×分散常量×length）、布尔旗标
三元嵌套、迭代器+SB 语句链+continue 组合、副作用异或包裹——**44 次改写，
非空行 76 → 33（-57%）**，javac 差分逐字节一致，副作用调用序列完整保留。

另有**双层数链路**（ddc_tools.rs 第二测）：刁钻混淆源 → javac（常量层被
编译器折叠、结构性混淆存活）→ d8 → ddc（叠加寄存器伪影）→ cure：
15 次改写，输出与 ddc/原始双重一致。

## 去混淆验证

- `tests/deobfuscate.rs`：模拟混淆器（不透明谓词/双异或/位噪声/布尔包装/
  StringBuilder/装箱链/迭代器/死赋值/拆分声明/拷贝链/valueOf 包装/常量分散拼接）
  → **29 次改写，非空行 -48%**，javac 差分逐字节一致，副作用调用序列保留；
- `tests/real_tools.rs`：javac → **ProGuard** → **jadx** → cure → 编译运行，
  输出与原始完全一致（jadx 去混淆较强，残留产物需要循环级数据流）；
- `tests/ddc_tools.rs`：javac → d8 → **ddc** → cure → 编译运行，
  **81 → 46 行（-44%）**，输出与 ddc 行为逐字节一致（ddc 产物是主目标素材：
  寄存器拷贝/语句级 SB 链/while(true)+break 全部被还原）；
- `tests/asc_tools.rs`：javac → d8 → zip APK → **ASC（androguard DAD）** → cure，
  **65 → 37 行（-43%）**。DAD 去混淆最弱（类型推断错误、`class LDemo;` 描述符
  泄漏、尾部 `return;`）——其输出本身无法通过 javac，本测试验证容错解析 0 错误
  + 净化显著 + 输出自洽 + 幂等；DAD 自身的类型错误被原样保留（不发明类型）。

## 参考

MLIR Canonicalization / LLVM InstCombine（规则组织）、Vineflower（反编译器自然化规则）、
egg / ast-grep / tree-sitter / google-java-format（本地 `/Users/e/Documents/github/` 下有源码可研读）。
