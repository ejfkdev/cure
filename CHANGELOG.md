# Changelog

本项目所有显著变更记录于此。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 修复（Test262 官方一致性测试集驱动的解析器加固——2026-10-23）
- **OOM 级 bug（曾致系统崩溃）**：标识符转义扫描的两个字符白名单
  （转义分支无 ZWJ/ZWNJ、字节分支有）不一致——`\u200C` 在转义分支
  break 且零字节消费，入口守卫却接受 `\`+`u` → 空 ident token 无限
  累积（100GB）。修复白名单对齐 + **零进展保险丝**（tokenize 层：
  非 Eof token 必须消费 ≥1 字节，违反即按整 UTF-8 字符跳过并记错
  ——任何未来词法 bug 都只丢一个字节，不再可能 OOM）。
- **行首 yield/await 误判**：sloppy 标识符守卫曾检查关键词自身的
  newline_before——生成器里 `yield` 换行开头（最常见形态）被当
  标识符，操作数全部错位。改为纯前瞻形态判定（= . [ ( , ) ; EOF）。
- 语法缺口（Test262 正句法干净解析率 86.8% → **92.2%**，49,266
  文件零 panic）：
  - 标识符/成员名 `\uXXXX`/`\u{...}` 转义解码（含 ℘ 等
    Other_ID_Start 字符的宽容接受策略）
  - Unicode 空白（NBSP/U+2028/U+FEFF）的 trivia 处理
  - `async function*` 表达式、`new.target`、嵌套 `new new X()`
  - 对象字面量/类成员的生成器方法（`*m()`）、计算键方法
    （`[Symbol.iterator]()`）、数字/字符串键方法、计算键访问器
    （`get [name]() {}` / `static get [Symbol.species]() {}`）
  - 标签语句不再要求语句形态门（`yield: 1` 等 sloppy 标签）
  - `for (using x of …)` / for 头 using 声明
  - `var x \n = 1` 的 ASI 续行（`=` 不受限产生式）
  - 稀疏数组 elision（`[, ,42,,]`）
  - 对象解构模式带默认值（`({x = v} = o)`）→ 整段不透明原文
- Test262 接入为常驻测试（tests/t262.rs：抽样 4000 文件零 panic +
  解析率 ≥85% 基线 + 保险丝单元验证；语料缺失自动跳过）。

### 新增
- **TypeScript/JavaScript 后端**（v0.2.0 的主体）：`cure-ts-ast` /
  `cure-ts-parser` / `cure-ts-print` / `cure-ts-simplify` 四 crate 全链路。
  - 后缀 `ts/tsx/mts/cts/js/jsx/mjs/cjs` 经 CLI `LANG_BACKENDS` 注册，
    与 Java 同享目录遍历/并行/`--check`/`--diff`/`--stats` 基础设施。
  - **JS 值级语法**映射引擎词汇（箭头/函数表达式 → Lambda，for-of/
    for-in → ForEach，`??`/`**`/`in`/typeof/await/delete → 引擎运算符
    扩展），引擎 60+ 规则开箱即用；**TS 类型/装饰器/接口/枚举**以原文
    保真（ty_raw/sig_raw）；**JSX 与类表达式**整段原文捕获（不透明）。
  - **ECMAScript 折叠语义**：f64/ToInt32 域、±2^53 精确性回验、
    -0/NaN/±Infinity 拒折、位运算按 ToInt32 环绕、`**` 右结合。
  - **JS 专属安全守卫**：`var` 提升陷阱（块外引用 → 声明不透明）、
    λ 按引用捕获名整体非局部化（闭包逃逸写杀盲区——node 差分抓获）、
    可选链/解构模式不透明、字符串胁迫抑制（`is_exact_int` 仅字面量）、
    `!!x` 折叠按语言裁决（JS 布尔胁迫 vs Java 类型保证——Lang 钩子
    `double_not_sound`）。
  - **词法/解析**：模板字面量嵌套（quasi 续接 vs 标签模板的 cont 标记）、
    正则 vs 除法（花括号来源追踪启发式）、ASI（newline_before + 受限
    产生式）、BigInt/数字分隔符、全现代语法（逻辑赋值、可选链、
    静态块、私有字段、for-await-of、satisfies/as/非空断言、解构）。
  - **打印机**：ECMA Number→String（最短往返数字 + 1e21/1e-6 指数
    阈值——与 node 逐字节一致）、优先级驱动最小括号、幂等。
  - **node 差分测试**（29 用例黄金集）：-0/NaN/字符串胁迫/hoisting/
    闭包捕获/真值/ASI/松散 vs 严格相等/模板/正则/标签——原版与简化版
    的 stdout/退出码逐字节一致。差分当场抓获 3 个真实 bug（闭包逃逸
    写杀、`!!x` 布尔胁迫折叠、模板 quasi 双重转义）。

### 修复
- `store_kill` 异常窗口守卫精化：仅当**存在包围 Try** 时窗口内可能抛
  的语句才构成障碍（无 try 时异常直接出方法、局部变量不可见）——
  ddc 0.1.33 的「声明序列 → new 对象 → 击杀赋值」形态恢复清理。
- `while_iterator_to_for_each`：Map 族 `keySet()/values()` 接收位合成
  `Set<K>/Collection<V>` 静态类型（链式调用接收位曾被 var_type 门
  一律拒绝）。
- 传播守卫 `typed_position` 精化：值的声明类型未解析（反编译自由名）
  或与 cast 类型同名/Object → 恒可编译（形状层守卫曾一律拒绝）。

## [0.1.9] - 2026-10-22

### 性能
- **语句级名字索引**：局部传播、存写击杀、赋值回折、死声明四条规则
  不再为每个候选变量重扫后续语句或整棵子树。索引与区域事件共用同一
  失效机制（编辑沿祖先链清空槽位），语义与递归扫描逐项等价。
  同一索引继续消除 `AssignBackFold` / `DeadDecl` / `DeadStore` 的
  逐语句子树扫描，并把「已索引且零事件」与「未索引」区分开（大量
  无命名事件的语句不再让 `scan_region` 回退整树递归）。5k 行巨类的
  cure 阶段 5.4s → 0.22s；ddc 全量导出 deepseek 8.1s → 2.95s、
  微信 53.9s → 38.4s，产物逐字节不变。


### 新增
- 规则补齐至 64 条（引擎 34 + Java 30）：`arith_reassoc`、`bit_identity`、
  `self_compare`、`box_unbox_chain`、`string_builder_fold`、
  `trailing_assign_return`、`trailing_if_continue`、`trailing_continue`、
  `const_field_propagate`、`const_method_inline`、`url_decode_fold` 等
- **静态块虚拟执行**（`static_exec`）：解释器整体求值常量静态初始化器，
  物化为最终字段写（含数组别名保持、长/窄化域恢复）
- P1 不透明调用屏障：解密器形态的「字段 = 未知调用」截断点重放
- JSR 308 类型使用位置注解保留（返回位/类型参数 bound/cast/instanceof/
  泛型实参）；维度位注解以整段 Raw 保真
- 泛型方法调用 witness 保留（`Stream.<Path>empty()`）
- vexec 带标签 break/continue 流传播（跨层 Label 捕获）
- `--dead-code` 扩展：死私有方法/字段/空调用清理 + 未用 import 收尾，
  盲扫覆盖 @interface Raw 体、Lambda 显式参数类型、裸 @MethodSource 约定
- 反编译语料验证管线：ddc 反编译 5 个真实 APK（144,592 文件）全量
  三不变量 + javac 差分（ddc 输出可编译的包 cure 后必须仍可编译）

### 修复（10 轮攻击-修复循环 + ddc 集成循环，累计 60+ 项，全部带
最小复现与 javac/运行时对拍验证；摘录要点）
- 语义保持：`new String(常量)` 身份坍缩、装箱三入口（接收位/物化/传播）、
  raw 泛型接收位擦除、var-null 推断、可达性（JLS 14.21）、catch 检查
  异常依据、方法引用二段 witness、消费条件性（Lambda/Assert/三元/短路）
- 解析保真：破损 for 头/成员级/case 区三处上下文感知错误恢复（错误数
  有界、幂等、类结构不破坏）；文本块 JLS 3.10.6 首行丢弃；一元 `+`
  保留（数值提升即语义）；`when` 上下文关键字作绑定名；限定接收参数
  `X Y.this`；sealed 头 JLS 8.1 序
- 解析器事故：失控错误循环 4M 错误/529MB → 有界（停滞守卫三处）
- `const_method_inline` 语句位形态守卫（`mk(...).a;` 非法语句——
  ddc 反编译实锄件抓获）

### 变更
- 测试 170 → 332；`cargo clippy` 0 error（lib 0 warning）
- 入库语料 `--stats`：结构简化 1032 / 仅格式 1943 / 未变 46
  （行数 −53.4%、节点 −10.3%、判定点 −13.9%）

### 计划
- 还原规则 V2：反射 `addSuppressed` 形态、嵌套双资源 try-with-resources
- 第二语言接入，验证 `Lang` trait 边界的泛化性

## [0.1.0] — 2026-10-07

首个公开发布。

### 核心
- 容错式、语义保持的 Java 简化 / 格式化引擎：58 条规则
  （引擎通用 34 + Java 专属 24），fixed-point 收敛驱动
- 多语言就绪架构：字面量折叠语义全走 `Lang` 钩子（引擎不硬编码任何
  语言的算术）；`cure-tree` 通用树基建（子节点内联容器 / 名字 intern /
  区域事件索引 / 效果表重建）——新语言前端 ~100 行 Lang 实现即可组装
  （见 cure-tree 玩具语言测试）
- 控制流扁平化还原（`cff_recover`）：obfuscator.io / Allatori 风格
  `while(true){switch(s)}` 状态机 → 结构化控制流
- 部分求值器（`literal_eval`）：纯字面量 JDK 方法编译期求值
- 反编译形态还原（`twr_recover`、`string_switch_recover`）
- 语法错误不中断：错误区域原文保留，其余照常简化
- CLI：目录递归 + 多核并行（动态取号队列，默认线程 = 核心数 × 1.5）、
  `--check` / `--diff` / `--stats` / `--report` / `-j` 等完整选项

### 正确性
- 三层验证：属性测试（解释器对拍返回值 + 副作用序列）、
  javac/java 差分测试（stdout/退出码/异常签名逐字节一致）、
  语料库重解析 + 幂等检查
- 170 个测试；12 套自动发现入库语料 3,999 文件全绿；
  OpenJDK 全源码语料 371,674 文件 0 失败

### 性能
- OpenJDK 全源码语料（371,674 文件 / 4.70 GB）：`--check` 14.8 s / 峰值
  内存 ~430 MiB；写出输出树 27.5 s（18 核笔记本）
- 词法 token 全借用零分配 + 源借用免拷贝 + 符号表化
- 跨平台批量 I/O：Linux `io_uring` feature（read syscall 6,977 → 10，
  syscall 时长 −53%）；macOS/Windows 线程池；默认构建零运行时依赖

[Unreleased]: https://github.com/ejfkdev/cure/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ejfkdev/cure/releases/tag/v0.1.0
