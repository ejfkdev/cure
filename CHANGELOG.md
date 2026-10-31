# Changelog

本项目所有显著变更记录于此。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [0.4.1] - 2026-10-31

### 修复

- **行尾指令注释不再因简化换行/失效**（Python / Dart）：带工具指令注释
  的语句（Python `# type: ignore`、`# noqa`、`# fmt:`、`# pyright:`、
  `# pylint:`、`# mypy:`、`# pragma:`、`# nosec`、`# isort:`；Dart
  `// ignore:`、`// coverage:ignore-*`、`// dart format`）此前可能被常量
  传播、死赋值/死存储、不可达清理、尾 `return` 删除或尾赋值合并移除，
  注释顺延到下一行，导致 mypy/flake8/pylint/analyzer 的抑制失效或误伤
  后续代码。现在这些语句不再被删除或合并，只允许同一行内的安全替换
  （如 `return y + x; // ignore: …` → `return 6 + x; // ignore: …`）。
  补 Python/Dart 回归测试；TS/JS 的指令冻结机制此前已覆盖，行为一致。
- 无指令注释的同类代码仍按原规则简化。

### 验证

- workspace 全测试（debug + release，618 项）通过；MSRV 1.74
  `cargo check --workspace` 通过。

## [0.4.0] - 2026-10-31

### 修复（版本兼容性）

- **Java 25 紧凑源文件**（JEP 512）：省略外层 `class`、顶层直接写字段/
  方法/实例 `main` 的源文件曾被报 `expected type declaration` 并把整段
  按 raw 保真（`--strict` 会失败）。现在识别隐式类成员形态，0 诊断、
  幂等，内容仍逐字节保留；JDK 25 编译验证通过。JDK 27 预览的基元模式
  （`case int i`）与未命名变量也验证可解析/保真。
- **Python 3.5/3.6 `await` 软关键字**：`await = 1`、`def f(await=2)`、
  `f(await=4)` 曾按现代 `await` 运算符解析报错。解析器现在按后随 token
  消歧（赋值/分隔/属性位按标识符，其余保持现代 async 语义），3.5/3.6
  旧代码与 3.7+ 代码都能解析。
- **版本矩阵核对**（临时语料，不作为常驻测试）：Python 2.0–3.15
  （py2 print/exec/raise/反引号/`<>`/long/元组形参 → t-string/PEP 695/
  696/701/758/810）、Java 8–27（records/patterns/sealed/text blocks/
  module import/compact source/primitive patterns 预览）、ES5–ES2026
  （async generators/class fields/static blocks/import attributes/using/
  iterator helpers）、TS 1.x–5.x（decorators/const type params/type
  arguments/satisfies/using）、Dart 1.24–3.13（旧 typedef/final 形参 →
  records/patterns/extension types/primary constructors/dot shorthands）
  均 0 诊断、二次运行 0 漂移；有可用 SDK 的组合（Node/tsc、javac、
  Dart 1.24.3/2.19.6/3.13、Python 3.15）编译或诊断集合与原文件一致。
- **注释保真（大幅补全）**：
  - **Python**：语句级注释全部保留——独立行注释、行尾注释（含
    `# type: ignore`、`# noqa`、`# fmt: off`）、文件头 license/编码块；
    打印器按源码行把独立注释放到下一条语句前、行尾注释接在语句末尾。
    简化删除语句时其注释会顺延到下一条语句/文件末尾（不丢失）。
  - **TypeScript/JavaScript**：保留文件头块与行级指令（`// @ts-ignore`、
    `@ts-expect-error`、`/*#__PURE__*/`、`eslint-disable-next-line`、
    `prettier-ignore`、`istanbul ignore next`、`@vite-ignore`）；带指令的
    语句按源码原文行结构整段输出（不再展开成多行），因此 `@ts-ignore`
    抑制的仍是原来那一行（曾把单行 `class Bad { x: number = "y"; }`
    展开后导致 tsc 多报一条）。带指令语句同时被标记为不透明，规则不会
    改写/删除它。普通叙述性注释在 TS/JS 代码中间仍不保留。
  - **Java**：文件头注释块（license、`//noinspection` 等）原样输出在
    package/import 之前；代码中间的行级注释仍不保留。
  - Dart 仍是唯一逐 token 全注释保真后端；TS 的
    `--preserve-function-source` 可保留函数/类/对象字面量内部注释。

### 增强（简化规则）

- **Python 27 → 28**：新增 `py_local_propagation`——函数首条
  `y = <标量字面量>` 且 y 只绑定一次、不是形参/`global`/`nonlocal`/
  `del`/海象目标、嵌套作用域不重绑定、函数无 locals/eval 反射时，把
  字面量内联到所有读取位（裸字符串表达式语句除外，避免生成新
  docstring）；`del y` 同时计为读取与禁用标记，防止把赋值删成
  UnboundLocalError。
- **Dart 24 → 25**：新增 `dart_local_propagation`——单 token 的
  `final/var/const y = <标量字面量>` 在块内所有出现都是明确的表达式
  操作数时内联并删除声明；声明/参数表/catch/模式/`for` 头/多声明器/
  字符串插值/成员访问/可空后缀位都会让规则整体跳过（token 模型无法
  改写插值或区分遮蔽绑定）。Dart 1/2/3 运行时差分（含局部变量、
  `this`/闭包/循环用例）0 失配。
- **TS/JS 新增 `ts_iife_inline`**：零参、无类型参数、单表达式或单
  `return` 体的 IIFE（`(() => expr)()`、`(function(){ return expr; })()`）
  内联为表达式；普通函数若体表达式依赖 `this`/`arguments`/`super`/
  `new.target` 则保留，含 `eval` 或异步/生成器/有参形态保留。node
  运行时差分（class 方法 this、arguments、eval、strict this）0 失配。

## [0.3.1] - 2026-10-27

### 修复

- **MSRV 1.74**：`py_trailing_assign_return` 里用了 1.82 才稳定的
  `Option::is_none_or`，导致声明 `rust-version = 1.74` 的 0.3.0 crate
  在 1.74–1.81 上无法编译（CI 的 MSRV 检查报警）。改用 `map_or`，
  `cargo check --workspace` 在 1.74 上通过；行为不变。

## [0.3.0] - 2026-10-27

### 新增（Dart 1/2/3 后端）

- **新语言后端**：新增 `cure-dart-ast` / `cure-dart-parser` /
  `cure-dart-print` / `cure-dart-simplify` 四个 crate 并接入 CLI；
  `--lang dart`（别名 `dart3`）与 `.dart` 自动识别，目录模式、`--check`、
  `--diff`、`--report`、`--stats`、`--disable`、`-w`、`-j` 全部可用。
- **无损 token 模型**：Dart 1/2/3 不建版本分支——词法层完整保留所有
  token 与注释，结构层只建立括号配对与「块/集合/switch」花括号分类。
  未建模语法（Dart 2 遗留 `final` 形参、旧式函数 typedef，Dart 3
  records/patterns、class 修饰符、extension type、switch 表达式、macro）
  原样保留，因此同一后端接受所有版本写法。注释、raw/三引号/插值字符串
  （含 `${...}` 内的嵌套字符串）、二/十六进制与下划线数字、全部运算符
  形态均逐字节保真。
- **规则（24 条）**：`dart_const_fold`（32 位整数/bool/字符串及
  `.length`/`.codeUnitAt`/`.contains`/`.indexOf`/`.isEven`/`.bitLength`/
  `toRadixString` 等确定性字面量成员；`%` 按 Dart Euclidean 语义、`~/`
  截断、移位/溢出超界不折；`/` 与浮点不折；带运算符优先级与一元前缀
  上下文检查（含 Dart 的「位运算优先于相等」优先级），`x - 1 + 2`、
  `x & 0x60 == 0`、`-1 % 3` 不会被折错；控制流条件里的 bool 折叠跳过，
  三元选择交给 `dart_ternary_const`，避免改变 late 变量流分析）、
  `dart_ternary_const` / `dart_ternary_bool` / `dart_ternary_not`
  （常量条件；`dart_ternary_const` 只在控制流条件之外选择同类字面量分支；
  `c ? true : false` 仅在条件可证为 bool 时折成 `c`；`!c ? a : b` →
  `c ? b : a`）、`dart_bool_literal_op`（`true && x`/`false || x` → `x`、
  `false && x`/`true || x` → 字面量、`x && true`/`x || false` → `x`；
  保留的操作数必须可证为 bool，动态值保留运行时 bool 检查；
  Dart 3 模式上下文跳过）、`dart_double_negation`（操作数可证为 bool
  时才去掉双否定，避免动态值类型检查差异）、`dart_negated_eq`
  （`!(a == b)` → `a != b`、`!(a is T)` → `a is! T`，按上下文补括号）、
  `dart_const_if`（常量 bool 且分支为块，保留块作用域；分支含
  赋值/`return`/`yield`/for-in 时跳过，避免改变流分析）、
  `dart_if_return`（`if` + return / else-return → 带括号条件的三元 return，
  仅当两分支是同类字面量，避免三元 LUB 改变静态返回类型；其他情况保留
  独立 return；if-case 模式不动）、`dart_if_assign_ternary`（同目标赋值分支 →
  `x = cond ? a : b`，仅同类字面量 RHS）、`dart_else_after_terminal`
  （then 分支必然终止且 `if` 是直接语句时去掉 else，else 块保留为裸块；
  无括号 else/循环体/标签语句不动）、`dart_nested_if`（单层嵌套 if 合并为
  `a && b`）、`dart_empty_then`（`if (c) {} else X` → `if (!c) X`）、
  `dart_assign_compound`（`x = x + y` → `x += y`，仅普通标识符且 RHS
  顶层没有更松/同级运算符，`x = x * 10 + digit` 不动；不做
  `x = x ?? y` → `x ??= y`，因为 `??=` 在变量非空时可能跳过 setter，
  而原式总会赋值）、
  `dart_unreachable`（直接终止语句后的代码；含
  `return`/`yield`、赋值或 for-in 的不可达区间保留，避免改变返回类型
  推断和 late 变量 definite assignment；带标签块跳过）、
  `dart_trailing_continue`（循环尾 `continue;`）、`dart_trailing_return`
  （无注释的函数/闭包末尾裸 `return;`）、`dart_empty_finally`
  （仅在仍有 catch/on 时删除空 finally）、`dart_try_unwrap`（仅 rethrow
  的 `catch`/`on` 子句删除；删空后或只剩空 `finally` 时解包为裸块，
  保留作用域）、`dart_adjacent_strings`（简单字面量 `'a' 'b'` 隐式拼接）、
  `dart_paren_removal`（仅原子表达式，避开控制条件/
  调用实参/函数表达式）、`dart_empty_else`、`dart_string_interp`
  （`${ident}` → `$ident`，后续字符必须是 ASCII 且不能延长标识符；
  非 ASCII ID_Continue 字符保守跳过；raw/嵌套字符串不动）。
  注意不删除 `new`：Dart 1 要求构造调用带 `new`，删除会破坏旧版本语义。
- **格式化器**：2 空格缩进、块/集合区分、switch case 缩进、泛型尖括号
  紧凑、`operator[]=` 与可空类型后接 `..` 的词法边界保持、注解与跨行块
  注释的换行保持、原始空行段落保留；`return !x` 与 `is! T` 的间距修正。
  全语料 token 序列与格式均幂等。
- **验证**：300 个 pub-cache 包的 **19,176** 个 `.dart` 文件，
  0 解析诊断；19,176/19,176 逐 token 往返一致、二次运行 0 编辑 0 字节
  差异；`dart format --output=none` 在 cure 前后拒绝同一批 90 个 Dart 3
  已移除语法的旧文件（0 新增失败）。Dart 1.24.3 / 2.19.6 / 3.13 均作为
  oracle：精选版本语法（旧 typedef、`final` 形参、null safety、super
  形参、primary constructor、dot shorthand）与 479 文件抽样中，Dart 1
  下 173 个有效、Dart 2.19 下 477 个有效，cure 后均 **0 新增失败**；
  精选程序在 Dart 1/2/3 上 stdout 一致；语句级 fuzzer（35 函数 × 3 输入
  × 10 seed，Dart 1/2/3）0 失配，另有规则定向 fuzzer、类型化表达式
  fuzzer 与 Dart 3 pattern fuzzer 各 10–20 seed；全语料 2.2 s / ~217 MiB，
  执行 2,132 次安全改写（7,120 个文件变化，418 个结构简化）。
- **性能**：Dart 词法器不再复制整个文件为 `Vec<char>`，token 文本改用
  `Box<str>`，括号配对索引压到 `u32`（4 B/节点）、花括号分类压到
  `u8`；模式上下文改为每轮一次 O(n) 掩码（替代逐 token 回扫），
  `--check` 全量约 1.3 s（禁用 11 条新规则约 1.1 s），全量写出
  19,176 文件约 2.2 s / ~217 MiB。
- **测试修复的语义问题**：switch 表达式 pattern（`pattern => value`）
  的 `&&`/`||` 曾按布尔运算折叠；Dart 的 `&`/`|`/`^` 与 `==` 的相对
  优先级曾按 C 处理；`x = x ?? y` → `x ??= y` 会跳过 setter；
  `true && x`、`x && true` 等对动态非 bool 值会丢掉运行时类型检查；
  带注释的函数尾 `return;` 可能是 dart2js 内联屏障；`x = x * 10 + digit`
  曾被折成 `x *= 10 + digit`；三元规则曾跨过 `??=`/`+=` 把 `!` 加到
  赋值目标外；`if` + return 曾在无括号 else 分支里吞掉外层的
  `return`；`if`/else 两分支表达式类型不同时三元 LUB 会退化为
  `Object`（shelf_static 的 `Response`/`Future<Response>`）；去掉无括号
  else/循环体里的 `else` 会让裸块逃出原作用域（stream_channel）；
  不可达赋值被删会让 late 变量变成 definitely unassigned
  （_fe_analyzer_shared for_each）；把 `1 > 2` 之类分析器不折叠的条件
  改成 `false`、或三元选分支时丢掉含赋值的分支，都会改变 late 变量的
  definite assignment；`${x}` 后紧跟组合符号时压缩成 `$x́` 会改变标识符。
  均已加守卫并补回归测试；`// ignore:` 注释的独立行/行尾位置也由打印器
  保持。
- **真实包测试**：用离线 pub 缓存对 50+ 个纯 Dart 包运行原始/处理后
  的 `dart test`（petitparser 1,602、xml 5,876、characters 17,520、
  collection 1,706、yaml_edit 1,052、crypto 822 等，约 3 万条测试），
  以上 convert/collection/glob/shelf_static/stream_channel 问题均由
  此发现并修复；修复后各包通过/失败集合与原包一致。
- **文档/发布**：`cure rules` 汇总为 **118** 条（67 Java + 27 Python + 24
  Dart）；README（中英）增加 Dart 后端、规则与验证；`publish.sh` 扩展到
  19 个 crate。

### 修复与增强（四语言，2026-10-26）

- **TypeScript/JavaScript 解析器**：
  - 类声明与类表达式的类型参数默认值可以是对象类型字面量：
    `class A<T extends object = {}>`、`const C = class<T = {}> { … }`
    曾把默认值里的 `{}` 当成类体，`>` 报语法错误并把后续成员挤乱；
    类头扫描现在跟踪尖括号深度。
  - 匿名泛型类表达式（`class<T> {}`）不再被当成标识符 `class`。
  - 函数体里的 TS 重载签名（`function f(a: string): string;`）曾按
    “缺少函数体”报错并合成空块 `{}`，把签名变成第二个实现；现在整条
    原文保留。
  - JSX 补齐：`<>…</>` 片段、`{...props}` 展开属性、`{/* 注释 */}`
    空表达式容器不再报语法错误。
  - 泛型装饰器 `@Deco<T>()`、`@ns.Wrap<T>(x)`、`@Plain<T>` 的类型实参
    曾被留在装饰器前缀之外，`<T>()` 变成独立语句、装饰器整体丢失
    （类不再被装饰）；现在整条装饰器（含类型实参与调用）原文保留。
    `@Deco() export default class …` / `@Deco() export class …` 的
    装饰器前缀也不再被 export 分支丢弃。
  - `.ts/.mts/.cts` 里的尖括号类型断言 `<Foo<Bar>>z` 曾被当成 JSX
    元素一路吞到语句边界（后续语句并进 Raw、并多打一个 `;`）；现在按
    TS 语义解析并打印成等价的 `z as Foo<Bar>`（在 `.tsx` 里也合法），
    `.tsx/.jsx` 仍按 JSX 原文捕获。
- **TypeScript 类型实参保真**：`f<number>(1)`、`new Box<string>(…)`、
    `o.m<T>?.(x)`、`f<T>` 实例化表达式与 ``tag<T>`…` `` 的类型实参
    以前被直接吞掉（运行时擦除，但类型层可见——`useState<T|null>(null)`
    会退化成 `useState(null)` 并推断成 `null`）。现在包成 `TypeArgs`
    原文节点逐字节打印，并标记为不透明区域（不参与表达式化简）。
- **TypeScript 打印器**：类体的 `}` 不再粘在最后一个成员后
  （`constructor() {}}`）；无函数体签名不再多打一个空格
  （`string ;`）。
- **Dart 打印器**：顶层声明体 `}` 后必换行（`class A {}` 后紧跟记录
  返回类型 `(int, String) f()` 曾粘成 `}(int, …)`）；`final (i, v)`、
  `case [_, ...]`、`yield* [x]`、对象模式 `(:final value)` 的间距；
  switch 表达式每个分支独占一行。
- **Dart 新规则**（23 → 24）：`dart_trailing_assign_return`
  （`final y = expr; return y;` → `return expr;`；要求该局部名在块内
  只出现这两处，初始值限定为上下文不敏感形态，避免返回位置推断改变
  具化泛型类型；`const`/`late`/带类型注解/多声明器/调用/集合字面量
  均跳过）。
- **Python 新规则**（25 → 27）：
  - `py_double_not`：`not not x` → `bool(x)`（两者都只调用一次真值
    协议并返回真正的 bool；`bool` 被遮蔽时不改写）。
  - `py_trailing_assign_return`：`y = expr; return y` → `return expr`；
    要求该名字在函数内只被这条 return 读取（嵌套闭包、`global`/
    `nonlocal`、`del` 与 raw 片段引用都计入计数），且函数没有
    `locals()`/`vars()`/`dir()`/`eval()`/`exec()` 反射。
- **验证**：workspace 全测试通过；Dart pub-cache 当前树 19,176 文件
  0 解析诊断、0 token 漂移、格式往返 0 漂移、二次运行 0 字节差异，
  `dart format` 仍在同一批 90 个旧语法文件上失败（0 新增）；Python
  3.14 stdlib 2,456 文件 0 解析诊断、编译失败集合与原树完全一致
  （同一批 8 个故意非法测试文件）；JS 语料 200 文件 0 诊断、二次运行
  0 差异、全部产物 `node --check` 通过；Java 语料三检与新增
  records/pattern-switch 特性矩阵、编译/运行差分均通过。

## [0.2.0] - 2026-10-26

### 新增（Python 2.7 + 3.x 后端）

- **新语言后端**：新增 `cure-python-ast` / `cure-python-parser` /
  `cure-python-print` / `cure-python-simplify` 四个 crate 并接入 CLI；
  `--lang py`（别名 `python`/`python3`）与 `.py`/`.pyi` 自动识别，目录模式
  按所选后端的后缀过滤；`--check`、`--diff`、`--report`、`--stats`、
  `--disable`、`-w`、`-j` 全部可用。
- **Python 2 语法**：print/exec/raise 语句、反引号 `` `x` ``、`<>`、
  long/旧式八进制字面量（`1L`、`0777L`）、`ur` 字符串、元组形参与
  lambda 元组参数、`except E, e`、无括号推导式元组
  （`[x for x in a, b]`）；`print`/`exec` 在新旧版本语义不同，歧义形态
  （`print -1 + 2`、`print[1,2]`、`print (x), y`、任意 `exec`）整行原文
  保留，两种解释器都保持原语义。
- **Python 2 语义守卫**：涉及 `1L` 的算术不折成 int（类型可观测）；
  模块内出现 `True =`/`False =`、元组目标、for 目标、`as`、def/lambda
  形参、class/import/global/del 等绑定时，True/False 按名字处理，
  不再折叠条件；非 UTF-8 源码（PEP 263）CLI 跳过并在诊断中说明。
- **解析器**：缩进栈（空格 + tab×8）、括号内隐式续行、反斜杠续行、
  三引号/raw/bytes/f-string；PEP 701 嵌套插值与深度 0 `:` 后的格式说明
  （`f"{x:#x}"` 不再被误当注释）；t-string；soft keyword `match`
  （`match = re.match(...)`、`match: T = ...` 仍按普通语句）；`case`、
  `except*`、PEP 695 `def f[T]`/`class C[T]`、`lazy import`、下标与列表中
  的海象运算符、lambda 默认值中的嵌套 lambda、尾逗号元组、切片列表
  （`a[:, 1]`）与显式元组区分。无法解析的区域保留原文，周边继续处理。
- **打印器**：4 空格规范缩进、elif 链、raw 语句续行保留源绝对缩进
  （三引号字符串内容不被重排）、装饰器续行按 token 感知重排（字符串
  内部不动）、参数表末行带注释时 `)` 换行、切片尾冒号（`a[0:2:]`）与
  下标元组（`a[..., ...]`）保留、`Paren(NamedExpr)` 不重复加括号。
- **规则（25 条）**：
  - `py_const_fold`：整数/布尔/字符串字面量、字符串 `+`/`*`、比较、真值、
    `and`/`or` 操作数语义；i128 计算/i64 回验，long、不精确大整数与
    不确定浮点不折。
  - `py_call_fold`：字面量 str 的 upper/lower/swapcase/capitalize/strip/
    replace/startswith/endswith/join，以及防遮蔽的 len/ord/chr/str/int/
    abs/bool/min/max（赋值、形参、import、for/with/except 目标、`exec`、
    `import *`、builtins/`__builtins__` 都会关闭对应折叠）。
  - `py_subscript_fold`：字面量字符串/列表/元组的下标与切片（越界、
    step 0、非 ASCII/转义字符串不折）。
  - 控制流/结构：`py_const_if` / `py_if_ternary` / `py_if_return`（相邻
    `if c: return a` 与 `return b` 合成 `return a if c else b`）/
    `py_if_empty`（`else: pass` 与 pass-only 分支取反）/ `py_nested_if` /
    `py_else_after_terminal` / `py_bool_cond`（`True if c else False` ↔
    `bool(c)`）/ `py_ternary_not` / `py_not_is_in`（`not (x is y)` →
    `x is not y`，含括号剥离）/ `py_paren_removal`（原子表达式外的冗余
    括号；多行字符串与可能改变优先级的形态不动）。
  - 数据流/清理：`py_dead_store` / `py_dead_assign`（函数作用域活跃性：
    未被读取的局部纯值可删，有副作用的右值降为表达式；遇到
    locals/vars/dir/eval/exec 反射、global/nonlocal 时关闭）/
    `py_dead_expr` / `py_str_accum`（相邻字符串累加折叠）/ `py_assign_back_fold`
    （`x = 2; x = x + 3` → `x = 5`，仅纯 int/字符串链）/ `py_return_none` /
    `py_trailing_return` / `py_trailing_continue` / `py_empty_finally` /
    `py_assert_true` / `py_unreachable` / `py_pass_cleanup`。
  - 语义守卫：docstring 不会被折叠/内联创建（必要时保留 `pass`）；
    `yield`/`global`/`nonlocal` 不因分支删除而丢失；Python 2 int/long、
    True/False 重绑定（含简化器自身不再产出被遮蔽的 True/False）、
    print/exec 歧义形态逐字节保留。
- **解析修复（真实语料抓获）**：类字段注解中的 `Field(min_length=1)` 等
  方括号内 `=`；raw f-string 的 `\{{` 花括号；多装饰器/2 空格缩进/
  三引号参数；`1else` 指数 tokenizer；Python 2 推导式 lambda 条件与
  未括号元组 iterable；`0b1and` 等按进制停止的数字扫描。

### 变更

- `--check` 与统计中的「改写」改为按**输出文本**判定：此前按规则编辑
  次数计算，已收敛文件可能因打印不可见的 AST 归一被误报为可改写；
- 目录模式的输出路径提示改为实际写入根（此前显示成输入目录的兄弟路径）；
- `cure rules` 标题汇总 Java + Python 规则数，并明确 Java / Python
  分组；`--lang` 帮助补上 `pyi`，示例增加 Python 用法；
- README（中英）规则清单补齐 33 条 Java 专属规则并修正引擎规则数，
  同步性能/测试数字；RELEASE.md 更新为 15 个 crate；
- TS/Python 子 crate 增加 crates.io README 与 keywords/categories；
  `.gitattributes` 将 Java 测试语料标记为 vendored，GitHub 语言统计
  聚焦 Rust 代码。

### 验证

- **Python 2.7**：标准库 1,609 个 UTF-8 源文件与 4,135 文件的真实包语料
  （Django 1.11、Twisted 20.3、pip、setuptools 44、requests、six、PyYAML）
  均 **0 解析诊断**；**1,607 + 4,129** 解析 → 打印往返在 CPython 2.7
  `ast.dump` 下等价（其余为故意非法/异构夹具）；5,744 个文件全部幂等；
  原本可编译的 **1,600 + 4,129** 个产物在简化后仍全部可编译（0 破坏）。
- **Python 3**：3.13/3.14/3.15 标准库与 3.14 site-packages（18,128 文件）
  合计 **0 诊断、26,553 个 AST 等价往返、0 幂等漂移、0 编译破坏**；
  3.13/3.15 site（1,083 / 422 文件）全绿；3.6/3.8 标准库（1,578 / 1,661
  文件）0 诊断且幂等。
- 全部语料合计 **32,289 次 AST 比较、35,548 个幂等文件、32,274 次编译、
  0 破坏**；简化器执行 **179,873 次改写**（Py2 标准库 7,833、Py2 包
  6,763、Py3.15 标准库 12,098、Py3.14 site-packages 111,340，其余
  3.13/3.14/3.6/3.8 语料 41,839），docstring 与类型语义保持；目录二次
  运行均为 **0 编辑**、逐字节一致。
- 精选 Python 2/3 程序（print/exec/raise、反引号、`<>`、long/int 类型、
  元组形参、装饰器三引号参数）与 CPython 对拍 stdout/退出码一致；
  表达式差分（4,056 个）和新增的语句级差分（800 个函数 × 4 组输入，
  Python 2.7 与 3.15 各 6,400 个结果）全部一致；
  `cargo test --workspace` **523 通过**（含 Python 2/边界/简化回归）。

## [0.1.11] - 2026-10-25

### 修复（读分析健全性——2026-10-25 第十八批）

- **赋值目标读**：成员/索引目标的基座与键是读（`a[i] = v` 读 `a`、`i`），
  复合赋值与 `++`/`--` 读旧值。修复两条扫描路径
  （`scan_kill_event` / `collect_stmt_kill_events`）、cure-tree 区域事件
  索引与 `Watch` 递归路径。此前 `i = 0; a[i] = 1; i = 2;` 会把前一条
  `i` 存储当成被支配的死写删除（jscd 寄存器风格高频命中）。
- **DeadStore 原文 RHS**：相邻 `x = a; x = b` 折叠前检查 `b` 是否含
  Raw/原文区域；`x = yield x` 这类 AST 不可见的读保守拒绝，避免删掉
  仍被读取的前一条存储。
- **jscd 集成**：cure 0.1.11 默认规则（无需 `--disable store_kill
  --disable dead_store`）在 jscd 0.2.2 的 call_once / string_regex /
  closure fixture 上与 jscd 原输出语义一致；测试覆盖索引写入目标与
  `yield` 生成器。
- 新增 JS/Java 差分回归；`cargo test --workspace` **474 通过**。

## [0.1.10] - 2026-10-25

### 兼容（复杂 TypeScript 类型语法——2026-10-25 第十七批）

- **类型声明原文保真扩展**：`type X<T> = …`（泛型参数）、`export type/
  interface/enum/declare/namespace` 整体捕获到语句边界；多行 union /
  conditional / mapped types 不再在首个换行处被拆散。
- **类型注解扫描**：变量/解构/类字段注解按 `{…}`/`[…]`/`<…>`/`=>`
  深度扫描（`T[K]`、`Record<string, any>[]`、`Promise<{ [K in keyof T]:
  T[K] }>`）；无分号 ambient 声明与下一成员/声明按行首形态终止。
- **返回类型**：函数声明/函数表达式/箭头/类方法/对象方法支持对象、
  元组、函数、条件类型；箭头返回类型前瞻允许裸 `{…}`；`as` 类型
  在深度 0 的 `?`/`:` 处终止（修复三元分支被吞成类型）。
- **泛型调用/new**：`f<T>(x)`、`o.m<T>(x)`、`new C<T>()`、`new C<T>`、
  实例化表达式 `f<T>` 擦除类型实参；仅在 TS 模式启用——`.js/.mjs/
  .cjs` 下 `a < b > (c)` 仍按比较链解析（`ParseOptions.ts_syntax`）。
- **类成员**：明确赋值 `field!: T`、可选 `field?: T`/`m?()`、泛型方法、
  无体 overload（含对象返回类型）。
- **参数列表判定**：剩余参数 `...args`、参数类型中的函数类型
  `cb: () => T`、泛型/元组类型中的 `|`/`&` 不再误判为非参数列表
  （曾导致类成员吞到 EOF）。
- 修复 `yield` 同行模板操作数：`yield \`…\`` 之前因模板 token 被
  排除在操作数之外而拆成两条语句（真实 npm 语料 `@tufjs/models`
  发生器方法抓获）；模板续接段 `\`…${yield}3…\`` 仍按外层模板处理。
- 修复压缩 JS 的三类解析问题：类声明后无分号/换行直接跟下一个声明
  （`}class B{}let x`）；`(v=1) : cond ? … => …` 被误判为带返回类型
  的箭头（返回类型扫描在深度 0 遇 `=` 即放弃）。
- 新增 `tests/typescript_types.rs`（5 组：复杂类型声明、泛型调用/new、
  类型化函数/箭头、`as` 三元、JS 比较链守卫）。
- 回归：全局 npm **1,334 个 TS/TSX/MTS/CTS 文件 0 解析诊断、两轮输出
  0 差异**，cure 产物经 Oxc `oxfmt` 解析 **0 语法错误**；JS 语料
  **4,358 个 .js/.mjs/.cjs 文件 0 解析诊断、两模式两轮 0 差异**且
  Oxc 0 语法错误；Test262 六层全绿；`cargo test --workspace`
  **471 通过**。

### 新增（函数源码保真模式——2026-10-25 第十六批）

- **`--preserve-function-source`**（库侧 `Config::preserve_function_source`
  + `TsAst::set_preserve_source` + `cure_ts_parser::parse_with_source`）：
  TS/JS 的函数/类/对象字面量按源字节原样打印——注释、空白、CRLF 全部
  保留，`Function.prototype.toString()` 与原文逐字节一致。保真区内部
  不做简化；**从保真区引用的绑定被冻结**（不传播、不删除），模块其余
  顶层代码仍正常简化。默认关闭。
- 打印层：`Lambda`/`ObjLit`/`FuncStmt`/函数声明/类声明在保真模式下走
  源码区间输出；区间按 token 字节偏移记录，CR 序列不归一化（Test262
  `line-terminator-normalisation-*` 要求原样返回）。
- 解析器：新增 `parse_with_source` / `parse_module_with_source` /
  `parse_script_with_source`，仅在这些入口记录源码与区间，默认解析
  零额外内存。
- 修复 `if_else_empty`：`if (c) {} else {B}` → `if (!c) {B}` 的代价
  可能恰好持平（空 Block + 条件 vs 一元 `!`），debug 断言曾
  `panic: non-simplifying edit`；该规则标记为 structural 结构改写。
- 新增 `tests/preserve_source.rs`（4 组：toString 逐字节、对象简写
  绑定不被传播删除、CRLF 保留、默认模式仍重排）+ 三个 Test262 驱动
  的 `T262_PRESERVE=1` 模式。
- 回归：**保真模式同步语义差分 34,007/34,007（100%，0 失配、0 豁免）**；
  默认模式保持 33,963/34,007（44 个 toString 用例为已知重排差异，0 失配）；
  `cargo test --workspace` **463 通过**。

### 正确性（随机差分扩展——2026-10-24 第十五批）

- 随机差分扩到 **5,000 例**，又抓到两个类型域错误：
  - `ternary_bool_op`：`c ? true : b ≡ c || b` 仅当 `c` 是语法级布尔；
    数值条件 `1 ? true : false` 得 `true`，而 `1 || false` 得 `1`。
    增加 `is_bool(c)` 守卫（case 2074）。
  - `double_neg_fold`：`-(-x) ≡ x` 仅对精确整数成立；布尔/字符串/`null`
    会被 ToNumber（`-(-true)` = 1），浮点 `-0` 也会变号。增加
    `is_exact_int(x)` 守卫（case 3157）。
- 新增两个差分回归用例（数值条件三元、非数值双重取负）。
- 回归：同步 33,963/34,007（44 allowlist，0 失配）、async 5,423、
  module 358、编译 49,165 valid / 0 破坏、往返/幂等各 49,159；
  npm 4,358 文件 **0 broken / 0 二次差异**；`cargo test --workspace`
  **458 通过**。

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

[Unreleased]: https://github.com/ejfkdev/cure/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/ejfkdev/cure/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ejfkdev/cure/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/ejfkdev/cure/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ejfkdev/cure/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ejfkdev/cure/releases/tag/v0.2.0
[0.1.11]: https://github.com/ejfkdev/cure/releases/tag/v0.1.11
[0.1.10]: https://github.com/ejfkdev/cure/releases/tag/v0.1.10
[0.1.0]: https://github.com/ejfkdev/cure/releases/tag/v0.1.0
