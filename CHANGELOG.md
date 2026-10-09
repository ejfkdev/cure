# Changelog

本项目所有显著变更记录于此。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [Unreleased]

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
