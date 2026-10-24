# Changelog

本项目所有显著变更记录于此。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 正确性（随机差分 + is_bool + prepare 顺序——2026-10-24 第十四批）

- **随机程序差分**（`randomized_program_differential`）：确定性 PRNG
  生成合法小程序，混合声明/同名遮蔽/`var` 提升/闭包/循环/三元/短路/
  模板/赋值；原版与 cure 版在 node 下比较 stdout + 退出码，并要求二次
  cure 逐字节一致。默认 80 例，`CURE_FUZZ_CASES` 可扩；开发中
  **1,000 例全部通过**。
- 修复 `is_bool`：`And/Or/Nullish` 只有**两侧都是语法级布尔**才是布尔。
  此前把任意逻辑表达式当布尔，`(-1 || true) || true` 被折成 `true`
  （应为 `-1`）——随机差分 135 抓获。
- 修复 prepare 顺序：绑定/`var_trap` 必须在区域事件与语句事实之前
  重建。顺序反了会按旧的“非不透明”状态缓存事件，导致
  `let v0=false; var v2=-4&v0;` 二轮才传播——随机差分 case 511 的
  幂等失败。现在首轮收敛。
- 回归：同步 33,963/34,007（44 allowlist，0 失配）、async 5,423、
  module 358、编译 49,165 valid / 0 破坏、往返/幂等各 49,159；
  npm 4,358 文件 **0 broken / 0 二次差异**；`cargo test --workspace`
  **456 通过**。

### 语法/规则（泛型函数/箭头 + 复合赋值——2026-10-24 第十三批）

- **TS 泛型函数/箭头**：函数声明/嵌套函数/函数表达式支持
  `<T extends …>`（含 `const` 类型参数），箭头支持
  `<T,>(x: T) => …` / `async <T>(x: T) => …`。`Lambda` 新增
  `generics_raw` 字段并原样打印；类声明 `class C<T>` 不再多打空格。
- **`ts_assign_to_compound`**：`a = a + b` → `a += b`（仅算术/位运算/
  移位/幂，目标与 RHS 左操作数须解析到同一绑定）。两种形态的求值序、
  GetValue/PutValue 和类型胁迫完全一致；成员目标 `o.x = o.x + 1`
  明确拒绝。
- 新增 `tests/modern_syntax.rs`（5 组：资源管理、导入属性/phase
  imports、装饰器/accessor/私有/static 块、TS 泛型与 `satisfies`、
  RegExp v/可选链）与 2 个 differential 用例（复合赋值语义、绑定感知）。
- 回归：Test262 六层全绿（同步 33,962/34,006、44 allowlist、async
  5,423、module 358、编译 49,165 valid / 0 破坏、往返/幂等各
  49,159）；npm 4,358 文件 **0 broken / 0 二次差异**（编辑数
  52,098 → 52,294）；`cargo test --workspace` **455 通过**。

### 性能（增量 effect 缓存 + 紧凑 Token——2026-10-24 第十二批）

- **增量 effect 缓存**：旧实现每个 pass 对整个 arena 跑
  `rebuild_effects`（4 轮 × 全部节点），13.6 MB 的 `pnpm.mjs`
  9085 个 pass 下耗时 31.8 s。现在 effect 只在首次/换根时后序计算
  可达节点，编辑沿祖先链标脏，prepare 只重算脏子树。同一文件
  **31.8 s → 4.75 s（约 6.7×）**，指令数 275B → 75B；格式-only
  仍为 0.41 s。
- **Token 紧凑布局**：`line/col/start/end` 从 `usize` 改为 `u32`，
  Token 64→48 字节。2.09M token 的大文件减少约 33 MB 结构占用；
  格式-only 峰值 RSS 327→294 MB，完整流程 506→476 MB。
- 真实 npm 语料 4,358 文件：约 10.5 s（wall），输出与编辑数不变，
  **0 broken / 0 二次差异**；Test262 六层全绿（同步 0 失配、async
  5,423、module 358、编译 49,165 valid / 0 破坏、往返/幂等各
  49,159）；`cargo test --workspace` **448 通过**。

### 健壮性（深嵌套栈护栏 + Raw 条目换行——2026-10-24 第十一批）

- **递归深度护栏**：parser 对语句/表达式/一元链统一计数，超过
  `MAX_NEST`（512）即停止深入并把**整文件**作为 `TopItem::Raw` 原文
  保真返回（清空部分 AST 的语义输出，保留一条诊断）。此前 20k 层括号
  会让进程栈溢出 abort；现在退出码 0、输出与输入逐字节一致，二次
  cure 稳定。
- **Raw 顶层条目换行**：Import/ExportRaw/Raw 与无体函数声明不再各自
  追加 `\n`（`print_module` 已统一补条目换行）。此前 raw 条目后多出
  空行，深嵌套 raw 文件首轮输出多一个换行、二轮才收敛。
- 新增回归 `deep_nesting_falls_back_to_verbatim_source`（2000 层括号：
  原文保真 + 二次幂等）。
- 回归：六层 Test262 全绿（同步 0 失配、async 5,423、module 358、
  编译 49,165 valid / 0 破坏、往返/幂等 49,159）、npm 4,358 文件
  **0 broken / 0 二次差异**；`cargo test --workspace` **448 通过**。

### 规则（绑定身份接入引擎别名判定——2026-10-24 第十批）

- 新增 `Lang::same_binding`：默认按名字键（Java 语义不变）；TS/JS 后端
  覆盖为 `BindingId` 等值——两侧都未解析时回落名字键，一侧解析一侧
  未解析则拒绝（保守）。
- 引擎规则改用绑定身份判定“同一存储位置”：`self_assign`、
  `assign_back_fold`（声明↔赋值目标 / `x=x op K` 的 RHS 左操作数）、
  `dead_store`（声明/前赋值/后赋值目标，以及 b 中是否引用旧值）、
  `local_propagation` 的纯自赋值守卫、`x - x → 0` / `x ^ x → 0`、
  `inverse_assign_pair`（NormAssign 新增 target）、
  `if_assign_ternary`。名字键仅保留为快速预筛与扫描窗口（保守命中）。
- 效果：同名遮蔽不再被当成同一变量；`var`/`let` 提升、catch/块内
  遮蔽、具名函数表达式等场景的别名折叠有绑定级依据。npm 语料的
  编辑数与输出不变（52,098 次编辑），Test262 打印往返/幂等仍
  49,159/49,159。
- 回归：同步 **33,962/34,006**、async **5,423**、module **358**、
  编译 **49,165 valid / 0 破坏**、npm **0 broken / 0 二次差异**、
  `cargo test --workspace` **447 通过**。

### 架构（TS/JS 绑定/作用域 resolver——2026-10-24 第九批）

- **每文件绑定 resolver**（`crates/cure-ts-ast/src/resolve.rs`）：
  建作用域树，`var` 绑定到最近函数作用域、`let/const/using` 到当前块，
  覆盖块/for/for-of/catch/参数/函数声明/具名函数表达式；引用解析到
  `Binding` 身份，对外提供 `bindings()` / `binding_of()` /
  `binding_id()` / `BindingKind`。
- 替换旧的名字集合遍历：`local_marks`、`captured_names`、
  `mutated_names`、`var_trap` 现在都由绑定事实推导；`is_local_var`
  使用**绑定级捕获**（lambda 自己的参数不再被同名全局捕获牵连）。
- 修复/加固：
  - 顶层 `let/const` 声明必须不透明：class/函数体是模块合成根之外的
    顶层项，其中的写对声明级规则不可见（`let x=0; class C { m(){++x;} }`
    曾被 local_propagation 折成 `0`）。resolver 把所有模块级声明标
    opaque，不再只覆盖 `var`。
  - 具名函数表达式名字是**不可变绑定**（sloppy 赋值静默忽略、strict
    抛错）：新增 `BindingKind::FunctionExprName`，禁止当普通局部变量
    做写删/传播；`ts_trailing_assign_return` 同时要求赋值目标与返回
    引用解析到同一绑定（同名遮蔽不再误判）。
  - 写计数按绑定：`mutated_names` 不再把未解析全局的同名写算到局部
    绑定上（减少误保守），raw 解构模式仍按词保守登记。
- 回归：同步语义 **33,962/34,006（99.87%）**、44 个 toString allowlist、
  async **5,423/5,423**、module **358/358**、编译 **49,165 valid / 0
  破坏**、打印往返/幂等 **49,159/49,159**；npm 4,358 文件 **0 broken /
  0 二次差异**；`cargo test --workspace` **446 通过**。

### 验证（Test262 打印往返 + 简化幂等——2026-10-24 第八批）

- **打印往返结构差分**（`examples/t262_roundtrip` +
  `tests/t262_roundtrip.rs`）：每个 Test262 文件解析 → 打印（不简化）
  → 再解析，按原文件的 script/module 目标比较 AST 结构等价（忽略
  `Paren`、字面量拼写、Group/多声明符展开）。**49,159/49,159 一致，
  0 漂移**；负向用例跳过（无效输入不属于打印契约）。
- **简化幂等差分**（`examples/t262_idem` + `tests/t262_idem.rs`）：
  cure 输出再跑一遍 cure，文本必须逐字节一致。**49,159/49,159 稳定，
  0 漂移**。
- 两层都用 `T262_OFFSET/T262_LIMIT` 分块、支持 `T262_DUMP` 落盘失败
  样本；`equiv_check` 调试工具（`CURE_EQUIV_DEBUG=1` 打印首个差异
  节点）。新增 `parse_module`/`parse_script` 与顶层 `import`/`export`
  自动判定；CLI 对 `.mjs/.mts` 强制 module、`.cjs/.cts` 强制 script。
- 抓出并修复：
  - **多声明符导出丢导出**：`export var a=1,b=2;` 曾打印成
    `export var a=1;
var b=2;`——`b` 变成非导出（模块语义改变）。
    `ExportStmt(Group)` 现在逐声明加 `export`。
  - **装饰器解析**：`skip_decorator` 把成员名一起吞掉，`@dec m() {}`
    整个成员塌成 `Raw("{}")`，类级 `@dec class C` 的装饰器也被丢弃。
    现在装饰器成员整条原文保真（规则不进入装饰器语义），类级装饰器
    正确进入 `ClassDecl.prefix`。
  - **await 语境**：`await (x)` 在非 async 函数里是名为 await 的函数
    调用（Test262 `private-field-rhs-await-absent`、嵌套 function/
    generator 用例）；此前按 Await 一元解析导致 SyntaxError。新增
    `async_ctx`——async 函数/async 方法/async 箭头/module 顶层为真，
    普通函数与箭头边界重置；`await`,`=`,`yield` 等标识符形态保持。
    同时给 parser 增加 module goal，避免顶层 await 的 script/module
    误读。
  - **长常量链首轮不收敛**：`var B="0"+A; var C="1"+B; …` 每轮只前进
    一层，六轮上限后仍有残留（二次 cure 才清完）。`const_init_ok` 已
    保证 Binary/Paren 常量的操作数全为标量、效果 ≤ MayRead，现在它们
    首轮直接进入 scalar 集合；`1.0 + ''` 等整值 Double 字符串化也在
    首轮折叠。
  - **Group 死声明存活**：`var wi=1024,Ti=64*wi*wi;` 中 Ti 传播后，
    wi 的唯一引用来自将死的兄弟——只按“任一兄弟引用”判定会保留 wi，
    二轮才删（npm 真实语料幂等差）。改为组内“被活兄弟引用”的不动点
    可达分析，首轮即删。
  - **括号数组/对象常量同一性**：首轮 scalar 判定递归到 Paren 内部，
    只有 Binary/Paren(标量) 才视为原始值；`const A=([1,2]); A.push(3)`
    不再把数组传播成互不影响的新副本（回归用例
    `diff_paren_array_identity`）。
- 回归：同步语义 **33,956/34,000（99.87%）**、44 个 toString 源文本
  限制、async **5,423/5,423**、module **358/358**、编译
  **49,165/49,165 合法文件 0 破坏**、打印往返/幂等 **49,159/49,159**；
  npm 4,358 文件编译 **0 broken**、两轮 **0 差异**；
  `cargo test --workspace` **439 通过**。

### 验证（npm 真实语料编译差分 + 幂等——2026-10-23 第七批）

- **真实 npm 语料**：全局安装 `/opt/homebrew/lib/node_modules` 下
  4,358 个 `.js/.mjs/.cjs`（约 568 MB，含 undici、pnpm、yarn、
  vite-plus、Lark CLI、bytenode 等）。目录模式全量运行后做两项校验：
  合法文件编译差分 **4,174 ok / 0 broken**（184 个原版自身不可编译
  或环境跳过）；对 cure 输出再跑一次 cure，`diff -rq` **0 差异**
  （早期为 80 → 59 → 5 → 0）。
- 修复（真实语料驱动——解析器/打印器）：
  - 控制语句头 `)` 后可起正则：`for (x of y) /re/.test(x)` 曾被
    当除法；调用/分组结尾仍按除法（`last_close_paren_pre`）。
  - TS 上下文关键字按声明形态判定：`module.exports = fn()`、
    `namespace.foo()`、`interface = 1` 不再被整条截成声明残句
    （pngjs 编译差分抓获）。
  - 解构 for-of/for-in 头只保留绑定原文、不整条捕获 body——单语句
    体里的 `if/else if` 链不再被语句边界扫描截断（oxfmt 的 `else`
    残句化）。
  - `DestructStmt` 消费自身分号：曾每轮解析多出一个 Empty `;`
    （`for (…) { [a[i],a[j]] = …; ; }` 无界增长）。
  - 箭头返回类型前瞻增加参数形态状态机：`c ? (a=()=>x,b=f()) : …`
    不再被误读成 `(…): T =>` 箭头。
  - 原文段拼接 `push_raw`：末行含 `//` 时补换行——参数尾注释曾吞掉
    后续 `) => {`（vitest `diff.js`）。
- 修复（真实语料驱动——常量/幂等）：
  - 模块多声明 Group 的常量接纳与死项清理按组内逐声明进行；
    `var p=…,a=p+…` 首轮即传播，且组外引用不再误删组内声明。
  - eval 污染扫描只沿**当前根可达**节点——arena 中已脱离树的
    Group 孩子不再把整个模块标成不透明（首轮 0 改写、二轮才优化）。
  - Group 中仅 `var`/`using` 声明保持不透明；`let`/`const` 的声明
    初始化式可首轮折叠（`const TICK_MS = (1000 >> 1) - 1` 曾二轮才
    变 499）。
  - 整值 `Double`（≤ ±2^53、fract=0）进入 ToInt32 折叠域：
    `1e3 >> 1` 与 `1000 >> 1` 首轮同折为 500。
  - `super.x` / `super[x]` / `super(…)` 原文作调用/成员基座时不再
    加括号——`await super.close()` 曾打印成 `await (super.close)()`，
    二轮 await 被当标识符调用变成 `await(super.close)()`。
- 回归：Test262 四层复跑——同步 **33,960/34,004（99.87%）**、
  44 个 `Function.prototype.toString` 源文本限制（新增 staging
  async/generator toString 用例入 allowlist）、async **5,423/5,423**、
  module **358/358**、编译 **49,165/49,165 合法文件 0 破坏**；
  `cargo test --workspace` **432 通过**。

### 验证（TS/JS module 语义差分——2026-10-23 第六批）

- **Test262 module 语义差分**（`examples/t262_module_sem` +
  `tests/t262_module.rs` + `scripts/t262_module_driver.mjs`）：module
  用例作为 ESM 在独立 vm 上下文链接（相对 fixture 由文件系统加载，
  含 async module 的 `$DONE`/定时器）并求值——**358/358 个可执行用例
  结果一致（0 失配）**，233 个原版自身失败（fixture/JSON 导入等）跳过。
- 修复：块展开守卫把**原文语句（Raw）**视为潜在词法声明——
  `{ await using z = …; }` 曾被展开合并作用域导致重复声明
  SyntaxError（module/await-using 用例抓获）；await using 语句以
  Raw 保真后必须阻止 BlockFlatten。

### 验证（TS/JS async 语义差分——2026-10-23 第五批）

- **Test262 async 语义差分**（`examples/t262_async_sem` +
  `tests/t262_async.rs` + `scripts/t262_async_driver.mjs`）：`flags`
  含 async 的 5,516 个可执行用例逐例在独立 vm 上下文执行，等待
  `$DONE`/微任务/定时器排空后比较异常状态与 stdout——**5,423 个执行
  结果一致（0 失配）**，93 个原版自身失败跳过。抽样 200 例已固化为
  常驻测试。
- 已知环境边界：`await-using/syntax/` 组在 node/V8 上会让**原版**
  SIGSEGV（与 cure 无关），async 语义层跳过该目录；编译层仍覆盖。
- 修复：`await using x = …;` 语句整条原文保真（曾拆成
  `await using; x = …`，异步 dispose 时机与不可变绑定全丢——
  staging explicit-resource-management 用例抓获）；async 驱动逐条
  输出 + 逐例清理定时器，避免长跑内存累积。

### 验证（TS/JS 编译差分——2026-10-23 第四批）

- **Test262 全量编译差分**（`examples/t262_compile` +
  `tests/t262_compile.rs` + `scripts/t262_compile_driver.mjs`）：全部
  49,266 个文件（含 async/module/negative）解析→简化→打印后，原版与
  cure 输出各自在 node 中编译（script `vm.Script` / module
  `vm.SourceTextModule`，只编译不执行）。**49,165 个合法原版文件在
  cure 后全部仍可编译（0 破坏）**。负向语法用例不计入断言（容错输出
  重新可编译不属于语义保持面）。抽样 1200 例已固化为常驻测试。
- 全量分块（`T262_OFFSET/T262_LIMIT`）避免一次性构建 5 万用例占内存。

### 修复（编译差分驱动——ESM/Annex B 语法）

- **Annex B HTML 注释**：`<!--`（任意位置）与行首 `-->`（含前导
  多行注释内含行终止符的 `0/*\n*/-->` 形态）按注释跳过——曾把
  `-->` 解析成 `--`/`>` 残句（annexB comments 家族 7 例 SyntaxError）。
- **ESM 头部**：`export default class/function` 保留 `export default`
  （匿名默认导出曾丢 default 变成非法 `export function() {}`）；
  `export * from …` 不再丢 `export`（raw 捕获起始位）；
  `ClassDecl` 新增 `prefix` 字段承载装饰器/export/default/abstract。
- **动态导入**：顶层 `import(...)` / `import.meta` 不再被当静态 import
  原文捕获（`.then` 链曾被拆散）；静态 import/export 原文捕获支持
  跨行导入属性 `with {…}`（属性子句内换行不触发 ASI）。
- **顶层 await**：`await [..]` 按一元 await 解析（曾走标识符索引路径
  产出 `await[]]`）；`for (await using …; …; …)` 经典 for 头整条原文
  保真（await-using forstatement 家族 SyntaxError）。
- **规则锚点守卫**：`fold_root` 对 VarDecl init 根逐个规则直调（绕过
  kind 分派）——`ts_try_cleanup` 等 ported 规则补 `kind(id)` 自证，
  曾把 `await function() {}` 的 init 一路替换成函数体 `Block`
  （输出 `/*?*/`）。

### 验证（TS/JS 语义差分——2026-10-23 第三批）

- **Test262 语义差分常驻化**：`crates/cure-ts-simplify/tests/t262_sem.rs`
  ——原版与 cure 版在全新 node vm 上下文分别执行，比较异常状态与
  stdout；全量扫描 34,006 个可执行用例，**33,962 个逐字节一致
  （99.87%）**，零非 toString 失配。
- 已知不可保持面（allowlist，文档化）：`Function.prototype.toString`
  依赖函数源码文本（cure 会重排函数体）；以箭头函数作计算属性名的
  `{[() => {}]: v}` 经 toString 参与键比较。其余失配即硬失败。
- 语料/环境缺失（`/tmp/test262`、`node`）时测试自动跳过；
  `T262_ROOT`/`T262_SAMPLE` 可覆盖。手动全量：`examples/t262_sem`。

### 修复（Test262 差分驱动——解析器/打印器）

- **行终止符与 ASI**：CR / CRLF / U+2028 / U+2029 正确计为
  LineTerminator（`return`/`break`/`continue`/后置换行语义）；字符串
  行续接（反斜杠 + CR/CRLF/U+2028/2029）整段删除；模板 quasi 内的
  多字节转义完整保留（曾 mojibake）。
- **字面量域**：LegacyOctalIntegerLiteral（`070` = 56）与 legacy 八进制
  字符串转义（`\07`）；`\u{D800}` 等孤立代理项保留原文（JSON.stringify
  码元语义）；字符串序比较按 **UTF-16 码元**（非 Unicode 标量序）。
- **生成器**：`yield` 作为表达式/三元条件/`in` 右操作数/模板插值；
  `yield *` 换行操作数；`(yield)` 括号语义必需。
- **类/函数边界**：`class extends class {} / function(){}` 嵌套体；
  `new tag\`…\`` 与 `tag\`…\`` 后缀；`new (f().g)()` 的 callee 括号；
  函数/箭头表达式源码空档（name/toString 相关）；对象字面量访问器
  （getter 整段原文）与计算键方法识别；计算键值不再被当方法简写。
- **语句/头部文法**：`for (let in obj)` 的 let 绑定、`async of => {}`、
  三元内 `in` 不计 for-in 分隔、for-init 顶层 `in` 加括号、
  `for (var a = ++effects in {})` 初始化器、`using`/`await using`
  声明；`{__proto__ = 0} = {}` 解构语句；私有名 `#f in o`；
  多声明符 + 解构混合 for 头共享一个 let/const。
- **打印安全**：稀疏数组尾洞（`[,]` 长度保持）、对象模式赋值整条
  加括号、`??` 与 `&&/||` 混合双向括号、`**` 一元操作数括号、函数体
  `}` 后 `/` 的正则/除法判定（语句块/声明 vs 表达式）。

### 修复（Test262 差分驱动——规则/引擎）

- **常量传播/求值禁区**：正则字面量不作标量常量（`g` 正则 lastIndex /
  对象同一性）；BigInt/正则实参不可随死调用丢弃；`decodeURI` 保留
  保留字符转义；字符串方法位置参数（includes/startsWith/endsWith）
  按 ToIntegerOrInfinity；`Number()` 的 `-0`、带符号十六进制 NaN、
  `parseInt("−0")`、isFinite/isInteger/isSafeInteger 的类型/边界域；
  `Math.round(±0)` 符号；`String.prototype.repeat(-1)` 不得被死调用删除。
- **写/读不可见区**：`delete x` 的常量替换、`++(x)` 的写计数、对象
  字面量访问器/函数表达式形参/catch 解构默认值/for-of 解构头的原文
  写（raw_words/Lambda params）；数组逃逸上下文白名单收紧（delete/
  update/成员赋值/ObjLit 逃逸）。
- **作用域与提升**：直接/间接/括号 eval 全单元禁用常量传播并污染
  eval 函数子树（含形参默认值）；`arguments` 函数子树保守化；
  let/const/using 禁止死声明/死存储删除（TDZ/块级遮蔽/资源释放时机）；
  `if (c) function f(){}` 的 Annex B 条件初始化与块展开守卫；
  `(yield)`/`(fn) = function(){}` 的括号语义；合成 Group 的直接声明
  不参与语句表规则（`let a, b;` 曾整条删除）。
- **求值时机**：局部传播仅限字面量或「未改写本地绑定」的纯标量
  表达式；store_kill/dead_store/assign_back_fold 遇到原文/类体/解构
  等不可见区域即停止；`x && false` / `x || true` 仅对布尔操作数折叠
  （JS 非布尔值与 ReferenceError）。
- 引擎：`DeadDecl` 自引用 init 守卫（TDZ）、`static_exec` 零降本提案
  拒绝、`reassoc_delta` i64 环绕（debug 溢出 panic）。

### 新增（TS/JS 混淆对抗——2026-10-23 第二批）

- **模块级常量链传播**（`ts_const_propagate`）：混淆器拷贝链
  `var a = 0x1; var b = a + 0x1;` 的先序接纳（零写/零遮蔽/同名唯一
  /init 纯净且自由变量全为先序标量）——值语义传播（数组/对象引用
  不传播——同一性破坏）。写扫描含 `arr[i] =` 基座与 `++/--` 目标。
- **常量数组下标折叠**（`ts_static_array_index`）：`_0x3f2a[2]` →
  元素字面量（含内联后的数组字面量直配形态）。**数组逃逸扫描**：
  候选数组的每个 VarRef 须处于只读上下文（Index 读基座/纯方法
  白名单接收者/for-of 迭代/spread/模板）——`copy = arr; copy.sort()`
  /`log.push(x)` 的经别名/方法突变对写扫描不可见（node 差分
  sorting_object/sequence 抓获）。
- **常量函数内联**（`ts_inline_func`）：`_0x5d3e(0x2)` → 函数体
  return 表达式（参数字面量替换）——字符串数组访问器/解密 helper
  的调用点展开。守卫：实参全字面量、体无 this/arguments/自引用/
  Raw。结构性声明（净简化由后续折叠完成）。
- **真值折叠**（`ts_truthy_fold`）：`!![]` → true、`!{}` → false
  （JSFuck 风格布尔噪声；经引擎 bool_not_fold 链式收敛）。
- **引号成员还原**（`ts_member_unquote`）：`console['log']` →
  `console.log`（字符串键为合法标识符且非数字形态）。
- **模块语句合成单根**（ModuleBlock）：顶层语句拼成一条语句列表——
  引擎获得完整父链（`if (true)` 解包、跨语句事件分析）+ 按位重分发。
  模块级名字不绑作用域（跨根读不可见）；直接 VarDecl 孩子不透明
  （多声明符 Group 拆开入块保证标记覆盖——`},Yn=-1;` 曾被组内
  规则删除，node 差分 real_16 抓获）。
- **模块死项清除**：常量/内联函数全使用位消灭后移除声明。门控：
  ESM 模块检测（import/export 存在——classic script 顶层是全局
  属性不可证安全）或 `--dead-code`。候选项自身签名不算引用。
- **export 声明真解析**（`TopItem::ExportStmt`）：`export const x = 1`
  的值级照常简化（曾整条原文捕获）。
- **十六进制归一化**：`0x1f` → `31`（混淆器偏爱 radix 形态）。

### 修复（解析器——真实语料差分驱动）
- **箭头返回类型前瞻的语句边界终止**：`cond ? (a ? b : c) : else`
  的三元 else 冒号曾被当返回类型起点扫到**后续语句**的 `=>`——跨
  语句吞成假箭头（`ctx.io === "input" ? (...) : def.out;` 之后的
  readonlyProcessor 整段错位——real_81 抓获）。`;`/`,`/裸 `{`/`}`
  顶层即中止。
- 解构 for-of 原文捕获**从 `for` 关键字起**（曾从 for 头起丢了
  `for (` 前缀，打印出 `const [k] of …))` 残句——real_198/91 抓获）。

### 验证
- node 执行对拍：**200/200** 真实库文件（VS Code/babel/zod 等捆绑
  产物）原版 vs 简化版 stdout/退出码逐字节一致；混淆样本 4 用例
  差分（字符串数组链/十六进制链/安全形态保持/轮转调用链）。
- 全 workspace 428 测试绿；真实语料 0 打印回归、第二轮起文本不动点。

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
