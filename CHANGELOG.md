# Changelog

本项目所有显著变更记录于此。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 计划
- 还原规则 V2：反射 `addSuppressed` 形态、嵌套双资源 try-with-resources
- 第二语言接入，验证 `Lang` trait 边界的泛化性

## [0.1.0] — 2026-10-07

首个公开发布。

### 核心
- 容错式、语义保持的 Java 简化 / 格式化引擎：49 条规则
  （引擎通用 32 + Java 专属 17），fixed-point 收敛驱动
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
- 162 个测试；12 套自动发现入库语料 3,999 文件全绿；
  OpenJDK 全源码语料 371,674 文件 0 失败

### 性能
- OpenJDK 全源码语料（4.70 GB）18 核笔记本 ~15 s
- 词法 token 全借用零分配 + 源借用免拷贝 + 符号表化
- 跨平台批量 I/O：Linux `io_uring` feature（read syscall 6,977 → 10，
  syscall 时长 −53%）；macOS/Windows 线程池；默认构建零运行时依赖

[Unreleased]: https://github.com/ejfkdev/cure/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ejfkdev/cure/releases/tag/v0.1.0
