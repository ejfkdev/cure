//! # cure-ts-parser
//!
//! TypeScript/JavaScript 容错解析器：lexer（[`lexer`]）+ 递归下降语法。
//!
//! 策略：
//! - **JS 值级语法**完整建模进 [`cure_ts_ast`]（引擎可简化）；
//! - **TS 类型/装饰器**以原文区间捕获（ty_raw/sig_raw——保真打印）；
//! - **JSX / 类表达式**整体原文捕获（`Raw` 表达式节点——不透明但保真）；
//! - **容错**：语句级失败回退为原文 `Raw` 节点（同步点：`;`/换行边界/
//!   `}`/EOF），解析错误永不中止整个文件——与 Java 后端同契约。
//! - **ASI**：`newline_before` 驱动的隐式分号 + restricted production
//!   （return/throw/break/continue/后缀 ++--）。

mod lexer;

use cure_engine::kind::{BinOp, UnOp};
use cure_ts_ast::{
    ClassDecl, ClassMember, JsLit, ModuleUnit, NodeData, NumVal, ParseError, TopItem,
    TsAst, TsId, VarKind,
};
pub use lexer::{LexError, Lexer, TokKind, Token};

pub struct ParseOutcome {
    pub ast: TsAst,
    pub unit: ModuleUnit,
    pub errors: Vec<ParseError>,
}

/// 解析配置。
#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    /// `Some(true)` = ESM，`Some(false)` = script，`None` = 按顶层
    /// import/export 自动判定。
    pub module_goal: Option<bool>,
    /// 记录函数/类/对象字面量源码区间（preserve_function_source）。
    pub capture_source: bool,
    /// 允许 TS 类型语法。仅影响有歧义的形态（泛型调用 `<T>(…)` /
    /// `new C<T>()` / 实例化表达式）；JS 文件应关闭，避免把
    /// `a < b > (c)` 误读成泛型调用。
    pub ts_syntax: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            module_goal: None,
            capture_source: false,
            ts_syntax: true,
        }
    }
}

/// 按显式配置解析（CLI 依扩展名选择 TS/JS 与 script/module）。
pub fn parse_with_options(src: &str, opts: ParseOptions) -> ParseOutcome {
    parse_inner(src, opts.module_goal, opts.capture_source, opts.ts_syntax)
}

/// JS 语义解析（关闭有歧义的 TS 泛型形态；Test262/npm 语料用）。
pub fn parse_js(src: &str) -> ParseOutcome {
    parse_with_options(src, ParseOptions { ts_syntax: false, ..Default::default() })
}

pub fn parse_js_module(src: &str) -> ParseOutcome {
    parse_with_options(
        src,
        ParseOptions { module_goal: Some(true), ts_syntax: false, ..Default::default() },
    )
}

pub fn parse_js_script(src: &str) -> ParseOutcome {
    parse_with_options(
        src,
        ParseOptions { module_goal: Some(false), ts_syntax: false, ..Default::default() },
    )
}

/// JS 语义 + 记录源码区间（保真模式用于 Test262/npm 的 `.js` 语料）。
pub fn parse_js_with_source(src: &str) -> ParseOutcome {
    parse_with_options(
        src,
        ParseOptions { capture_source: true, ts_syntax: false, ..Default::default() },
    )
}

pub fn parse_js_module_with_source(src: &str) -> ParseOutcome {
    parse_with_options(
        src,
        ParseOptions {
            module_goal: Some(true),
            capture_source: true,
            ts_syntax: false,
        },
    )
}

pub fn parse_js_script_with_source(src: &str) -> ParseOutcome {
    parse_with_options(
        src,
        ParseOptions {
            module_goal: Some(false),
            capture_source: true,
            ts_syntax: false,
        },
    )
}

/// 解析源文件（容错）。自动判定 script/module：顶层 `import`/`export`
/// 视为 module（顶层 await 可用）；否则按 script（`await` 可作标识符）。
///
/// 默认允许 TS 歧义形态（泛型调用 `f<T>(x)` / `new C<T>()`）。**纯 JS
/// 输入请用 [`parse_js`]**——否则 `a < b > (c)` 会被当作泛型调用。
pub fn parse(src: &str) -> ParseOutcome {
    parse_inner(src, None, false, true)
}

/// 按 module 目标解析（ESM：顶层 await 是运算符）。
pub fn parse_module(src: &str) -> ParseOutcome {
    parse_inner(src, Some(true), false, true)
}

/// 按 script 目标解析（顶层 await 按标识符/普通名）。
pub fn parse_script(src: &str) -> ParseOutcome {
    parse_inner(src, Some(false), false, true)
}

/// 解析并记录函数/类/对象字面量的源码区间（供
/// `preserve_function_source` 原样打印）。默认解析不承担这份内存。
pub fn parse_with_source(src: &str) -> ParseOutcome {
    parse_inner(src, None, true, true)
}

/// [`parse_with_source`] 的 module 目标版本。
pub fn parse_module_with_source(src: &str) -> ParseOutcome {
    parse_inner(src, Some(true), true, true)
}

/// [`parse_with_source`] 的 script 目标版本。
pub fn parse_script_with_source(src: &str) -> ParseOutcome {
    parse_inner(src, Some(false), true, true)
}

fn parse_inner(
    src: &str,
    module_goal: Option<bool>,
    capture_source: bool,
    ts_syntax: bool,
) -> ParseOutcome {
    let mut shebang = String::new();
    let body = if let Some(rest) = src.strip_prefix("#!") {
        if let Some(nl) = rest.find('\n') {
            shebang = src[..2 + nl].to_string();
            &rest[nl + 1..]
        } else {
            shebang = src.to_string();
            ""
        }
    } else {
        src
    };
    let (toks, lex_errs) = Lexer::new(body).tokenize();
    let module_goal = module_goal.unwrap_or_else(|| detect_module_goal(&toks));
    let mut p = Parser {
        src: body,
        toks,
        pos: 0,
        ast: TsAst::new(),
        unit: ModuleUnit { shebang, items: Vec::new() },
        errors: Vec::new(),
        async_ctx: module_goal,
        depth: 0,
        too_deep: false,
        capture_source,
        ts_syntax,
    };
    if capture_source {
        p.ast.set_source(body);
    }
    for e in lex_errs {
        p.errors.push(ParseError {
            line: e.line,
            col: e.col,
            message: e.message,
        });
    }
    p.parse_module();
    if p.too_deep {
        // 部分 AST 已不可信：整文件原文保真（含注释/格式），
        // 后续 simplify 对 TopItem::Raw 不做任何改写。
        p.unit.items.clear();
        p.unit.items.push(TopItem::Raw(body.to_string()));
        p.errors.clear();
        p.errors.push(ParseError {
            line: 1,
            col: 1,
            message: "nesting too deep; preserved verbatim".into(),
        });
    }
    ParseOutcome {
        ast: p.ast,
        unit: p.unit,
        errors: p.errors,
    }
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    ast: TsAst,
    unit: ModuleUnit,
    errors: Vec<ParseError>,
    /// 当前函数体/[Await] 语境：async 函数/async 方法/async 箭头/
    /// module 顶层为 true；普通函数边界重置为 false（箭头不继承外层
    /// async——`() => await x` 在 async 函数里也是语法错误）。
    async_ctx: bool,
    /// 递归嵌套深度（语句/表达式/一元链）。超过 `MAX_NEST` 时不再深入，
    /// 整个文件走原文保真（避免恶意/生成代码的栈溢出）。
    depth: usize,
    too_deep: bool,
    /// 记录节点源码区间（preserve_function_source）。
    capture_source: bool,
    /// 允许 TS 歧义形态（泛型调用/`new` 类型实参）。
    ts_syntax: bool,
}

/// 递归上限。正常源码远低于此；超出者整文件 Raw 保真。
const MAX_NEST: usize = 512;

/// 顶层 `import`/`export` → module 目标。只扫花括号/圆/方括号深度 0；
/// `import(`/`import.meta` 是表达式，不算声明。
fn detect_module_goal(toks: &[Token]) -> bool {
    let mut depth = 0i32;
    for (i, t) in toks.iter().enumerate() {
        if t.kind == TokKind::Punct {
            match t.text.as_str() {
                "{" | "(" | "[" => depth += 1,
                "}" | ")" | "]" => depth -= 1,
                _ => {}
            }
            continue;
        }
        if depth != 0 || t.kind != TokKind::Ident {
            continue;
        }
        if t.text == "export" {
            return true;
        }
        if t.text == "import" {
            let next = toks.get(i + 1);
            let prev_dot = i > 0 && toks[i - 1].kind == TokKind::Punct && toks[i - 1].text == ".";
            if prev_dot {
                continue;
            }
            if let Some(n) = next {
                if n.kind == TokKind::Ident || n.kind == TokKind::Str {
                    return true;
                }
                if n.kind == TokKind::Punct && matches!(n.text.as_str(), "{" | "*") {
                    return true;
                }
            }
        }
    }
    false
}

/// 二元运算符表：`(优先级, BinOp)`。`??` 与 `||` 同层（混合需括号——
/// 解析器宽容接受，引擎/打印保真）。
fn bin_info(op: &str) -> Option<(u8, BinOp)> {
    use BinOp::*;
    Some(match op {
        "??" => (2, Nullish),
        "||" => (2, Or),
        "&&" => (3, And),
        "|" => (4, BitOr),
        "^" => (5, BitXor),
        "&" => (6, BitAnd),
        "==" => (7, Eq),
        "!=" => (7, Ne),
        "===" => (7, StrictEq),
        "!==" => (7, StrictNe),
        "<" => (8, Lt),
        ">" => (8, Gt),
        "<=" => (8, Le),
        ">=" => (8, Ge),
        "in" => (8, In),
        "instanceof" => (8, Instanceof),
        "<<" => (9, Shl),
        ">>" => (9, Shr),
        ">>>" => (9, UShr),
        "+" => (10, Add),
        "-" => (10, Sub),
        "*" => (11, Mul),
        "/" => (11, Div),
        "%" => (11, Rem),
        "**" => (12, Pow),
        _ => return None,
    })
}

fn assign_op_of(punct: &str) -> Option<BinOp> {
    use BinOp::*;
    Some(match punct {
        "+=" => Add,
        "-=" => Sub,
        "*=" => Mul,
        "/=" => Div,
        "%=" => Rem,
        "**=" => Pow,
        "<<=" => Shl,
        ">>=" => Shr,
        ">>>=" => UShr,
        "&=" => BitAnd,
        "|=" => BitOr,
        "^=" => BitXor,
        "&&=" => And,
        "||=" => Or,
        "??=" => Nullish,
        _ => return None,
    })
}

const STMT_KW: &[&str] = &[
    "var", "let", "const", "if", "else", "while", "do", "for", "return", "break", "continue",
    "throw", "try", "switch", "function", "class", "import", "export", "with", "debugger",
    "using", "enum", "interface", "namespace", "declare", "type", "abstract",
];

impl<'a> Parser<'a> {
    // ---- token 工具 ----

    fn tk(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn tk_at(&self, off: usize) -> &Token {
        &self.toks[(self.pos + off).min(self.toks.len() - 1)]
    }
    fn is_punct(&self, p: &str) -> bool {
        self.tk().kind == TokKind::Punct && self.tk().text == p
    }
    fn is_punct_at(&self, off: usize, p: &str) -> bool {
        let t = self.tk_at(off);
        t.kind == TokKind::Punct && t.text == p
    }
    fn is_kw(&self, w: &str) -> bool {
        self.tk().kind == TokKind::Ident && self.tk().text == w
    }
    fn text(&self) -> &str {
        &self.tk().text
    }
    fn at_eof(&self) -> bool {
        self.tk().kind == TokKind::Eof
    }
    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos.min(self.toks.len() - 1)].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn err_here(&mut self, msg: &str) {
        let t = self.tk().clone();
        self.errors.push(ParseError {
            line: t.line as usize,
            col: t.col as usize,
            message: msg.to_string(),
        });
    }
    /// token 下标区间 [a, b) 的源码原文。b ≤ a（空区间）→ 空串
    ///（trivia 使 toks[b-1].end < toks[a].start——直接切片会越界）。
    fn raw_between(&self, a: usize, b: usize) -> String {
        if b <= a || a >= self.toks.len() {
            return String::new();
        }
        let s = self.toks[a].start as usize;
        let e = if b <= self.toks.len() {
            self.toks[b - 1].end as usize
        } else {
            self.src.len()
        };
        if e <= s {
            return String::new();
        }
        let src = self.src;
        let mut s = s.min(src.len());
        let mut e = e.min(src.len());
        // 字符边界对齐（防御：偏移来自异常路径时可能落在多字节
        // 字符中间——from_utf8_lossy 兜底而不是 panic）
        while s < e && !src.is_char_boundary(s) {
            s += 1;
        }
        while e > s && !src.is_char_boundary(e) {
            e -= 1;
        }
        src[s..e].to_string()
    }

    // ---- 模块层 ----

    fn parse_module(&mut self) {
        while !self.at_eof() {
            let before = self.pos;
            self.parse_top_item();
            if self.pos == before {
                self.advance();
            }
        }
    }

    fn parse_top_item(&mut self) {
        // 装饰器前缀（捕获进头部原文）
        let mut deco_raw = String::new();
        while self.is_punct("@") {
            let start = self.pos;
            self.skip_decorator();
            // 装饰器后的实体在同一「语句」——扫到边界会吃过头；
            // 只跳装饰器本身
            deco_raw.push_str(&self.raw_between(start, self.pos));
            deco_raw.push(' ');
        }
        // 仅**静态** import 语句走原文捕获；`import(...)`（动态导入）
        // 与 `import.meta` 是表达式（曾有 `.then` 被拆成残句——
        // Test262 dynamic-import/import-attributes 家族抓获）。
        if self.is_kw("import")
            && !self.is_punct_at(1, "(")
            && !self.is_punct_at(1, ".")
        {
            let raw = self.capture_module_stmt();
            self.unit.items.push(TopItem::Import(raw));
            return;
        }
        if self.is_kw("export") {
            self.parse_export(deco_raw);
            return;
        }
        // TS 上下文关键字只有在声明形态下才按原文整条捕获：
        // `module.exports = …` / `namespace.foo()` / `interface = 1` 都是
        // 普通标识符表达式（曾把 CommonJS 入口 `module.exports = fn()`
        // 截成 `module.exports =` 残句 + 匿名函数声明；pngjs 等真实库
        // 编译差分抓获）。
        let kw_decl_shape = (self.is_kw("module") || self.is_kw("namespace"))
            && (self.tk_at(1).kind == TokKind::Ident
                || self.tk_at(1).kind == TokKind::Str)
            || (self.is_kw("interface") && self.tk_at(1).kind == TokKind::Ident)
            || (self.is_kw("declare")
                && matches!(
                    self.tk_at(1).text.as_str(),
                    "var" | "let" | "const" | "function" | "class" | "module" | "namespace"
                        | "interface" | "type" | "abstract" | "enum" | "global"
                ));
        if kw_decl_shape {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if self.type_decl_shape() {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if self.is_kw("enum") || (self.is_kw("const") && self.tk_at(1).text == "enum") {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if let Some(f) = self.try_parse_function_decl(deco_raw.clone(), false) {
            self.unit.items.push(f);
            return;
        }
        if self.is_kw("class") || (self.is_kw("abstract") && self.tk_at(1).text == "class") {
            if let Some(c) = self.try_parse_class_decl(deco_raw) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
        }
        let s = self.parse_stmt();
        self.unit.items.push(TopItem::Stmt(s));
    }

    fn parse_export(&mut self, deco_raw: String) {
        let kw_pos = self.pos;
        self.advance(); // export
        if self.is_kw("default") {
            self.advance();
            // `export default function [name](){}`——default 关键字随
            // sig 原文保留（匿名默认导出曾丢 default 变成非法
            // `export function() {}`）
            if let Some(f) = self.try_parse_function_decl("export default".into(), false) {
                self.unit.items.push(f);
                return;
            }
            if let Some(c) = self.try_parse_class_decl("export default".into()) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
            let e = self.parse_assign_expr();
            self.consume_semi();
            let _ = e;
            let raw = format!("export default {}", self.raw_between(kw_pos + 2, self.pos));
            self.unit.items.push(TopItem::ExportRaw(raw));
            return;
        }
        if self.is_kw("function")
            || (self.is_kw("async") && self.tk_at(1).text == "function")
        {
            if let Some(f) = self.try_parse_function_decl(deco_raw.clone(), true) {
                self.unit.items.push(f);
                return;
            }
        }
        // `export type/interface/enum/declare/namespace`：TS 类型声明
        // 必须整体原文捕获。类型别名/接口头可跨行（union、conditional、
        // mapped types），`capture_module_stmt_from` 遇首个换行就停——
        // 会把 zod 等真实文件的类型 RHS 拆成表达式语句。
        let ts_decl = self.is_kw("type")
            || self.is_kw("interface")
            || self.is_kw("enum")
            || (self.is_kw("const") && self.tk_at(1).text == "enum")
            || self.is_kw("declare")
            || ((self.is_kw("namespace") || self.is_kw("module"))
                && matches!(
                    self.tk_at(1).kind,
                    TokKind::Ident | TokKind::Str
                ));
        if ts_decl {
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(kw_pos, self.pos);
            self.unit.items.push(TopItem::ExportRaw(raw));
            return;
        }
        // export const/let/var/using/解构声明——真解析（值级可简化）
        if self.is_kw("var")
            || self.is_kw("let")
            || self.is_kw("const")
            || self.is_kw("using")
        {
            let stmt = self.parse_stmt();
            self.unit.items.push(TopItem::ExportStmt(stmt));
            return;
        }
        if self.is_kw("class") || (self.is_kw("abstract") && self.tk_at(1).text == "class") {
            let mut pre = deco_raw;
            pre.push_str("export");
            if let Some(c) = self.try_parse_class_decl(pre) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
        }
        let raw = self.capture_module_stmt_from(kw_pos);
        self.unit.items.push(TopItem::ExportRaw(raw));
    }

    /// import/export 语句原文捕获：支持行首 `with`/`assert` 导入属性
    /// 子句（ASI 在 `with` 前不生效）；其余行终止符按 ASI 结束语句。
    fn capture_module_stmt(&mut self) -> String {
        let start = self.pos;
        self.capture_module_stmt_from(start)
    }

    fn capture_module_stmt_from(&mut self, start: usize) -> String {
        let mut depth = 0i32;
        let mut first = true;
        // 导入属性子句（`with {…}`）内允许任意换行（ASI 不适用——
        // Test262 import-attribute-newlines 抓获：`with` 与 `{`/属性
        // 名/值之间都可换行）
        let mut attrs = false;
        loop {
            if self.at_eof() {
                break;
            }
            let t = self.tk().clone();
            if !first && depth == 0 {
                if t.kind == TokKind::Punct && t.text == ";" {
                    self.advance();
                    break;
                }
                if !attrs
                    && t.newline_before
                    && !(t.kind == TokKind::Ident
                        && matches!(t.text.as_str(), "with" | "assert"))
                {
                    break;
                }
            }
            first = false;
            if depth == 0
                && t.kind == TokKind::Ident
                && matches!(t.text.as_str(), "with" | "assert")
            {
                attrs = true;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "{" | "(" | "[" => depth += 1,
                    "}" | ")" | "]" => depth -= 1,
                    _ => {}
                }
            }
            self.advance();
        }
        self.raw_between(start, self.pos)
    }

    /// 平衡扫描到语句边界（`;`/同层 `}`/EOF；深度感知）。
    fn skip_balanced_to_stmt_boundary(&mut self) {
        let mut paren = 0i32;
        let mut brace = 0i32;
        let mut brack = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            let t = self.tk();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => paren += 1,
                    ")" => paren -= 1,
                    "[" => brack += 1,
                    "]" => brack -= 1,
                    "{" => brace += 1,
                    "}" => {
                        if brace == 0 && paren == 0 && brack == 0 {
                            return;
                        }
                        brace -= 1;
                    }
                    ";" if brace == 0 && paren == 0 && brack == 0 => {
                        self.advance();
                        return;
                    }
                    _ => {}
                }
            }
            if brace == 0 && paren == 0 && brack == 0 && self.tk_at(1).newline_before {
                let nt = self.tk_at(1).clone();
                let ends_stmt = match nt.kind {
                    TokKind::Eof => true,
                    TokKind::Punct => matches!(nt.text.as_str(), "}" | ")" | "]" | ";"),
                    TokKind::Ident => STMT_KW.contains(&nt.text.as_str()) && nt.text != "let",
                    _ => false,
                };
                if ends_stmt {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    // ---- 函数/类声明 ----

    fn try_parse_function_decl(&mut self, prefix_raw: String, exported: bool) -> Option<TopItem> {
        let start = self.pos;
        let mut is_async = false;
        if self.is_kw("async") && self.tk_at(1).text == "function" && !self.tk_at(1).newline_before {
            is_async = true;
            self.advance();
        }
        if !self.is_kw("function") {
            self.pos = start;
            return None;
        }
        let fn_start = if is_async { start } else { self.pos };
        self.advance();
        let mut is_generator = false;
        if self.is_punct("*") {
            is_generator = true;
            self.advance();
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        // TS 类型参数：function f<T extends U>(…) / function <T>(…)
        let mut generics = String::new();
        if self.is_punct("<") {
            let gs = self.pos;
            self.skip_angle_balanced();
            generics = self.raw_between(gs, self.pos).trim().to_string();
        }
        if !self.is_punct("(") {
            self.pos = start;
            return None;
        }
        let ps = self.pos;
        self.skip_parens();
        let params_full = self.raw_between(ps, self.pos);
        let mut ret_raw = String::new();
        if self.is_punct(":") {
            let rs = self.pos;
            self.skip_return_type();
            ret_raw = self.raw_between(rs, self.pos);
        }
        let body = if self.is_punct("{") {
            Some(self.enter_async(is_async, |p| p.parse_block()))
        } else {
            // declare/abstract 无体形态——原文回退
            if self.is_punct(";") {
                self.advance();
            }
            None
        };
        let mut sig = String::new();
        let mut decl_prefix = String::new();
        if !prefix_raw.is_empty() {
            sig.push_str(&prefix_raw);
            sig.push(' ');
            decl_prefix.push_str(&prefix_raw);
            decl_prefix.push(' ');
        }
        if exported {
            sig.push_str("export ");
            decl_prefix.push_str("export ");
        }
        if is_async {
            sig.push_str("async ");
        }
        sig.push_str("function");
        if is_generator {
            sig.push('*');
        }
        if !name.is_empty() {
            sig.push(' ');
            sig.push_str(&name);
        }
        if !generics.is_empty() {
            if name.is_empty() {
                sig.push(' ');
            }
            sig.push_str(&generics);
        }
        sig.push_str(&params_full);
        sig.push_str(&ret_raw);
        let raw_span = if body.is_some() {
            self.span_u32(fn_start)
        } else {
            None
        };
        Some(TopItem::Func { sig_raw: sig, name, body, prefix: decl_prefix, raw_span })
    }

    fn try_parse_class_decl(&mut self, deco_raw: String) -> Option<ClassDecl> {
        let start = self.pos;
        let mut prefix = String::new();
        if !deco_raw.is_empty() {
            prefix.push_str(deco_raw.trim_end());
            prefix.push(' ');
        }
        let mut head = String::new();
        let kind_kw: String;
        let class_start: usize;
        if self.is_kw("abstract") && self.tk_at(1).text == "class" {
            prefix.push_str("abstract ");
            self.advance();
        }
        if self.is_kw("class") {
            kind_kw = "class".into();
            class_start = self.pos;
            self.advance();
        } else {
            self.pos = start;
            return None;
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        let hs = self.pos;
        // 找类体 `{`：括号深度 0 且跳过 extends 里的嵌套类表达式
        //（`class B extends class { } { … }`——曾把父类体当成本类体，
        // 成员全部落到类外；Test262 sm/class 家族抓获）。
        let mut pd = 0i32;
        loop {
            if self.at_eof() {
                break;
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" => {
                        pd += 1;
                        self.advance();
                        continue;
                    }
                    ")" | "]" => {
                        if pd == 0 {
                            break;
                        }
                        pd -= 1;
                        self.advance();
                        continue;
                    }
                    ";" if pd == 0 => break,
                    "{" if pd == 0 => break,
                    _ => {}
                }
            } else if pd == 0 && t.kind == TokKind::Ident && t.text == "class" {
                self.skip_class_expr();
                continue;
            } else if pd == 0 && t.kind == TokKind::Ident && t.text == "function" {
                // extends 里的函数表达式（`class extends function(){} {}`）
                self.advance();
                if self.is_punct("*") {
                    self.advance();
                }
                if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
                    self.advance();
                }
                if self.is_punct("(") {
                    self.skip_parens();
                }
                if self.is_punct("{") {
                    self.skip_braces();
                }
                continue;
            }
            self.advance();
        }
        head.push_str(&self.raw_between(hs, self.pos));
        if !self.is_punct("{") {
            self.pos = start;
            return None;
        }
        self.advance();
        let mut members = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            if let Some(m) = self.parse_class_member() {
                members.push(m);
            }
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        }
        let raw_span = self.span_u32(class_start);
        Some(ClassDecl { prefix, head_raw: head, kind_kw, name, members, raw_span })
    }

    fn parse_class_member(&mut self) -> Option<ClassMember> {
        let start = self.pos;
        // static 块
        if self.is_kw("static") && self.is_punct_at(1, "{") && !self.tk_at(1).newline_before {
            self.advance();
            let body = self.enter_async(false, |p| p.parse_block());
            return Some(ClassMember::StaticBlock { body });
        }
        if self.is_punct("{") {
            let body = self.enter_async(false, |p| p.parse_block());
            return Some(ClassMember::StaticBlock { body });
        }
        let mut had_deco = false;
        while self.is_punct("@") {
            had_deco = true;
            self.skip_decorator();
        }
        let mut method_async = false;
        loop {
            let t = self.tk().clone();
            // 修饰符循环：后随**任意键形态**（Ident/计算键 `[`/数字/字符串
            //——`static get [Symbol.species]() {}` 的 get 后随 `[`）
            if t.kind == TokKind::Ident
                && matches!(
                    t.text.as_str(),
                    "static" | "public" | "private" | "protected" | "readonly" | "abstract"
                        | "override" | "declare" | "async" | "get" | "set" | "const" | "accessor"
                )
                && (self.tk_at(1).kind == TokKind::Ident
                    || self.tk_at(1).kind == TokKind::Num
                    || self.tk_at(1).kind == TokKind::Str
                    || self.is_punct_at(1, "[")
                    // `static #x` —— 私有字段/方法名以 `#` 起（Test262
                    // class elements 族 SyntaxError 抓获）
                    || self.is_punct_at(1, "#")
                    // `static *gen()` —— 生成器方法（曾把 static 当独立
                    // 字段打印，`static; *method()` 语义改变）
                    || self.is_punct_at(1, "*"))
                && !self.tk_at(1).newline_before
            {
                if t.text == "async" {
                    method_async = true;
                }
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct("*") {
            self.advance();
        }
        let name_tok = self.tk().clone();
        let name = if name_tok.kind == TokKind::Ident {
            self.advance().text
        } else if name_tok.kind == TokKind::Punct && name_tok.text == "#" {
            self.advance();
            let mut n = String::from("#");
            if self.tk().kind == TokKind::Ident {
                n.push_str(&self.advance().text);
            }
            n
        } else if name_tok.kind == TokKind::Num || name_tok.kind == TokKind::Str {
            // 数字/字符串键（`0() {}` / `"s"() {}`——方法或字段）
            self.advance().text
        } else if name_tok.kind == TokKind::Punct && name_tok.text == "[" {
            // 计算键 [expr]：解析本体；名字进 sig_raw 原文（占位空串）
            self.advance();
            let _ = self.parse_assign_expr();
            if self.is_punct("]") {
                self.advance();
            } else {
                self.err_here("expected ]");
            }
            String::new()
        } else {
            let s = if had_deco { start } else { self.pos };
            self.skip_balanced_to_stmt_boundary();
            return Some(ClassMember::Raw(self.raw_between(s, self.pos)));
        };
        // TS 可选/明确赋值标记：`field?: T`、`field!: T`、`m?()`。
        if self.ts_syntax && (self.is_punct("?") || self.is_punct("!")) {
            self.advance();
        }
        // 泛型参数
        if self.is_punct("<") {
            self.skip_angle_balanced();
        }
        // 方法
        if self.is_punct("(") && !self.tk().newline_before {
            self.skip_parens();
            if self.is_punct(":") {
                self.skip_return_type();
            }
            // sig 捕获到 `{` 之前（body 是 arena 节点——含进去会双重打印；
            // 幂等差分抓获：constructor 曾打印体两遍）
            let sig_end = self.pos;
            let body = if self.is_punct("{") {
                Some(self.enter_async(method_async, |p| p.parse_block()))
            } else {
                self.consume_semi();
                None
            };
            if had_deco {
                // 装饰器成员整体原文保真：装饰器表达式（调用/成员链/
                // 括号）与成员体的边界比普通成员脆弱，且规则不触碰
                // 装饰器语义——整条 Raw 最安全。
                return Some(ClassMember::Raw(self.raw_between(start, self.pos)));
            }
            let sig_raw = self.raw_between(start, sig_end);
            return Some(ClassMember::Method { sig_raw, body });
        }
        // 字段：[: ty] [= init] ;
        if self.is_punct(":") && !self.tk().newline_before {
            self.skip_type_annotation();
        }
        let mut eq_pos = self.pos;
        let init = if self.is_punct("=") && !self.tk().newline_before {
            eq_pos = self.pos;
            self.advance();
            Some(self.enter_async(false, |p| p.parse_assign_expr()))
        } else {
            None
        };
        // sig 到 `=` 之前（init 是 arena 节点——打印不重复）。等号位置
        // 必须在解析 init **之前**记录：逆向搜索会命中 init 内部的
        // `x = () => { var t = … }` 的赋值号（Test262 class field 族抓到
        // sig 截成 `x = () => { var t`、打印时体重复）。
        let sig_end = if init.is_some() { eq_pos } else { self.pos };
        self.consume_semi();
        if had_deco {
            return Some(ClassMember::Raw(self.raw_between(start, self.pos)));
        }
        let sig_raw = self.raw_between(start, sig_end);
        Some(ClassMember::Field { sig_raw, name, init })
    }

    /// 在 [start, pos) 找最后一个顶层 `=` 的下标（字段 sig/init 分界）。
    /// 正向找**顶层**赋值号（解构模式 `{x = 1} = v` 的内部默认值在
    /// 括号深度内；逆向搜索会命中值表达式里的赋值）。
    fn eq_pos_forward(&self, start: usize) -> usize {
        let mut depth = 0i32;
        let mut i = start;
        while i < self.pos && i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => {
                        depth -= 1;
                        if depth < 0 {
                            break;
                        }
                    }
                    "=" if depth == 0 => return i,
                    _ => {}
                }
            }
            i += 1;
        }
        self.pos
    }

    fn skip_decorator(&mut self) {
        self.advance(); // @
        // `@(expr)` 括号装饰器：整体跳过
        if self.is_punct("(") {
            self.skip_parens();
            return;
        }
        // 装饰器成员表达式：IdentifierReference (`.` IdentifierName)*
        // 后接可选调用链。**不能**把裸 Ident 一直吞下去——`@dec m()`
        // 的 `m` 是下一个成员名（旧实现吞掉 m 和 ()，只留 `{}` 当
        // Raw，装饰器与方法一起丢失）。
        if self.tk().kind == TokKind::Ident {
            self.advance();
        }
        loop {
            if self.is_punct(".") {
                self.advance();
                if self.tk().kind == TokKind::Ident {
                    self.advance();
                } else if self.is_punct("#") {
                    self.advance();
                    if self.tk().kind == TokKind::Ident {
                        self.advance();
                    }
                } else {
                    break;
                }
            } else if self.is_punct("(") {
                self.skip_parens();
            } else {
                break;
            }
        }
    }

    fn skip_parens(&mut self) {
        if !self.is_punct("(") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("(") {
                depth += 1;
            } else if self.is_punct(")") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn skip_angle_balanced(&mut self) {
        if !self.is_punct("<") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("<") {
                depth += 1;
            } else if self.is_punct(">") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            } else if self.is_punct(">>") {
                depth -= 2;
                if depth <= 0 {
                    self.advance();
                    return;
                }
            } else if self.is_punct(">>>") {
                depth -= 3;
                if depth <= 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    // ---- 语句 ----

    fn parse_block(&mut self) -> TsId {
        if !self.is_punct("{") {
            self.err_here("expected {");
            return self.ast.block(vec![]);
        }
        self.advance();
        let mut stmts = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            let s = self.parse_stmt();
            // 多声明符 `let a, b;` 的 Group 在此展开为独立语句：Group 的
            // kind 是 Block，引擎语句表规则（dead_decl 等）只扫描组内
            // 兄弟，组外引用不可见 → 曾把声明当死代码删除。
            if matches!(self.ast.data(s), NodeData::Group) {
                stmts.extend(self.ast.children(s).iter().copied());
            } else {
                stmts.push(s);
            }
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        self.ast.block(stmts)
    }

    /// 消费分号（含 ASI：`}`/EOF/换行前）。
    fn consume_semi(&mut self) {
        if self.is_punct(";") {
            self.advance();
            return;
        }
        if self.is_punct("}") || self.at_eof() || self.tk().newline_before {
            return;
        }
        self.err_here("expected ;");
    }

    fn parse_stmt(&mut self) -> TsId {
        if self.depth >= MAX_NEST {
            self.too_deep = true;
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.depth += 1;
        let out = self.parse_stmt_inner();
        self.depth -= 1;
        out
    }

    fn parse_stmt_inner(&mut self) -> TsId {
        let t = self.tk().clone();
        match t.kind {
            TokKind::Punct => match t.text.as_str() {
                "{" => self.parse_block(),
                ";" => {
                    self.advance();
                    self.ast.empty()
                }
                "@" => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                _ => self.parse_expr_or_decl_stmt(),
            },
            TokKind::Ident => match t.text.as_str() {
                "var" | "const" => self.parse_var_decl_stmt(),
                "let" if self.let_is_decl() => self.parse_var_decl_stmt(),
                "using" if self.using_is_decl() => self.parse_var_decl_stmt(),
                "await"
                    if self.tk_at(1).kind == TokKind::Ident
                        && self.tk_at(1).text == "using"
                        && (self.tk_at(2).kind == TokKind::Ident
                            || self.is_punct_at(2, "{")
                            || self.is_punct_at(2, "[")) =>
                {
                    // `await using x = …;`（async 函数/模块顶层）：整条原文
                    // 保真（await using 的异步 dispose 时机 + 绑定模式不可
                    // 结构化；曾拆成 `await using; x = …`——Test262
                    // explicit-resource-management/async 家族抓获）
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "if" => self.parse_if(),
                "while" => self.parse_while(),
                "do" => self.parse_do_while(),
                "for" => self.parse_for(),
                "return" => self.parse_return(),
                "break" | "continue" => self.parse_break_continue(),
                "throw" => self.parse_throw(),
                "try" => self.parse_try(),
                "switch" => self.parse_switch(),
                "function" | "async" if self.async_function_ahead() => {
                    match self.try_parse_function_stmt() {
                        Some(s) => s,
                        None => self.parse_expr_or_decl_stmt(),
                    }
                }
                "class" | "abstract" if self.tk_at(1).kind == TokKind::Ident => {
                    let s = self.pos;
                    self.skip_class_expr_stmt();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "debugger" => {
                    self.advance();
                    self.consume_semi();
                    self.ast.raw("debugger;")
                }
                "with" => {
                    // with 的**语句体**必须一起原文捕获：体是 with 作用域
                    // 的一部分（`with (o) var x = 1, _ = f;` 的初始化器读到
                    // with 对象属性）。曾只捕到 `with (o)` —— 体变成后续
                    // 语句，闭包读到外层值（Test262 with/scope-var-close 抓获）。
                    let s = self.pos;
                    self.advance();
                    if self.is_punct("(") {
                        self.skip_parens();
                    }
                    if self.is_punct("{") {
                        self.skip_braces();
                        self.consume_semi();
                    } else {
                        self.skip_balanced_to_stmt_boundary();
                        // `else` 分支（同一行）
                        if self.tk().kind == TokKind::Ident && self.tk().text == "else" {
                            self.advance();
                            if self.is_punct("{") {
                                self.skip_braces();
                            } else {
                                self.skip_balanced_to_stmt_boundary();
                            }
                        }
                    }
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "enum" => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "interface" | "namespace" | "module"
                    if self.tk_at(1).kind == TokKind::Ident
                        || self.tk_at(1).kind == TokKind::Str =>
                {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "type" if self.type_decl_shape() => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                _ => self.parse_expr_or_decl_stmt(),
            },
            _ => self.parse_expr_or_decl_stmt(),
        }
    }

    /// `let` 是声明还是标识符。
    fn let_is_decl(&self) -> bool {
        let n = self.tk_at(1);
        if n.newline_before {
            return false;
        }
        matches!(n.kind, TokKind::Ident)
            || (n.kind == TokKind::Punct && matches!(n.text.as_str(), "[" | "{"))
    }

    fn using_is_decl(&self) -> bool {
        let n = self.tk_at(1);
        !n.newline_before && n.kind == TokKind::Ident
    }

    fn async_function_ahead(&self) -> bool {
        if self.is_kw("function") {
            return true;
        }
        self.is_kw("async") && self.tk_at(1).text == "function" && !self.tk_at(1).newline_before
    }

    /// 类型扫描的行首边界：上一 token 已能结束一个类型，且当前
    /// 行首 token 是声明/语句关键字（或 `@`）→ 类型到此结束。
    /// 处理无分号的 ambient 声明：`export const utils: { … }` 后直接
    /// `export const …`；也防止 `as string[]` 吞掉下一条语句。
    fn ends_type_at_new_stmt(&self) -> bool {
        let t = self.tk();
        if !t.newline_before {
            return false;
        }
        let prev = if self.pos == 0 {
            None
        } else {
            Some(self.toks[self.pos - 1].text.as_str())
        };
        let prev_ends = match prev {
            None => false,
            Some(p) => !matches!(
                p,
                "typeof" | "keyof" | "readonly" | "infer" | "extends" | "is" | "new"
                    | "abstract" | "asserts" | "as" | "satisfies" | "in" | "out" | "|" | "&"
                    | "?" | ":" | "=>" | "," | "<" | "(" | "[" | "."
            ),
        };
        if !prev_ends {
            return false;
        }
        if t.kind == TokKind::Eof || (t.kind == TokKind::Punct && t.text == "@") {
            return true;
        }
        if t.kind != TokKind::Ident {
            return false;
        }
        if STMT_KW.contains(&t.text.as_str()) {
            return true;
        }
        // 类成员边界（无分号的字段/方法）：`set: T` 下一行 `_type: K`。
        // 关键看名字后的形态——`: ? ! ( < = ;` 说明是新成员签名。
        if matches!(
            t.text.as_str(),
            "static" | "get" | "set" | "async" | "readonly" | "declare" | "override"
                | "abstract" | "public" | "private" | "protected" | "accessor"
        ) {
            return true;
        }
        let n = self.tk_at(1);
        n.kind == TokKind::Punct
            && matches!(n.text.as_str(), ":" | "?" | "!" | "(" | "<" | "=" | ";")
    }

    /// 跳过 TS 类型注解，停在深度 0 的 `=`/`;`/`,`/`)`/`]`/`}` 之前。
    /// 类型可含对象/元组/索引访问/函数/条件类型：`{…}`、`[…]`、`<…>`、
    /// `=>` 都是类型语法的一部分，不能再以 `{`/`[` 为提前终止符
    ///（`let value: T[K] | undefined` 曾被截成 `let value: T; [K] …`）。
    fn skip_type_annotation(&mut self) {
        let (mut paren, mut brace, mut brack, mut angle) = (0i32, 0i32, 0i32, 0i32);
        loop {
            if self.at_eof() {
                return;
            }
            if paren == 0 && brace == 0 && brack == 0 && angle == 0
                && self.ends_type_at_new_stmt()
            {
                return;
            }
            let t = self.tk().clone();
            if paren == 0 && brace == 0 && brack == 0 && angle == 0 && t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "=" | ";" | "," | ")" | "]" | "}" => return,
                    _ => {}
                }
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => paren += 1,
                    ")" => paren -= 1,
                    "[" => brack += 1,
                    "]" => brack -= 1,
                    "{" => brace += 1,
                    "}" => brace -= 1,
                    "<" => angle += 1,
                    ">" => angle -= 1,
                    ">>" => angle -= 2,
                    ">>>" => angle -= 3,
                    _ => {}
                }
            }
            self.advance();
        }
    }

    /// `type Name[<TypeParams>] =` 形态判定（类型别名）。
    /// 泛型参数必须允许：`type M<T> = T extends …`（zod 的 mapped/
    /// conditional types 曾因只认 `type X =` 被当表达式语句拆散）。
    fn type_decl_shape(&self) -> bool {
        if !self.is_kw("type") || self.tk_at(1).kind != TokKind::Ident {
            return false;
        }
        let mut i = 2;
        if self.is_punct_at(2, "<") {
            // match_angle 接收绝对 token 下标；其余 *_at 是相对偏移。
            let Some(close) = self.match_angle(self.pos + 2) else {
                return false;
            };
            i = close - self.pos + 1;
        }
        self.is_punct_at(i, "=")
    }

    /// 当前节点从 `start_tok` 到最近消费 token 的源码字节区间。
    fn span_u32(&self, start_tok: usize) -> Option<(u32, u32)> {
        if !self.capture_source {
            return None;
        }
        let s = self.toks.get(start_tok)?.start;
        let e = if self.pos > start_tok {
            self.toks.get(self.pos - 1)?.end
        } else {
            s
        };
        Some((s, e))
    }

    fn set_span_here(&mut self, id: TsId, start_tok: usize) {
        if let Some((s, e)) = self.span_u32(start_tok) {
            self.ast.set_source_span(id, s as usize, e as usize);
        }
    }

    /// 嵌套函数声明语句 → FuncStmt（Custom——引擎不触碰，体进 arena）。
    fn try_parse_function_stmt(&mut self) -> Option<TsId> {
        let start = self.pos;
        let mut is_async = false;
        if self.is_kw("async") && self.tk_at(1).text == "function" {
            is_async = true;
            self.advance();
        }
        if !self.is_kw("function") {
            self.pos = start;
            return None;
        }
        let fn_start = if is_async { start } else { self.pos };
        self.advance();
        let mut is_generator = false;
        if self.is_punct("*") {
            is_generator = true;
            self.advance();
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        let mut generics = String::new();
        if self.is_punct("<") {
            let gs = self.pos;
            self.skip_angle_balanced();
            generics = self.raw_between(gs, self.pos).trim().to_string();
        }
        if !self.is_punct("(") {
            self.pos = start;
            return None;
        }
        let ps = self.pos;
        self.skip_parens();
        let params_full = self.raw_between(ps, self.pos);
        let params_inner = params_full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&params_full)
            .to_string();
        let mut ret_raw = String::new();
        if self.is_punct(":") {
            let rs = self.pos;
            self.skip_return_type();
            ret_raw = self.raw_between(rs, self.pos);
        }
        let body = if self.is_punct("{") {
            self.enter_async(is_async, |p| p.parse_block())
        } else {
            self.pos = start;
            return None;
        };
        let mut sig = String::new();
        if is_async {
            sig.push_str("async ");
        }
        sig.push_str("function");
        if is_generator {
            sig.push('*');
        }
        if !name.is_empty() {
            sig.push(' ');
            sig.push_str(&name);
        }
        if !generics.is_empty() {
            if name.is_empty() {
                sig.push(' ');
            }
            sig.push_str(&generics);
        }
        sig.push_str(&params_full);
        sig.push_str(&ret_raw);
        let id = self.ast.func_stmt(&sig, &name, &params_inner, body);
        self.set_span_here(id, fn_start);
        Some(id)
    }

    /// 嵌套类声明语句：扫 `class … { … }` 平衡。
    /// 类声明自带终止（不需要分号）——压缩代码 `}class B{}let x` 里
    /// 强制 consume_semi 会报 `expected ;`（diff.min.js 抓获）。
    fn skip_class_expr_stmt(&mut self) {
        self.skip_class_expr();
        if self.is_punct(";") {
            self.advance();
        }
    }

    fn parse_var_decl_stmt(&mut self) -> TsId {
        let kw = self.advance().text;
        let kind = match kw.as_str() {
            "var" => VarKind::Var,
            "let" => VarKind::Let,
            "using" => VarKind::Using,
            _ => VarKind::Const,
        };
        let kw_for_print = kw.clone();
        let mut stmts: Vec<TsId> = Vec::new();
        loop {
            if self.is_punct("{") || self.is_punct("[") {
                // 解构声明
                let ps = self.pos;
                if self.is_punct("{") {
                    self.skip_braces();
                } else {
                    self.skip_brackets();
                }
                let mut pattern = self.raw_between(ps, self.pos);
                if self.is_punct(":") && !self.tk().newline_before {
                    let ts = self.pos;
                    self.skip_type_annotation();
                    pattern.push_str(&self.raw_between(ts, self.pos));
                }
                if self.is_punct("=") && !self.tk().newline_before {
                    self.advance();
                    let value = self.parse_assign_expr();
                    stmts.push(self.ast.destruct_stmt(&kw_for_print, &pattern, value));
                } else {
                    self.err_here("destructuring declaration without initializer");
                    stmts.push(self.ast.raw(&format!("{kw_for_print} {pattern}")));
                }
            } else if self.tk().kind == TokKind::Ident {
                let name = self.advance().text;
                let mut ty_raw: Option<String> = None;
                // TS 明确赋值断言 `let x!: T;`
                let mut ty_prefix = String::new();
                if self.ts_syntax && self.is_punct("!") && !self.tk().newline_before {
                    self.advance();
                    ty_prefix.push('!');
                }
                if self.is_punct(":") && !self.tk().newline_before {
                    let ts = self.pos;
                    self.skip_type_annotation();
                    ty_prefix.push_str(&self.raw_between(ts, self.pos));
                }
                if !ty_prefix.is_empty() {
                    ty_raw = Some(ty_prefix);
                }
                // `=` 不受限产生式：`var x \n = 1` 按续行解析（JS 的
                // ASI 只在下一 token 无法续行时插入——`=` 只能续行）
                let init = if self.is_punct("=") {
                    self.advance();
                    Some(self.parse_assign_expr())
                } else {
                    None
                };
                stmts.push(self.ast.var_decl(&name, kind, ty_raw.as_deref(), init));
            } else {
                self.err_here("expected declarator");
                break;
            }
            if self.is_punct(",") {
                self.advance();
                continue;
            }
            break;
        }
        self.consume_semi();
        if stmts.len() == 1 {
            stmts[0]
        } else {
            self.ast.group(stmts)
        }
    }

    fn skip_braces(&mut self) {
        if !self.is_punct("{") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("{") {
                depth += 1;
            } else if self.is_punct("}") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn skip_brackets(&mut self) {
        if !self.is_punct("[") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("[") {
                depth += 1;
            } else if self.is_punct("]") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn parse_if(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after if");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let cond = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        let then = self.parse_stmt_or_group();
        let els = if self.is_kw("else") {
            self.advance();
            Some(self.parse_stmt_or_group())
        } else {
            None
        };
        self.ast.if_(cond, then, els)
    }

    fn parse_stmt_or_group(&mut self) -> TsId {
        if self.is_punct("{") {
            self.parse_block()
        } else {
            self.parse_stmt()
        }
    }

    fn parse_while(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after while");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let cond = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        let body = self.parse_stmt_or_group();
        self.ast.while_(cond, body)
    }

    fn parse_do_while(&mut self) -> TsId {
        self.advance();
        let body = self.parse_stmt_or_group();
        if self.is_kw("while") {
            self.advance();
            if self.is_punct("(") {
                self.advance();
                let cond = self.parse_expr();
                if self.is_punct(")") {
                    self.advance();
                } else {
                    self.err_here("expected )");
                }
                self.consume_semi();
                return self.ast.do_while(body, cond);
            }
        }
        self.err_here("malformed do-while");
        self.ast.empty()
    }

    fn parse_for(&mut self) -> TsId {
        let for_pos = self.pos; // `for` 关键字位（destruct 回退捕获用）
        self.advance();
        let is_await = if self.is_kw("await") {
            self.advance();
            true
        } else {
            false
        };
        if !self.is_punct("(") {
            self.err_here("expected ( after for");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let head_start = self.pos;
        // 头部分派：of / in / 经典
        let mut depth = 0i32;
        let mut of_at = None;
        let mut in_at = None;
        // 三元嵌套深度：`for (true ? '' in f() : g(); ; )` 的 `in`
        // 是二元运算符，不是 for-in 分隔符（Test262 conditional
        // in-branch 抓获）。
        let mut qdepth = 0i32;
        // 声明头之后的第一个标识符是**绑定名**（`for (await using of of [])`
        // 的绑定名就是 of）——扫描分隔关键字时跳过它，否则第一个 of
        // 被当分隔符、iterable 从第二个 of 开始错位。
        let mut search_from = self.pos;
        if self.toks[search_from].kind == TokKind::Ident
            && self.toks[search_from].text == "await"
            && self.toks.get(search_from + 1).map(|t| t.text.as_str()) == Some("using")
        {
            search_from += 2;
        } else if self.toks[search_from].kind == TokKind::Ident
            && matches!(
                self.toks[search_from].text.as_str(),
                "var" | "let" | "const" | "using"
            )
        {
            search_from += 1;
        }
        let mut skip_binding_of =
            self.toks[search_from].kind == TokKind::Ident && self.toks[search_from].text == "of";
        let mut i = search_from;
        while i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Eof {
                break;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" if depth == 0 => break,
                    ")" | "]" | "}" => depth -= 1,
                    ";" if depth == 0 && qdepth == 0 => break,
                    "?" if depth == 0 => qdepth += 1,
                    ":" if depth == 0 && qdepth > 0 => qdepth -= 1,
                    _ => {}
                }
            } else if t.kind == TokKind::Ident && depth == 0 && qdepth == 0 {
                if t.text == "of" && of_at.is_none() && in_at.is_none() {
                    // `async of => {}`（参数名为 of 的箭头）不是分隔符
                    if self.toks.get(i + 1).is_some_and(|n| n.text == "=>") {
                        i += 1;
                        continue;
                    }
                    if i == search_from && skip_binding_of {
                        skip_binding_of = false;
                        i += 1;
                        continue;
                    }
                    of_at = Some(i);
                    break;
                }
                if t.text == "in" && of_at.is_none() && in_at.is_none() {
                    in_at = Some(i);
                    break;
                }
            }
            i += 1;
        }
        if let Some(kw_at) = of_at.or(in_at) {
            let is_of = of_at.is_some();
            let bind_raw = self.raw_between(head_start, kw_at);
            // `for (let in obj)` 的 let 是**绑定名**（sloppy 允许 let
            // 作标识符）——紧邻分隔符时不是声明（Test262 head-lhs-let
            // 抓获：曾打印成 `for (let in in obj)`）。
            let let_is_binding = self.toks[head_start].text == "let"
                && self
                    .toks
                    .get(head_start + 1)
                    .is_some_and(|t| matches!(t.text.as_str(), "in" | "of"));
            let mut decl = if self.toks[head_start].kind == TokKind::Ident
                && !let_is_binding
                && matches!(self.toks[head_start].text.as_str(), "var" | "let" | "const" | "using")
            {
                Some(match self.toks[head_start].text.as_str() {
                    "var" => VarKind::Var,
                    "let" => VarKind::Let,
                    "using" => VarKind::Using,
                    _ => VarKind::Const,
                })
            } else {
                None
            };
            let (name, ty_raw) = if decl.is_some() {
                let n = &self.toks[head_start + 1];
                // Annex B：sloppy for-in 允许声明初始化器
                //（`for (var a = ++effects in {})`）——初始化器只求值一次，
                // 必须随绑定原文保留（曾被丢弃，副作用消失；
                // Test262 nonstrict-initializer 抓获）。
                if !is_of && n.kind == TokKind::Ident && head_start + 2 < kw_at
                    && self.toks[head_start + 2].text == "="
                {
                    let bind = self.raw_between(head_start + 1, kw_at);
                    (bind.trim().to_string(), None)
                } else if n.kind == TokKind::Ident {
                    let nm = n.text.clone();
                    let ty = if head_start + 2 < kw_at && self.toks[head_start + 2].text == ":" {
                        Some(self.raw_between(head_start + 2, kw_at))
                    } else {
                        None
                    };
                    (nm, ty)
                } else {
                    // 解构 for-of/for-in：绑定原文整体进 name（含
                    // `let/const/var` 与模式），decl=None 让打印器原样
                    // 输出 head。**不**整条原文捕获 body——单语句体里的
                    // if/else 链会被语句边界扫描截断，`else` 残句化
                    //（真实库 oxfmt 编译差分抓获：for (... ) if (...) …;
                    // else if ... 的 else 被拆成独立语句）。
                    decl = None;
                    (bind_raw.trim().to_string(), None)
                }
            } else {
                (bind_raw.trim().to_string(), None)
            };
            self.pos = kw_at + 1;
            let iterable = self.parse_expr();
            if self.is_punct(")") {
                self.advance();
            } else {
                self.err_here("expected ) in for head");
            }
            let body = self.parse_stmt_or_group();
            return self.ast.for_each(
                &name,
                is_of,
                decl,
                ty_raw.as_deref(),
                iterable,
                body,
                is_await,
            );
        }
        // 经典 for 头以 `await using` 声明开头（资源在每轮末异步释放）：
        // 整条 for 原文保真（表达式级改写会拆散 head 的 `;`——
        // Test262 await-using forstatement 家族 SyntaxError 抓获）
        if self.is_kw("await") && self.tk_at(1).text == "using" {
            self.pos = for_pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(for_pos, self.pos));
        }
        // 经典 for：init（声明已含 `;` 消费）
        let mut inits: Vec<TsId> = Vec::new();
        if !self.is_punct(";") {
            if self.is_kw("var")
                || self.is_kw("const")
                || self.is_kw("using")
                || (self.is_kw("let") && self.let_is_decl())
            {
                let d = self.parse_var_decl_stmt();
                if self.ast.kind(d) == cure_engine::NodeKind::Block {
                    inits = self.ast.children(d).to_vec();
                } else {
                    inits = vec![d];
                }
            } else {
                let e = self.parse_expr();
                inits.push(self.ast.expr_stmt(e));
                if self.is_punct(";") {
                    self.advance();
                } else {
                    self.err_here("expected ; in for head");
                }
            }
        } else {
            self.advance();
        }
        let cond = if self.is_punct(";") {
            None
        } else {
            Some(self.parse_expr())
        };
        if self.is_punct(";") {
            self.advance();
        } else {
            self.err_here("expected ; in for head");
        }
        let mut steps: Vec<TsId> = Vec::new();
        while !self.is_punct(")") && !self.at_eof() {
            let e = self.parse_expr();
            steps.push(self.ast.expr_stmt(e));
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected ) in for head");
        }
        let body = self.parse_stmt_or_group();
        self.ast.for_(inits, cond, steps, body)
    }

    fn parse_return(&mut self) -> TsId {
        self.advance();
        if self.tk().newline_before || self.is_punct(";") || self.is_punct("}") || self.at_eof() {
            self.consume_semi();
            return self.ast.ret(None);
        }
        let e = self.parse_expr();
        self.consume_semi();
        self.ast.ret(Some(e))
    }

    fn parse_break_continue(&mut self) -> TsId {
        let kw = self.advance().text;
        let label = if self.tk().kind == TokKind::Ident && !self.tk().newline_before {
            Some(self.advance().text)
        } else {
            None
        };
        self.consume_semi();
        if kw == "break" {
            self.ast.break_(label.as_deref())
        } else {
            self.ast.continue_(label.as_deref())
        }
    }

    fn parse_throw(&mut self) -> TsId {
        self.advance();
        if self.tk().newline_before {
            self.err_here("throw followed by newline (ASI)");
            self.consume_semi();
            return self.ast.raw("throw;");
        }
        let e = self.parse_expr();
        self.consume_semi();
        self.ast.throw(e)
    }

    fn parse_try(&mut self) -> TsId {
        self.advance();
        let block = if self.is_punct("{") {
            self.parse_block()
        } else {
            self.err_here("expected { after try");
            return self.ast.raw("try");
        };
        let mut catches = Vec::new();
        while self.is_kw("catch") {
            self.advance();
            let param_raw = if self.is_punct("(") {
                let ps = self.pos;
                self.skip_parens();
                let full = self.raw_between(ps, self.pos);
                Some(
                    full.strip_prefix('(')
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(&full)
                        .to_string(),
                )
            } else {
                None
            };
            let cbody = if self.is_punct("{") {
                self.parse_block()
            } else {
                self.err_here("expected { after catch");
                self.ast.block(vec![])
            };
            catches.push(self.ast.catch_(param_raw.as_deref(), cbody));
        }
        let finally = if self.is_kw("finally") {
            self.advance();
            if self.is_punct("{") {
                Some(self.parse_block())
            } else {
                self.err_here("expected { after finally");
                None
            }
        } else {
            None
        };
        if catches.is_empty() && finally.is_none() {
            self.err_here("try without catch/finally");
        }
        self.ast.try_(block, catches, finally)
    }

    fn parse_switch(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after switch");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let subject = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        if !self.is_punct("{") {
            self.err_here("expected { after switch");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let mut cases = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            if let Some(c) = self.parse_case() {
                cases.push(c);
            }
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        self.ast.switch_(subject, cases)
    }

    fn parse_case(&mut self) -> Option<TsId> {
        let case_start = self.pos;
        if self.is_kw("case") {
            self.advance();
            let label = self.parse_expr();
            if self.is_punct("=>") {
                // TS switch 表达式箭头 case——整条原文
                self.skip_balanced_to_stmt_boundary();
                return Some(self.ast.raw(&self.raw_between(case_start, self.pos)));
            }
            if self.is_punct(":") {
                self.advance();
            } else {
                self.err_here("expected : after case");
            }
            let stmts = self.parse_case_stmts();
            Some(self.ast.case_(vec![label], false, false, stmts))
        } else if self.is_kw("default") {
            self.advance();
            if self.is_punct("=>") {
                self.skip_balanced_to_stmt_boundary();
                return Some(self.ast.raw(&self.raw_between(case_start, self.pos)));
            }
            if self.is_punct(":") {
                self.advance();
            } else {
                self.err_here("expected : after default");
            }
            let stmts = self.parse_case_stmts();
            Some(self.ast.case_(vec![], true, false, stmts))
        } else {
            self.err_here("expected case/default in switch");
            None
        }
    }

    fn parse_case_stmts(&mut self) -> Vec<TsId> {
        let mut stmts = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            if self.is_kw("case") || self.is_kw("default") {
                break;
            }
            let before = self.pos;
            let s = self.parse_stmt();
            stmts.push(s);
            if self.pos == before {
                self.advance();
            }
        }
        stmts
    }

    fn parse_expr_or_decl_stmt(&mut self) -> TsId {
        let t = self.tk().clone();
        // 标签语句：语句位的 `ident :` 恒为标签（对象字面量不能起始
        // 语句；三元不可能在 ident 后立刻出现冒号——语句位无前驱
        // 操作数。test262：yield: 1 / await: 2 等 sloppy 标签）
        if t.kind == TokKind::Ident && self.is_punct_at(1, ":") && !self.tk_at(1).newline_before {
            let name = self.advance().text;
            self.advance(); // :
            let stmt = self.parse_stmt();
            return self.ast.label(&name, stmt);
        }
        let start = self.pos;
        let e = self.parse_expr();
        // 解构赋值语句：Assign(target 是 ObjLit/ArrayLit)
        if let Some(stmt) = self.wrap_destruct_assign(e, start) {
            // 语句自身的 `;` 必须在此消费——否则下一轮解析把它当
            // Empty 语句，且每轮再解析都多出一个 `;`（无界增长）
            self.consume_semi();
            return stmt;
        }
        // 裸字符串表达式语句 = 指令序言（"use strict" 等）：**原文**保留。
        // 反转义后重新编码会把 `'use str\u0069ct'` 变成 `'use strict'`，
        // 从而把非指令变成 use strict 指令（Test262 directive-prologue
        // 家族抓获：转义/续行形态必须保持非严格）。
        if matches!(self.ast.data(e), NodeData::Literal(JsLit::Str(_))) {
            let raw = self.raw_between(start, self.pos);
            self.consume_semi();
            return self.ast.raw(&raw);
        }
        self.consume_semi();
        self.ast.expr_stmt(e)
    }

    fn wrap_destruct_assign(&mut self, e: TsId, start: usize) -> Option<TsId> {
        if self.ast.kind(e) != cure_engine::NodeKind::Assign {
            return None;
        }
        let ch = self.ast.children(e).to_vec();
        if ch.len() != 2 {
            return None;
        }
        let (target, value) = (ch[0], ch[1]);
        let pattern = match self.ast.data(target) {
            NodeData::ObjLit { .. } | NodeData::ArrayLit => {
                let eq = self.eq_pos_forward(start);
                self.raw_between(start, eq)
            }
            _ => return None,
        };
        Some(self.ast.destruct_stmt("", &pattern, value))
    }

    // ---- 表达式 ----

    fn parse_expr(&mut self) -> TsId {
        let first = self.parse_assign_expr();
        if !self.is_punct(",") {
            return first;
        }
        let mut elems = vec![first];
        while self.is_punct(",") {
            self.advance();
            elems.push(self.parse_assign_expr());
        }
        self.ast.seq(elems)
    }

    fn parse_assign_expr(&mut self) -> TsId {
        if self.depth >= MAX_NEST {
            self.too_deep = true;
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.depth += 1;
        let out = self.parse_assign_expr_inner();
        self.depth -= 1;
        out
    }

    fn parse_assign_expr_inner(&mut self) -> TsId {
        if let Some(a) = self.try_parse_arrow() {
            return a;
        }
        let left = if self.is_kw("yield") {
            // sloppy 位的 yield 标识符用法：yield = x / yield.x /
            // yield[x] / yield(x) / yield, / yield)——按 VarRef（
            // 生成器语境外 yield 是普通名；test262 non-strict 家族）。
            // 标识符形态必须继续走赋值后缀（`yield = 1` 曾是
            // `yield =; 1;` 残句——Test262 assignmenttargettype 抓获）。
            // 前瞻 token 形态（`yield` 自身当然不是这些标点——原实现
            // 漏了 _at(1)，`yield = 1` 一直走 yield 表达式路径产出残句）
            // 注意：**不能**把 `[`/`(` 当作标识符判据——生成器里
            // `yield [i, ...]` / `yield (x)` 是 yield 表达式（曾把
            // `yield [i,a[i]]` 拆成 `yield; [ ... ]`，Test262
            // generators/intl402 zip 抓获）。非生成器里两种解析的
            // 原文相同（Raw 保真）。
            if self.is_punct_at(1, "=")
                || self.is_punct_at(1, ".")
                || self.is_punct_at(1, ",")
                || self.is_punct_at(1, ")")
                || self.is_punct_at(1, ";")
                || self.tk_at(1).kind == TokKind::Eof
            {
                let name = self.advance().text;
                self.ast.var(&name)
            } else {
                // yield 表达式：原文捕获（生成器恢复 v1 不简化）
                let s = self.pos;
                self.advance();
                let mut starred = false;
                if self.is_punct("*") {
                    self.advance();
                    starred = true;
                }
                // `yield *` 的换行合法（只有 yield 与 * 之间禁换行）——
                // starred 时强制解析操作数（`yield *\n foo` 曾被截成
                // `yield *;`，Test262 staging/sm/generators 抓获）。
                // `yield` 后同行的**新模板**是操作数（`yield \`…\``）；
                // 模板续接段（`${ yield }3…` 里的 `}3…`，cont=true）
                // 属于外层模板，不是 yield 操作数。
                let template_cont = matches!(
                    self.tk().kind,
                    TokKind::TemplateQuasi { cont: true, .. }
                );
                if (starred || (!self.tk().newline_before && !template_cont))
                    && !self.is_punct(";")
                    && !self.is_punct(")")
                    && !self.is_punct("]")
                    && !self.is_punct("}")
                    && !self.is_punct(",")
                    && !self.is_punct(":")
                    && !self.is_punct("?")
                {
                    let _ = self.parse_assign_expr();
                }
                let y = self.ast.raw(&self.raw_between(s, self.pos));
                // `yield ? a : b`：YieldExpression 可作三元条件
                if self.is_punct("?") {
                    self.advance();
                    let a = self.parse_assign_expr();
                    if self.is_punct(":") {
                        self.advance();
                    } else {
                        self.err_here("expected : in conditional");
                    }
                    let b = self.parse_assign_expr();
                    return self.ast.ternary(y, a, b);
                }
                return y;
            }
        } else {
            self.parse_conditional()
        };
        if self.tk().kind == TokKind::Punct {
            if let Some(op) = assign_op_of(&self.tk().text) {
                self.advance();
                let right = self.parse_assign_expr();
                return self.ast.assign_op(op, left, right);
            }
            if self.tk().text == "=" {
                self.advance();
                let right = self.parse_assign_expr();
                return self.ast.assign(left, right);
            }
        }
        left
    }

    /// 从返回类型起点找到箭头函数的 `=>`（绝对下标）。对象/映射/
    /// 函数/条件类型里的 `{…}`、`<…>` 都按类型处理；深度 0 的
    /// `;`/`,`/`)`/`]`/`}` 说明不是箭头返回类型。
    fn find_arrow_after_type(&self, start: usize) -> Option<usize> {
        let (mut paren, mut brack, mut brace, mut angle) = (0i32, 0i32, 0i32, 0i32);
        let mut i = start;
        while i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Eof {
                return None;
            }
            if paren == 0 && brack == 0 && brace == 0 && angle == 0
                && t.kind == TokKind::Punct
            {
                match t.text.as_str() {
                    "=>" => return Some(i),
                    // 类型语法在深度 0 不含赋值号；出现 `=` 说明这是
                    // 表达式语境（`(v=1) : cond ? … => …` 曾被误判为
                    // 带返回类型的箭头，把后续三元吞掉）。
                    "=" => return None,
                    ";" | "," | ")" | "]" | "}" => return None,
                    "{" => {
                        brace += 1;
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => paren += 1,
                    ")" => paren -= 1,
                    "[" => brack += 1,
                    "]" => brack -= 1,
                    "{" => brace += 1,
                    "}" => brace -= 1,
                    "<" => angle += 1,
                    ">" => angle -= 1,
                    ">>" => angle -= 2,
                    ">>>" => angle -= 3,
                    _ => {}
                }
            }
            i += 1;
        }
        None
    }

    fn try_parse_arrow(&mut self) -> Option<TsId> {
        let src_start = self.pos;
        let mut p = self.pos;
        let mut is_async = false;
        if self.toks[p].kind == TokKind::Ident && self.toks[p].text == "async" {
            let n = self.toks.get(p + 1)?;
            if (n.kind == TokKind::Ident
                || (n.kind == TokKind::Punct && matches!(n.text.as_str(), "(" | "<")))
                && !n.newline_before
            {
                is_async = true;
                p += 1;
            }
        }
        // TS 泛型箭头：<T,>(x: T) => … / <T extends U>(x) => …
        // 类型参数必须后随括号参数表；`<` 在表达式起始不是合法 JS。
        let mut generics = String::new();
        if self.toks[p].kind == TokKind::Punct && self.toks[p].text == "<" {
            let close = self.match_angle(p)?;
            let params_at = close + 1;
            let ok = self
                .toks
                .get(params_at)
                .is_some_and(|t| t.kind == TokKind::Punct && t.text == "(");
            if !ok {
                return None;
            }
            generics = self.raw_between(p, params_at).trim().to_string();
            p = params_at;
        }
        // ident =>
        if self.toks[p].kind == TokKind::Ident && generics.is_empty() {
            let n = self.toks.get(p + 1)?;
            if n.kind == TokKind::Punct && n.text == "=>" {
                let param = self.toks[p].text.clone();
                self.pos = p + 2;
                let body = self.enter_async(is_async, |p| p.parse_arrow_body());
                let id = self.ast.lambda(&param, body, true, is_async, false);
                self.set_span_here(id, src_start);
                return Some(id);
            }
            return None;
        }
        // ( … ) [: ret] =>
        if self.toks[p].kind == TokKind::Ident {
            return None;
        }
        if !(self.toks[p].kind == TokKind::Punct && self.toks[p].text == "(") {
            return None;
        }
        let close = self.match_paren(p)?;
        let mut q = close + 1;
        let mut ret_raw = String::new();
        if q < self.toks.len()
            && self.toks[q].kind == TokKind::Punct
            && self.toks[q].text == ":"
            && self.params_plausible(p, close)
        {
            // 返回类型前瞻。对象/映射/函数/条件类型里的 `{…}` 属于
            // 类型（`(x): { a: T } => …` 曾因裸 `{` 被拒）。
            let arrow_at = self.find_arrow_after_type(q + 1)?;
            ret_raw = self.raw_between(q, arrow_at);
            q = arrow_at;
        }
        let arrow_tok = self.toks.get(q)?;
        if !(arrow_tok.kind == TokKind::Punct && arrow_tok.text == "=>") {
            return None;
        }
        let params_full = self.raw_between(p, close + 1);
        let params_inner = params_full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&params_full)
            .to_string();
        self.pos = q + 1;
        let body = self.enter_async(is_async, |p| p.parse_arrow_body());
        let id = if generics.is_empty() && ret_raw.is_empty() {
            self.ast.lambda(&params_inner, body, true, is_async, false)
        } else {
            self.ast.lambda_named_generic_ret(
                &generics,
                &ret_raw,
                &params_inner,
                body,
                true,
                is_async,
                false,
                "",
                false,
            )
        };
        self.set_span_here(id, src_start);
        Some(id)
    }

    /// 括号组是否可能是**参数列表**（TS 返回类型注解前瞻的守卫）：
    /// `(v=()=>e.value,$=hr(e))` 这类逗号表达式/后续三元里的括号
    /// 不能被误读成 `(…): T =>` 的箭头参数（真实库 minified 代码
    /// 的 `c ? (a=()=>x,b=f()) : …` 曾被吞成一层箭头）。
    fn params_plausible(&self, open: usize, close: usize) -> bool {
        #[derive(Clone, Copy, PartialEq)]
        enum St {
            SegStart,
            InIdent,
            InTail,
        }
        let mut st = St::SegStart;
        let mut depth = 0i32;
        let mut i = open + 1;
        while i < close {
            let t = &self.toks[i];
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" | "{" | "<" => {
                        depth += 1;
                        i += 1;
                        continue;
                    }
                    ")" | "]" | "}" | ">" => {
                        if depth > 0 {
                            depth -= 1;
                        }
                        // 类型语境（InTail）里闭合元组/泛型后仍是类型：
                        // `RadixTree<T> | undefined`、`[A, B]` 之后的
                        // `|`/`&` 不能触发“非参数列表”拒绝。
                        if depth == 0 && st != St::InTail {
                            st = St::InIdent;
                        }
                        i += 1;
                        continue;
                    }
                    ">>" => {
                        depth -= 2;
                        if depth < 0 {
                            depth = 0;
                        }
                        i += 1;
                        continue;
                    }
                    ">>>" => {
                        depth -= 3;
                        if depth < 0 {
                            depth = 0;
                        }
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
            }
            if depth == 0 {
                let ident_like = matches!(
                    t.kind,
                    TokKind::Ident | TokKind::Num | TokKind::Str | TokKind::Regex
                );
                if t.kind == TokKind::Punct {
                    match t.text.as_str() {
                        "," => st = St::SegStart,
                        // 剩余参数 `...args`：省略号后仍需名字，保持
                        // SegStart 等下一个 ident（曾把 `...args: T =>`
                        // 误判为非参数列表，箭头整体解析失败）。
                        "..." if st == St::SegStart => {}
                        ":" | "=" | "?" if st == St::InIdent => st = St::InTail,
                        // 参数类型里的函数类型 `cb: () => T`：`=>` 不
                        // 终止参数段（曾让 `shape: () => Shape` 的
                        // 泛型箭头整体解析失败，类成员吞到 EOF）。
                        "=>" => st = St::InTail,
                        _ => {
                            if st != St::InTail {
                                return false;
                            }
                        }
                    }
                } else if ident_like {
                    if st == St::SegStart {
                        st = St::InIdent;
                    } else if st != St::InTail {
                        return false;
                    }
                }
            }
            i += 1;
        }
        true
    }

    fn match_paren(&self, i: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut j = i;
        while j < self.toks.len() {
            let t = &self.toks[j];
            if t.kind == TokKind::Eof {
                return None;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => depth += 1,
                    ")" => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j);
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }
        None
    }

    /// `<…>` 泛型参数的配对结束下标（支持嵌套与 `>>`/`>>>`）。
    fn match_angle(&self, i: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut j = i;
        while j < self.toks.len() {
            let t = &self.toks[j];
            if t.kind == TokKind::Eof {
                return None;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "<" => depth += 1,
                    ">" => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j);
                        }
                    }
                    ">>" => {
                        depth -= 2;
                        if depth <= 0 {
                            return Some(j);
                        }
                    }
                    ">>>" => {
                        depth -= 3;
                        if depth <= 0 {
                            return Some(j);
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }
        None
    }

    fn parse_arrow_body(&mut self) -> TsId {
        if self.is_punct("{") {
            self.parse_block()
        } else {
            self.parse_assign_expr()
        }
    }

    fn parse_conditional(&mut self) -> TsId {
        let cond = self.parse_binary(2);
        if self.is_punct("?") {
            self.advance();
            let then = self.parse_assign_expr();
            if self.is_punct(":") {
                self.advance();
            } else {
                if std::env::var("CURE_DBG_COND").is_ok() {
                    eprintln!(
                        "[cond] L{} C{} then={:?} next={:?}",
                        self.tk().line as usize,
                        self.tk().col as usize,
                        cure_engine::kind::NodeKind::from(self.ast.kind(then)),
                        self.tk().text
                    );
                }
                self.err_here("expected : in conditional");
            }
            let els = self.parse_assign_expr();
            return self.ast.ternary(cond, then, els);
        }
        cond
    }

    fn parse_binary(&mut self, min_prec: u8) -> TsId {
        let mut left = self.parse_unary();
        loop {
            let op = if self.tk().kind == TokKind::Punct {
                self.tk().text.clone()
            } else if self.tk().kind == TokKind::Ident
                && matches!(self.tk().text.as_str(), "in" | "instanceof")
            {
                self.tk().text.clone()
            } else {
                break;
            };
            let Some((prec, binop)) = bin_info(&op) else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = if op == "**" {
                self.parse_binary(prec) // 右结合
            } else {
                self.parse_binary(prec + 1)
            };
            left = self.ast.bin(binop, left, right);
        }
        left
    }

    fn parse_unary(&mut self) -> TsId {
        if self.depth >= MAX_NEST {
            self.too_deep = true;
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.depth += 1;
        let out = self.parse_unary_inner();
        self.depth -= 1;
        out
    }

    fn parse_unary_inner(&mut self) -> TsId {
        let t = self.tk().clone();
        if t.kind == TokKind::Punct {
            let op = match t.text.as_str() {
                "!" => UnOp::Not,
                "~" => UnOp::BitNot,
                "+" => UnOp::Plus,
                "-" => UnOp::Neg,
                "++" => UnOp::PreInc,
                "--" => UnOp::PreDec,
                _ => return self.parse_postfix(),
            };
            self.advance();
            let e = self.parse_unary();
            return self.ast.un(op, e);
        }
        if t.kind == TokKind::Ident {
            match t.text.as_str() {
                "typeof" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::TypeOf, e);
                }
                "void" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Void, e);
                }
                "delete" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Delete, e);
                }
                "await" if !self.await_is_ident() => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Await, e);
                }
                _ => {}
            }
        }
        self.parse_postfix()
    }

    /// 在指定 [Await] 语境里执行闭包，退出时恢复（函数边界重置）。
    fn enter_async<T>(&mut self, is_async: bool, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.async_ctx;
        self.async_ctx = is_async;
        let out = f(self);
        self.async_ctx = saved;
        out
    }

    /// `await` 是否是普通标识符（sloppy 非异步语境：`await(x)` 调用/
    /// `await.x` 成员/`await = x` 赋值/`await, await)` 参数位——
    /// test262 非 async 文件大量使用 await 作名）。
    fn await_is_ident(&self) -> bool {
        // async 语境（async 函数体/async 方法/async 箭头/module 顶层）
        // 里 `await` 是运算符，即使后随 `(`——`await (x)` 是 Await 的
        // 括号操作数；非 async 语境里 `await (x)` 是名为 await 的函数
        // 调用（`function await(){}; await(null)`，Test262
        // private-field-rhs-await-absent 抓获）。
        if self.async_ctx {
            return false;
        }
        // 只看前瞻形态（换行接操作数 `await \n x` 是合法 await——
        // 不可当标识符判据）
        self.is_punct_at(1, "(")
            || self.is_punct_at(1, ".")
            || self.is_punct_at(1, "=")
            || self.is_punct_at(1, ",")
            || self.is_punct_at(1, ";")
            || self.is_punct_at(1, ")")
            || self.is_kw_at(1, "instanceof")
            || self.is_kw_at(1, "in")
            || self.is_punct_at(1, "?")
            || self.is_punct_at(1, ":")
    }

    /// 调用/new 的 TS 类型实参 `<…>` 前瞻：只有后续形态明确表明
    /// 是泛型（`(`、标签模板、成员链、语句结束、`as`/`satisfies`）
    /// 才返回 `>` 之后的下标；`a < b > c` 仍按比较解析。
    fn generic_args_ahead(&self) -> Option<usize> {
        if !self.ts_syntax || !self.is_punct("<") {
            return None;
        }
        let close = self.match_angle(self.pos)?;
        let after = close + 1;
        let t = self.toks.get(after)?;
        let ok = match t.kind {
            TokKind::Eof => true,
            TokKind::Punct => matches!(
                t.text.as_str(),
                "(" | ")" | "]" | "}" | ";" | "," | "." | "?." | "!"
            ),
            TokKind::TemplateQuasi { cont: false, .. } => true,
            TokKind::Ident => matches!(
                t.text.as_str(),
                "as" | "satisfies" | "instanceof" | "in"
            ),
            _ => false,
        };
        if ok {
            Some(after)
        } else {
            None
        }
    }

    fn parse_postfix(&mut self) -> TsId {
        let mut e = self.parse_primary_with_new();
        loop {
            let t = self.tk().clone();
            // 泛型调用/实例化表达式：`f<T>(x)`、`f<T>`、`tag<T>\`x\``。
            // 类型实参运行时擦除，只吞掉不建节点。
            if t.kind == TokKind::Punct && t.text == "<" {
                if let Some(after) = self.generic_args_ahead() {
                    self.pos = after;
                    continue;
                }
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "." => {
                        self.advance();
                        let name = self.expect_member_name();
                        e = self.ast.member(e, &name, false);
                    }
                    "?." => {
                        self.advance();
                        if self.is_punct("<") {
                            if let Some(after) = self.generic_args_ahead() {
                                self.pos = after;
                            }
                        }
                        if self.is_punct("(") {
                            let args = self.parse_call_args();
                            e = self.ast.opt_call(e, args);
                        } else if self.is_punct("[") {
                            self.advance();
                            let idx = self.parse_expr();
                            if self.is_punct("]") {
                                self.advance();
                            } else {
                                self.err_here("expected ]");
                            }
                            e = self.ast.index(e, idx, true);
                        } else {
                            let name = self.expect_member_name();
                            e = self.ast.member(e, &name, true);
                        }
                    }
                    "[" => {
                        self.advance();
                        let idx = self.parse_expr();
                        if self.is_punct("]") {
                            self.advance();
                        } else {
                            self.err_here("expected ]");
                        }
                        e = self.ast.index(e, idx, false);
                    }
                    "(" => {
                        let args = self.parse_call_args();
                        e = self.ast.call(e, args);
                    }
                    "!" if !t.newline_before => {
                        // TS 非空断言
                        self.advance();
                        e = self.ast.ts_non_null(e);
                    }
                    "++" | "--" if !t.newline_before => {
                        let op = if t.text == "++" { UnOp::PostInc } else { UnOp::PostDec };
                        self.advance();
                        e = self.ast.un(op, e);
                    }
                    _ => break,
                }
                continue;
            }
            if matches!(t.kind, TokKind::TemplateQuasi { cont: false, .. }) {
                // 标签模板 `tag\`x\``（MemberExpression 后缀）
                e = self.parse_template_after(Some(e));
                continue;
            }
            if self.ts_syntax
                && t.kind == TokKind::Ident
                && matches!(t.text.as_str(), "as" | "satisfies")
                && !t.newline_before
                && !self.is_punct_at(1, "<")
            {
                let is_satisfies = t.text == "satisfies";
                self.advance();
                let ts = self.pos;
                self.skip_type_region();
                let ty = self.raw_between(ts, self.pos);
                e = if is_satisfies {
                    self.ast.ts_satisfies(&ty, e)
                } else {
                    self.ast.ts_as(&ty, e)
                };
                continue;
            }
            // 标签模板：**新模板开始**（cont=false——插值续接段属外围
            // 模板，parse_template_after 的循环在等它，不能当 tag 附着）
            if matches!(t.kind, TokKind::TemplateQuasi { cont: false, .. }) {
                e = self.parse_template_after(Some(e));
                continue;
            }
            break;
        }
        e
    }

    fn parse_primary_with_new(&mut self) -> TsId {
        if self.is_kw("new") {
            self.advance();
            // new.target 元属性
            if self.is_punct(".") && self.tk_at(1).text == "target" {
                self.advance();
                self.advance();
                return self.ast.raw("new.target");
            }
            // 嵌套 new（new new X()()）：操作数以 new 开头递归
            let mut e = if self.is_kw("new") {
                self.parse_primary_with_new()
            } else {
                self.parse_primary()
            };
            loop {
                if self.is_punct(".") {
                    self.advance();
                    let name = self.expect_member_name();
                    e = self.ast.member(e, &name, false);
                } else if self.is_punct("[") {
                    self.advance();
                    let idx = self.parse_expr();
                    if self.is_punct("]") {
                        self.advance();
                    } else {
                        self.err_here("expected ]");
                    }
                    e = self.ast.index(e, idx, false);
                } else {
                    break;
                }
            }
            // `new Foo<T>(…)` / `new Foo<T>`：类型实参擦除。
            if self.is_punct("<") {
                if let Some(after) = self.generic_args_ahead() {
                    self.pos = after;
                }
            }
            // `new tag\`x\``：标签模板是 MemberExpression，属于 callee
            while matches!(self.tk().kind, TokKind::TemplateQuasi { cont: false, .. }) {
                e = self.parse_template_after(Some(e));
            }
            let args = if self.is_punct("(") {
                self.parse_call_args()
            } else {
                vec![]
            };
            return self.ast.new_(e, args);
        }
        self.parse_primary()
    }

    fn parse_call_args(&mut self) -> Vec<TsId> {
        if !self.is_punct("(") {
            self.err_here("expected (");
            return vec![];
        }
        self.advance();
        let mut args = Vec::new();
        while !self.is_punct(")") && !self.at_eof() {
            if self.is_punct("...") {
                self.advance();
                let e = self.parse_assign_expr();
                args.push(self.ast.spread(e));
            } else {
                args.push(self.parse_assign_expr());
            }
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        args
    }

    /// 模板字面量解析（当前 token 是首 quasi）。tag = Some 时构造标签模板。
    fn parse_template_after(&mut self, tag: Option<TsId>) -> TsId {
        let mut quasis: Vec<String> = Vec::new();
        let mut interps: Vec<TsId> = Vec::new();
        let first = self.advance();
        let mut is_end = match first.kind {
            TokKind::TemplateQuasi { is_end, .. } => is_end,
            _ => true,
        };
        quasis.push(first.text.clone());
        while !is_end {
            let e = self.parse_expr();
            interps.push(e);
            let t = self.tk().clone();
            match t.kind {
                TokKind::TemplateQuasi { is_end: ie, .. } => {
                    self.advance();
                    quasis.push(t.text.clone());
                    is_end = ie;
                }
                _ => {
                    self.err_here("expected template continuation");
                    break;
                }
            }
        }
        match tag {
            Some(t) => self.ast.tagged_template(t, quasis, interps),
            None => self.ast.template(quasis, interps),
        }
    }

    fn expect_member_name(&mut self) -> String {
        let t = self.tk().clone();
        if t.kind == TokKind::Ident || t.kind == TokKind::Num {
            self.advance();
            t.text
        } else if t.kind == TokKind::Punct && t.text == "#" {
            self.advance();
            let n = self.tk().text.clone();
            if self.tk().kind == TokKind::Ident {
                self.advance();
            }
            format!("#{n}")
        } else {
            self.err_here("expected member name");
            String::new()
        }
    }

    /// 类型区域扫描（as/satisfies/注解后）：平衡泛型括号，停在表达式边界。
    /// 跳过返回类型注解，停在函数体 `{`（或重载 `;`）。
    /// 返回类型可含对象/映射/函数/条件类型：`Promise<{ [K in keyof T]:
    /// Awaited<T[K]> }>` 的第一个 `{` 属类型，不是身体；而
    /// `(): { a: T } {` 的第二个 `{` 才是身体。
    fn skip_return_type(&mut self) {
        let (mut paren, mut brack, mut brace, mut angle) = (0i32, 0i32, 0i32, 0i32);
        let mut prev: Option<String> = None;
        loop {
            if self.at_eof() {
                return;
            }
            let t = self.tk().clone();
            if paren == 0 && brack == 0 && brace == 0 && angle == 0 {
                if self.ends_type_at_new_stmt() {
                    return;
                }
                if t.kind == TokKind::Punct {
                    match t.text.as_str() {
                        ";" | ")" | "]" | "}" => return,
                        "{" => {
                            let type_pos = match prev.as_deref() {
                                None => true,
                                Some(p) => matches!(
                                    p,
                                    "=>" | "|" | "&" | "<" | "," | "(" | "[" | ":" | "?"
                                        | "extends" | "keyof" | "typeof" | "infer" | "in"
                                        | "is" | "as" | "readonly" | "new" | "abstract"
                                        | "asserts"
                                ),
                            };
                            if type_pos {
                                brace += 1;
                                prev = Some(t.text.clone());
                                self.advance();
                                continue;
                            }
                            return;
                        }
                        _ => {}
                    }
                }
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => paren += 1,
                    ")" => paren -= 1,
                    "[" => brack += 1,
                    "]" => brack -= 1,
                    "{" => brace += 1,
                    "}" => brace -= 1,
                    "<" => angle += 1,
                    ">" => angle -= 1,
                    ">>" => angle -= 2,
                    ">>>" => angle -= 3,
                    _ => {}
                }
            }
            prev = Some(t.text.clone());
            self.advance();
        }
    }

    fn skip_type_region(&mut self) {
        let mut depth = 0i32;
        while !self.at_eof() {
            if depth == 0 && self.ends_type_at_new_stmt() {
                return;
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "<" | "(" | "[" | "{" => {
                        depth += 1;
                        self.advance();
                    }
                    ">" => {
                        depth -= 1;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ">>" => {
                        depth -= 2;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ">>>" => {
                        depth -= 3;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ")" | "]" | "}" => {
                        if depth == 0 {
                            return;
                        }
                        depth -= 1;
                        self.advance();
                    }
                    ";" | "," | "=" | "?" | ":" if depth == 0 => { return; },
                    _ => { self.advance(); },
                }
                continue;
            }
            if t.kind == TokKind::Ident {
                let nxt = self.tk_at(1).clone();
                let stop_after_ident = nxt.kind == TokKind::Punct
                    && matches!(nxt.text.as_str(), ")" | "]" | "}" | "," | ";")
                    && depth == 0;
                self.advance();
                if stop_after_ident {
                    return;
                }
                continue;
            }
            if t.kind == TokKind::Str || t.kind == TokKind::Num {
                self.advance();
                continue;
            }
            return;
        }
    }

    fn parse_primary(&mut self) -> TsId {
        let t = self.tk().clone();
        match t.kind {
            TokKind::Num => {
                self.advance();
                let lit = parse_num_lit(&t.text);
                self.ast.lit(lit)
            }
            TokKind::Str => {
                self.advance();
                match decode_str_lit(&t.text) {
                    Some(v) => self.ast.lit(JsLit::Str(v)),
                    // 孤立代理项：Rust String 无法表示——保留原文转义
                    //（JSON.stringify('\uD800') 等语义依赖精确代码单元）
                    None => self.ast.raw(&t.text),
                }
            }
            TokKind::Regex => {
                self.advance();
                self.ast.lit(JsLit::Regex(t.text))
            }
            TokKind::TemplateQuasi { .. } => self.parse_template_after(None),
            TokKind::Ident => match t.text.as_str() {
                "function" => {
                    let st = self.pos;
                    self.parse_function_expr(false, st)
                }
                "async" if self.tk_at(1).text == "function" && !self.tk_at(1).newline_before => {
                    let st = self.pos;
                    self.advance(); // async（parse_function_expr 消费 function）
                    self.parse_function_expr(true, st)
                }
                "class" if self.tk_at(1).kind == TokKind::Ident || self.tk_at(1).kind == TokKind::Punct => {
                    // 类表达式（class / class Name …）
                    if self.tk_at(1).kind == TokKind::Ident
                        || self.is_punct_at(1, "extends")
                        || self.is_punct_at(1, "{")
                        || self.is_kw_at(1, "extends")
                    {
                        let s = self.pos;
                        self.skip_class_expr();
                        let raw = self.raw_between(s, self.pos);
                        self.ast.class_expr(&raw)
                    } else {
                        self.advance();
                        self.ast.var(&t.text)
                    }
                }
                "yield"
                    if self.is_punct_at(1, ";")
                        || self.is_punct_at(1, ")")
                        || self.is_punct_at(1, "}")
                        || self.is_punct_at(1, "]")
                        || self.is_punct_at(1, ",")
                        || self.is_punct_at(1, ":")
                        || self.tk_at(1).kind == TokKind::Eof =>
                {
                    // `'' in yield` / `yield ? yield : yield` 中的
                    // 无操作数 yield（YieldExpression 是 AssignmentExpression
                    // ——可作二元右操作数/三元条件；曾直接语法错误）
                    let s = self.pos;
                    self.advance();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "this" => {
                    self.advance();
                    self.ast.this()
                }
                "super" => {
                    self.advance();
                    let mut raw = String::from("super");
                    if self.is_punct(".") {
                        self.advance();
                        raw.push('.');
                        raw.push_str(&self.expect_member_name());
                    } else if self.is_punct("[") {
                        let br = self.pos;
                        self.advance();
                        let _ = self.parse_expr();
                        if self.is_punct("]") {
                            self.advance();
                        }
                        // 只追加方括号段：raw_between(s, …) 含 "super" 本身，
                        // 曾产出 `supersuper['x']`（Test262 super 家族抓获）。
                        raw.push_str(&self.raw_between(br, self.pos));
                    } else if self.is_punct("(") {
                        let ps = self.pos;
                        self.skip_parens();
                        raw.push_str(&self.raw_between(ps, self.pos));
                    }
                    self.ast.raw(&raw)
                }
                "true" | "false" => {
                    self.advance();
                    self.ast.lit(JsLit::Bool(t.text == "true"))
                }
                "null" => {
                    self.advance();
                    self.ast.lit(JsLit::Null)
                }
                _ => {
                    self.advance();
                    self.ast.var(&t.text)
                }
            },
            TokKind::Punct => match t.text.as_str() {
                "(" => {
                    self.advance();
                    let e = self.parse_expr();
                    if self.is_punct(")") {
                        self.advance();
                    } else {
                        self.err_here("expected )");
                    }
                    self.ast.paren(e)
                }
                "[" => {
                    self.advance();
                    let mut elems = Vec::new();
                    while !self.is_punct("]") && !self.at_eof() {
                        // 稀疏数组 elision（`[, ,42,,]`——空洞位占位；
                        // 打印原样输出空槽保持长度语义）
                        if self.is_punct(",") {
                            elems.push(self.ast.raw(""));
                            self.advance();
                            continue;
                        }
                        if self.is_punct("...") {
                            self.advance();
                            let e = self.parse_assign_expr();
                            elems.push(self.ast.spread(e));
                        } else {
                            elems.push(self.parse_assign_expr());
                        }
                        if self.is_punct(",") {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if self.is_punct("]") {
                        self.advance();
                    } else {
                        self.err_here("expected ]");
                    }
                    self.ast.array_lit(elems)
                }
                "{" => self.parse_obj_lit(),
                "<" => {
                    // JSX 元素（.tsx）——原文捕获
                    let raw = self.capture_jsx();
                    self.ast.raw(&raw)
                }
                "#" => {
                    // 私有名表达式（`#f in obj`）：`#` 是名字的一部分，
                    // 打印必须保留（曾被剥成普通标识符 f——private-field
                    // presence 族抓获）。
                    self.advance();
                    let mut n = String::from("#");
                    if self.tk().kind == TokKind::Ident {
                        n.push_str(&self.advance().text);
                    }
                    self.ast.raw(&n)
                }
                _ => {
                    self.err_here(&format!("unexpected token {:?}", t.text));
                    self.advance();
                    self.ast.raw(&t.text)
                }
            },
            TokKind::Eof => {
                self.err_here("unexpected EOF");
                self.ast.empty()
            }
        }
    }

    fn is_kw_at(&self, off: usize, w: &str) -> bool {
        let t = self.tk_at(off);
        t.kind == TokKind::Ident && t.text == w
    }

    fn parse_function_expr(&mut self, is_async: bool, src_start: usize) -> TsId {
        self.advance(); // function（async 已在外层 advance）
        let mut is_gen = false;
        if self.is_punct("*") {
            is_gen = true;
            self.advance();
        }
        let name = if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
            self.advance().text
        } else {
            String::new()
        };
        let mut generics = String::new();
        if self.is_punct("<") {
            let gs = self.pos;
            self.skip_angle_balanced();
            generics = self.raw_between(gs, self.pos).trim().to_string();
        }
        if !self.is_punct("(") {
            self.err_here("expected ( after function");
            return self.ast.raw("function");
        }
        // `function (` vs `function(`：Function.prototype.toString 与
        // 计算属性键依赖原文，保留空档（Test262 cpn-class 家族抓获）。
        let space = self.pos > 0
            && self.tk().start as usize > self.toks[self.pos - 1].end as usize;
        let ps = self.pos;
        self.skip_parens();
        let full = self.raw_between(ps, self.pos);
        let inner = full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&full)
            .to_string();
        let mut ret_raw = String::new();
        if self.is_punct(":") {
            let rs = self.pos;
            self.skip_return_type();
            ret_raw = self.raw_between(rs, self.pos);
        }
        let body = if self.is_punct("{") {
            self.enter_async(is_async, |p| p.parse_block())
        } else {
            self.err_here("expected { in function expression");
            self.ast.block(vec![])
        };
        let id = if generics.is_empty() && ret_raw.is_empty() {
            self.ast.lambda_named(&inner, body, false, is_async, is_gen, &name, space)
        } else {
            self.ast.lambda_named_generic_ret(
                &generics,
                &ret_raw,
                &inner,
                body,
                false,
                is_async,
                is_gen,
                &name,
                space,
            )
        };
        self.set_span_here(id, src_start);
        id
    }

    fn parse_obj_lit(&mut self) -> TsId {
        let open = self.pos;
        self.advance(); // {
        let mut props: Vec<String> = Vec::new();
        let mut values: Vec<TsId> = Vec::new();
        while !self.is_punct("}") && !self.at_eof() {
            if self.is_punct("...") {
                self.advance();
                let e = self.parse_assign_expr();
                props.push("...".into());
                values.push(e);
            } else {
                let (p, v) = self.parse_obj_prop();
                props.push(p);
                values.push(v);
            }
            // 解构模式信号：键后是 `=`（默认值 `{x = v}`——对象字面量
            // 语法无此形态；声明/赋值目标的模式）→ 整段 `{…}` 原文
            //（绑定名不可见 → 不透明，语义保真）
            if self.is_punct("=") {
                self.pos = open;
                self.skip_braces();
                return self.ast.raw(&self.raw_between(open, self.pos));
            }
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        let id = self.ast.obj_lit(props, values);
        self.set_span_here(id, open);
        id
    }

    fn parse_obj_prop(&mut self) -> (String, TsId) {
        let t = self.tk().clone();
        // 生成器/异步生成器方法前缀：`*f()` / `async *f()` / `async f()`
        let mut gen_prefix = String::new();
        if t.kind == TokKind::Ident && t.text == "async" && self.is_punct_at(1, "*") {
            self.advance(); // async
            gen_prefix.push_str("async ");
        }
        if self.is_punct("*") {
            gen_prefix.push('*');
            self.advance();
        }
        let t = if gen_prefix.is_empty() { t } else { self.tk().clone() };
        // 访问器/异步方法：get x() / set [k]() / async 0()——整体原文
        //（前缀与键组合形态过多，v1 原文保真）。前瞻：键（Ident/`[expr]`/
        // Num/Str）之后的 `(`——计算键需跳到配对 `]` 后检查
        if gen_prefix.is_empty()
            && t.kind == TokKind::Ident
            && matches!(t.text.as_str(), "get" | "set" | "async")
            && !self.tk_at(1).newline_before
            && self.accessor_ahead()
        {
            let s = self.pos;
            self.skip_obj_method();
            let raw = self.raw_between(s, self.pos);
            let raw = raw.trim_end().trim_end_matches(';').to_string();
            return (raw, self.ast.raw(""));
        }
        // 键（ident / str / num / 计算键）
        let key = match self.tk().clone() {
            Token { kind: TokKind::Ident, text, .. } => {
                self.advance();
                text
            }
            Token { kind: TokKind::Str | TokKind::Num, text, .. } => {
                self.advance();
                text
            }
            Token { kind: TokKind::Punct, text, .. } if text == "[" => {
                // 计算键 [expr]：raw 不含两端括号（含进去打印层再包
                // 一层——往返每轮多一层 `[`）
                let s = self.pos;
                self.advance();
                let _ = self.parse_assign_expr();
                if self.is_punct("]") {
                    self.advance();
                } else {
                    self.err_here("expected ]");
                }
                format!("[{}]", self.raw_between(s + 1, self.pos - 1))
            }
            _ => {
                self.err_here("expected property key");
                if !self.at_eof() {
                    self.advance();
                }
                String::new()
            }
        };
        // 方法（任意键形 + 泛型 + 返回类型 + `(`）：name(params): T { body }
        let mut meth_generics = String::new();
        if self.is_punct("<") {
            let gs = self.pos;
            self.skip_angle_balanced();
            meth_generics = self.raw_between(gs, self.pos);
        }
        if self.is_punct("(") && !self.tk().newline_before {
            let ps = self.pos;
            self.skip_parens();
            let params_full = self.raw_between(ps, self.pos);
            let inner = params_full
                .strip_prefix('(')
                .and_then(|x| x.strip_suffix(')'))
                .unwrap_or(&params_full)
                .to_string();
            let mut ret_raw = String::new();
            if self.is_punct(":") {
                let rs = self.pos;
                self.skip_return_type();
                ret_raw = self.raw_between(rs, self.pos);
            }
            let body = if self.is_punct("{") {
                let is_async = gen_prefix.starts_with("async");
                self.enter_async(is_async, |p| p.parse_block())
            } else {
                self.err_here("expected { in object method");
                self.ast.block(vec![])
            };
            let lam = self.ast.lambda(&inner, body, false, false, !gen_prefix.is_empty());
            return (
                format!("{gen_prefix}{key}{meth_generics}{params_full}{ret_raw}"),
                lam,
            );
        }
        if !self.is_punct(":") {
            // 简写 { x }（仅 ident 键；gen 前缀后无方法是语法错——
            // 容错按原文残段处理）
            if t.kind == TokKind::Ident && gen_prefix.is_empty() {
                return (key.clone(), self.ast.var(&t.text));
            }
            if gen_prefix.is_empty() {
                return (key, self.ast.raw(""));
            }
            return (format!("{gen_prefix}{key}"), self.ast.raw(""));
        }
        self.advance(); // :
        let v = self.parse_assign_expr();
        (format!("{gen_prefix}{key}"), v)
    }

    /// 扫过对象方法（`[kw] key(…) { … }`）到平衡体尾——键支持
    /// Ident/计算键 `[expr]`/Num/Str 全形态。
    fn skip_obj_method(&mut self) {
        self.advance(); // get/set/async
        if self.is_punct("[") {
            self.skip_brackets();
        } else if matches!(self.tk().kind, TokKind::Ident | TokKind::Num | TokKind::Str) {
            self.advance();
        }
        if self.is_punct("*") {
            self.advance();
        }
        self.skip_parens();
        if self.is_punct(":") {
            self.skip_return_type();
        }
        if self.is_punct("{") {
            self.skip_braces();
        }
        if self.is_punct(";") {
            self.advance();
        }
    }

    /// 访问器前瞻：get/set/async + 键形态 + `(`。
    fn accessor_ahead(&self) -> bool {
        if matches!(
            self.tk_at(1).kind,
            TokKind::Ident | TokKind::Num | TokKind::Str
        ) {
            return self.is_punct_at(2, "(");
        }
        if self.is_punct_at(1, "[") {
            // 计算键：找配对 `]` 后的 `(`
            let mut depth = 0i32;
            let mut i = self.pos + 1;
            while i < self.toks.len() {
                let t = &self.toks[i];
                if t.kind == TokKind::Eof {
                    return false;
                }
                if t.kind == TokKind::Punct {
                    match t.text.as_str() {
                        "[" => depth += 1,
                        "]" => {
                            depth -= 1;
                            if depth == 0 {
                                return self
                                    .toks
                                    .get(i + 1)
                                    .is_some_and(|n| n.kind == TokKind::Punct && n.text == "(");
                            }
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
        }
        false
    }

    /// 类表达式边界扫描。
    fn skip_class_expr(&mut self) {
        self.advance(); // class
        if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
            self.advance();
        }
        // 找类体 `{`：**括号/方括号深度 0** 的 `{` 才是类体。extends
        // 里的箭头/函数体处于括号内（`class extends (() => {}) {}`）——
        // 曾把箭头体当类体，类表达式原文被截断（Test262 heritage-arrow
        // 家族抓获）。
        let mut pd = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" => {
                        pd += 1;
                        self.advance();
                        continue;
                    }
                    ")" | "]" => {
                        if pd == 0 {
                            return;
                        }
                        pd -= 1;
                        self.advance();
                        continue;
                    }
                    ";" | "," if pd == 0 => return,
                    "{" if pd == 0 => break,
                    _ => {}
                }
            } else if pd == 0 && t.kind == TokKind::Ident && t.text == "class" {
                // extends 里的类表达式（`class extends class { } { … }`）
                self.skip_class_expr();
                continue;
            } else if pd == 0 && t.kind == TokKind::Ident && t.text == "function" {
                // extends 里的函数表达式（`class extends function(){} {}`）
                self.advance();
                if self.is_punct("*") {
                    self.advance();
                }
                if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
                    self.advance();
                }
                if self.is_punct("(") {
                    self.skip_parens();
                }
                if self.is_punct("{") {
                    self.skip_braces();
                }
                continue;
            }
            self.advance();
        }
        if self.is_punct("{") {
            let mut depth = 0i32;
            loop {
                if self.at_eof() {
                    return;
                }
                if self.is_punct("{") {
                    depth += 1;
                } else if self.is_punct("}") {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        return;
                    }
                }
                self.advance();
            }
        }
    }

    /// JSX 元素原文捕获（平衡标签/插值/字符串）。
    fn capture_jsx(&mut self) -> String {
        let start = self.pos;
        let mut stack: Vec<String> = Vec::new();
        loop {
            if self.at_eof() {
                break;
            }
            let t = self.tk().clone();
            match t.kind {
                TokKind::Punct => match t.text.as_str() {
                    "<" => {
                        self.advance();
                        if self.is_punct("/") {
                            self.advance();
                            let name = self.expect_member_name();
                            if self.is_punct(">") {
                                self.advance();
                            }
                            loop {
                                match stack.pop() {
                                    Some(n) if n == name => break,
                                    Some(_) => continue,
                                    None => break,
                                }
                            }
                            if stack.is_empty() {
                                break;
                            }
                        } else {
                            let name = if self.is_punct(">") {
                                String::new()
                            } else {
                                self.expect_member_name()
                            };
                            stack.push(name.clone());
                            // 属性扫描到 `>` / `/>`
                            let mut attr_depth = 0i32;
                            loop {
                                if self.at_eof() {
                                    return self.raw_between(start, self.pos);
                                }
                                let a = self.tk().clone();
                                if a.kind == TokKind::Punct {
                                    if a.text == "/" && self.is_punct_at(1, ">") && attr_depth == 0 {
                                        // `/>` 自闭合（两 token 形态）
                                        self.advance();
                                        self.advance();
                                        stack.pop();
                                        if stack.is_empty() {
                                            return self.raw_between(start, self.pos);
                                        }
                                        break;
                                    }
                                    match a.text.as_str() {
                                        "{" => {
                                            attr_depth += 1;
                                            self.advance();
                                            if attr_depth == 1 {
                                                let _ = self.parse_expr();
                                                if self.is_punct("}") {
                                                    self.advance();
                                                    attr_depth -= 1;
                                                }
                                            }
                                        }
                                        "}" => {
                                            attr_depth -= 1;
                                            self.advance();
                                        }
                                        ">" if attr_depth == 0 => {
                                            self.advance();
                                            break;
                                        }
                                        _ => { self.advance(); },
                                    }
                                } else {
                                    self.advance();
                                }
                            }
                        }
                    }
                    "{" => {
                        self.advance();
                        let _ = self.parse_expr();
                        if self.is_punct("}") {
                            self.advance();
                        }
                    }
                    _ => { self.advance(); },
                },
                _ => { self.advance(); },
            }
        }
        self.raw_between(start, self.pos)
    }
}

/// 数值字面量 → JsLit（十六/八/二进制与分隔符原文保留）。
fn parse_num_lit(text: &str) -> JsLit {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    if clean.ends_with('n') {
        return JsLit::BigInt(text.to_string());
    }
    for (prefix, radix) in [("0x", 16u32), ("0X", 16), ("0o", 8), ("0O", 8), ("0b", 2), ("0B", 2)] {
        if let Some(digits) = clean.strip_prefix(prefix) {
            if let Ok(v) = i64::from_str_radix(digits, radix) {
                return JsLit::NumRaw { text: text.to_string(), val: NumVal::Int(v) };
            }
            break;
        }
    }
    if !clean.contains('.') && !clean.contains('e') && !clean.contains('E') {
        // LegacyOctalIntegerLiteral（sloppy）：`070` = 56；`08`/`09`
        // 按十进制（NonOctalDecimalIntegerLiteral）
        if clean.len() > 1
            && clean.starts_with('0')
            && clean[1..].chars().all(|c| matches!(c, '0'..='7'))
        {
            if let Ok(v) = i64::from_str_radix(&clean[1..], 8) {
                return JsLit::Int(v);
            }
        }
        if let Ok(v) = clean.parse::<i64>() {
            return JsLit::Int(v);
        }
    }
    if let Ok(v) = clean.parse::<f64>() {
        if v.is_finite() {
            return JsLit::Double(v);
        }
    }
    JsLit::NumRaw { text: text.to_string(), val: NumVal::Double(f64::NAN) }
}

/// 字符串字面量解码（原文含引号）。
fn decode_str_lit(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 {
        return Some(raw.to_string());
    }
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000c}'),
            Some('v') => out.push('\u{000b}'),
            // LegacyOctalEscapeSequence（sloppy 字符串）：`\07` = U+0007。
            // 首字符 0-3 允许最多 3 位八进制，4-7 最多 2 位（≤ 0xFF）。
            Some(c @ '0'..='7') => {
                let mut v = c.to_digit(8).unwrap();
                let mut n = 1;
                let max = if c <= '3' { 3 } else { 2 };
                while n < max {
                    let Some(&d) = chars.peek() else { break };
                    match d {
                        '0'..='7' => {
                            let nv = v * 8 + d.to_digit(8).unwrap();
                            if nv > 0xFF {
                                break;
                            }
                            v = nv;
                            n += 1;
                            chars.next();
                        }
                        _ => break,
                    }
                }
                if let Some(ch) = char::from_u32(v) {
                    out.push(ch);
                }
            }
            Some('x') => {
                let h: String = (0..2).filter_map(|_| chars.next()).collect();
                if let Ok(v) = u32::from_str_radix(&h, 16) {
                    if let Some(ch) = char::from_u32(v) {
                        out.push(ch);
                    }
                }
            }
            Some('u') => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let h: String = chars.by_ref().take_while(|c| *c != '}').collect();
                    if let Ok(v) = u32::from_str_radix(&h, 16) {
                        match char::from_u32(v) {
                            Some(ch) => out.push(ch),
                            // `\u{D800}` 等孤立代理项 → 放弃解码，调用方保留原文
                            None => return None,
                        }
                    }
                } else {
                    let h: String = (0..4).filter_map(|_| chars.next()).collect();
                    if let Ok(v) = u32::from_str_radix(&h, 16) {
                        // 代理对：`\uD834\uDD85` 必须合成 U+1D185。
                        // 逐半解码会把两个高位/低位代理都丢弃（char 无法
                        // 表示孤立代理）——字符串内容静默缩水（Test262
                        // intl402 toLocaleLowerCase Turkish 抓获：
                        // MUSICAL SYMBOL COMBINING DOIT 消失）。
                        if (0xD800..=0xDBFF).contains(&v) {
                            let mut ahead = chars.clone();
                            if ahead.next() == Some('\\') && ahead.next() == Some('u') {
                                let lh: String = (0..4).filter_map(|_| ahead.next()).collect();
                                if let Ok(lo) = u32::from_str_radix(&lh, 16) {
                                    if (0xDC00..=0xDFFF).contains(&lo) {
                                        let cp = 0x10000 + ((v - 0xD800) << 10) + (lo - 0xDC00);
                                        if let Some(ch) = char::from_u32(cp) {
                                            for _ in 0..6 {
                                                chars.next();
                                            }
                                            out.push(ch);
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        match char::from_u32(v) {
                            Some(ch) => out.push(ch),
                            // 孤立代理项（无配对低位代理）→ 原文保留
                            None => return None,
                        }
                    }
                }
            }
            Some('\n') => {}
            // LineContinuation：反斜杠后可接 CR / CRLF / U+2028 /
            // U+2029——整段删除（曾把 CR 原样留下，字符串内容变长）
            Some('\r') => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            Some('\u{2028}') | Some('\u{2029}') => {}
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_ok(src: &str) -> ParseOutcome {
        let out = parse(src);
        assert!(out.errors.is_empty(), "parse errors: {:#?}", out.errors);
        out
    }

    #[test]
    fn parses_basic_module() {
        let out = roundtrip_ok("const x = 1;\nfunction f(a) { return a + x; }\n");
        assert_eq!(out.unit.items.len(), 2);
    }

    #[test]
    fn parses_arrow_and_template() {
        let out = roundtrip_ok("const f = (a, b) => a + b;\nconst g = x => x * 2;\nconst s = `x${a}y`;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn parses_control_flow() {
        let out = roundtrip_ok(
            "function f(n) {\n  if (n > 1) { return 1; } else { return 2; }\n  for (let i = 0; i < n; i++) { f(i); }\n  for (const x of [1,2]) { x; }\n  try { f(1); } catch (e) { throw e; } finally { f(0); }\n  switch (n) { case 1: return 2; default: return 3; }\n  while (true) { break; }\n}\n",
        );
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn asi_return() {
        let out = roundtrip_ok("function f() {\n  return\n  1;\n}\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn regex_vs_division_in_parser() {
        let out = roundtrip_ok("var re = /ab+c/g;\nvar d = a / b;\nvar x = /x/.test(s);\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn nested_template() {
        let out = roundtrip_ok("const s = `a${ `b${x}` }c`;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn modern_syntax_no_panic() {
        let out = parse(
            "class A extends B {\n  #x = 1;\n  static y = 2;\n  constructor(a) { super(a); }\n  async m(x) { return x; }\n  get val() { return this.#x; }\n  static { A.z = 3; }\n}\nconst o = { a: 1, b, m() { return 1; } };\nconst [p, q = 2] = arr;\nconst { a: { b: c } } = obj;\nlabel: for (;;) { break label; }\n",
        );
        let _ = out;
    }

    #[test]
    fn optional_chain_and_nullish() {
        let out = roundtrip_ok("const v = a?.b ?? d ?? e;\n");
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn jsx_raw_capture() {
        let out = roundtrip_ok(
            "const e = <div className=\"a\">hello {name}<br/><span>x</span></div>;\n",
        );
        assert!(out.errors.is_empty());
    }

    #[test]
    fn ts_annotations() {
        let out = roundtrip_ok(
            "const x: number = 1;\nlet y = z satisfies string;\nlet w = q as unknown as string;\nfunction f(a: number): string { return `${a}`; }\n",
        );
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn logical_assign_and_pow() {
        let out = roundtrip_ok("a ||= b;\nc &&= d;\ne ??= f;\ng **= 2;\nconst h = 2 ** 3 ** 2;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn destructuring_statements() {
        let out = roundtrip_ok("[a, b] = [b, a];\nconst { x, y } = point;\n");
        let _ = out;
    }

    #[test]
    fn for_await_of() {
        let out = roundtrip_ok("async function f(stream) { for await (const chunk of stream) { use(chunk); } }\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn generators_yield() {
        let out = parse("function* g() { yield 1; yield* h; const x = yield; }\n");
        let _ = out;
    }

    #[test]
    fn comma_seq_and_paren() {
        let out = roundtrip_ok("for (i = 0, j = 1; i < j; i++, j--) { f(i, j); }\n");
        assert!(out.errors.is_empty());
    }
}
