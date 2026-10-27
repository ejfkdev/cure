//! Dart 格式化器：token 级、注释保真。
//!
//! 不重建语法树，因此不会因为版本差异丢代码；所有 token（含注释、
//! 字符串原文）按原顺序输出。布局是确定性的：2 空格缩进、块/集合
//! 花括号区分、switch case 缩进、泛型尖括号紧凑、原始空行保留。

use cure_dart_ast::{BraceKind, DartAst, TokKind};

pub fn print_module(ast: &DartAst) -> String {
    let mut p = Printer {
        ast,
        out: String::new(),
        indent: 0,
        at_line_start: true,
        prev: None,
        paren_depth: 0,
        bracket_depth: 0,
        brace_depth: 0,
        generic_depth: 0,
        generic_open: None,
        brace_stack: Vec::new(),
        switches: Vec::new(),
        ternary: Vec::new(),
        paren_kind: Vec::new(),
        tight_next: false,
        tight_index: None,
    };
    for i in 0..ast.tokens.len() {
        p.emit(i);
    }
    let mut s = p.out;
    while s.ends_with('\n') {
        s.pop();
    }
    s.push('\n');
    s
}

#[derive(Default)]
struct SwitchState {
    open: usize,
    case_body: bool,
    header: Option<CaseHeader>,
    /// 开括号时的括号深度：switch 表达式的分支逗号只在同层才是分隔符
    ///（`_ => f(a, b)` 里的逗号属于调用，不能断行）。
    paren: usize,
    bracket: usize,
}

#[derive(Clone, Copy)]
struct CaseHeader {
    paren: usize,
    bracket: usize,
    brace: usize,
}

struct Printer<'a> {
    ast: &'a DartAst,
    out: String,
    indent: usize,
    at_line_start: bool,
    prev: Option<usize>,
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
    generic_depth: usize,
    generic_open: Option<usize>,
    brace_stack: Vec<usize>,
    switches: Vec<SwitchState>,
    ternary: Vec<(usize, usize, usize)>,
    /// 每个未闭合 `(` 是否是 `for` 头，以及当时的块/集合深度。
    paren_kind: Vec<(bool, usize, usize)>,
    tight_next: bool,
    tight_index: Option<usize>,
}

impl<'a> Printer<'a> {
    fn tok(&self, i: usize) -> &str {
        self.ast.text(i)
    }

    fn kind(&self, i: usize) -> TokKind {
        self.ast.tokens[i].kind
    }

    fn newline(&mut self) {
        while self.out.ends_with(' ') {
            self.out.pop();
        }
        if !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.at_line_start = true;
    }

    fn ind(&mut self) {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.out.push_str("  ");
            }
            self.at_line_start = false;
        }
    }

    fn space(&mut self) {
        if !self.at_line_start && !self.out.ends_with(' ') && !self.out.ends_with('\n') {
            self.out.push(' ');
        }
    }

    fn write(&mut self, s: &str) {
        self.ind();
        self.out.push_str(s);
    }

    fn prev_text(&self) -> Option<&str> {
        self.prev.map(|p| self.tok(p))
    }

    fn next_sig(&self, i: usize) -> Option<usize> {
        self.ast.next_sig(i)
    }

    fn prev_sig(&self, i: usize) -> Option<usize> {
        self.ast.prev_sig(i)
    }

    fn emit(&mut self, i: usize) {
        if self.tight_index != Some(i) {
            self.tight_index = None;
        }
        let t = &self.ast.tokens[i];
        // 注解结束后的原始换行：`@override` / `@Foo()` 单独占行。
        // 必须放在注释分支之前——下一条可能是跨行块注释，注释本身也要
        // 跟随注解的换行，否则二轮会把注释吸回注解行。
        if !self.at_line_start
            && t.newlines_before > 0
            && self.prev.is_some_and(|p| self.annotation_end(p))
            && !matches!(t.text.as_ref(), ";" | "," | ")" | "]")
        {
            self.newline();
        }
        if t.kind == TokKind::LineComment {
            // 源码里独立成行的注释必须重新另起一行；否则 `// ignore:`
            // 会被吸到上一条语句末尾，改变它抑制的诊断行。
            if t.newlines_before > 0 && !self.at_line_start {
                self.newline();
            }
            if self.at_line_start
                && t.newlines_before >= 2
                && !self.out.ends_with("\n\n")
            {
                self.out.push('\n');
            } else if !self.at_line_start {
                self.space();
            }
            self.write(&t.text);
            self.newline();
            self.prev = Some(i);
            return;
        }
        if t.kind == TokKind::BlockComment {
            if t.newlines_before > 0 && !self.at_line_start {
                self.newline();
            }
            if self.at_line_start
                && t.newlines_before >= 2
                && !self.out.ends_with("\n\n")
            {
                self.out.push('\n');
            } else if !self.at_line_start && self.needs_space_before(i) {
                self.space();
            }
            self.write(&t.text);
            if t.text.ends_with('\n') {
                self.at_line_start = true;
            } else if self
                .next_sig(i)
                .is_some_and(|n| self.ast.tokens[n].newlines_before > 0)
            {
                // 源码里块注释后换行：保留，避免把下一条语句并到注释所在行。
                self.newline();
            }
            self.prev = Some(i);
            return;
        }

        if self.at_line_start
            && !self.out.is_empty()
            && t.newlines_before >= 2
            && self
                .prev_sig(i)
                .map(|p| matches!(self.tok(p), ";" | "}" | "{"))
                .unwrap_or(true)
            && !self.out.ends_with("\n\n")
        {
            self.out.push('\n');
        }

        match t.kind {
            TokKind::Ident | TokKind::Number | TokKind::Str => self.emit_word(i),
            TokKind::Punct => self.emit_punct(i),
            _ => unreachable!(),
        }
        self.prev = Some(i);
    }

    fn emit_word(&mut self, i: usize) {
        let text = self.ast.tokens[i].text.clone();
        if matches!(text.as_ref(), "case" | "default") {
            let direct = self
                .switches
                .last()
                .is_some_and(|st| self.brace_stack.last() == Some(&st.open));
            if direct {
                if let Some(st) = self.switches.last_mut() {
                    if st.case_body {
                        self.indent = self.indent.saturating_sub(1);
                        st.case_body = false;
                    }
                }
                self.newline();
                self.write(&text);
                if let Some(st) = self.switches.last_mut() {
                    st.header = Some(CaseHeader {
                        paren: self.paren_depth,
                        bracket: self.bracket_depth,
                        brace: self.brace_depth,
                    });
                }
                self.prev = Some(i);
                return;
            }
        }
        if self.needs_space_before(i) {
            self.space();
        }
        self.write(&text);
    }

    fn needs_space_before(&self, i: usize) -> bool {
        let Some(p) = self.prev else {
            return false;
        };
        if self.at_line_start || self.out.ends_with('\n') || self.out.ends_with(' ') {
            return false;
        }
        let prev = self.tok(p);
        if prev == "{" && self.ast.brace_kind(p) == Some(BraceKind::Collection) {
            return false;
        }
        if matches!(prev, "(" | "[" | "." | "?." | ".." | "?.." | "@" | "#" | "...") {
            return false;
        }
        if self.generic_open == Some(p) {
            return false;
        }
        if self.generic_depth > 0 && matches!(prev, "<" | ">" | ">>" | ">>>") {
            return false;
        }
        if prev == "!"
            && self
                .prev_sig(p)
                .is_some_and(|q| matches!(self.tok(q), "is" | "as"))
        {
            // `is!`：否定符贴着关键字，类型前仍需空格。
            return true;
        }
        if matches!(prev, "!" | "~") {
            return false;
        }
        // 对象模式简写 `Box(:final value)`：`(`/`,` 后的 `:` 与字段名
        // 紧贴（冒号本身已无尾随空格）。
        if prev == ":"
            && self
                .prev_sig(p)
                .is_some_and(|q| matches!(self.tok(q), "(" | ","))
        {
            return false;
        }
        if matches!(prev, "+" | "-") && self.is_unary_at(p) {
            return false;
        }
        if matches!(prev, "++" | "--") && self.is_prefix_at(p) {
            return false;
        }
        let cur = &self.ast.tokens[i];
        // 默认：两个 word 之间、或 word 紧跟闭括号/后置运算符时需要空格。
        self.ast.tokens[p].kind.is_word()
            || matches!(prev, ")" | "]" | "}" | ">" | ">>" | ">>>" | "!" | "++" | "--")
            || cur.kind.is_word()
            || matches!(cur.text.as_ref(), "(" | "[" | "{")
    }

    fn is_unary_at(&self, i: usize) -> bool {
        let prev = self.prev_sig(i).map(|q| self.tok(q));
        match prev {
            None => true,
            Some(p) => matches!(
                p,
                "(" | "["
                    | "{"
                    | ","
                    | ";"
                    | ":"
                    | "="
                    | "=>"
                    | "?"
                    | "??"
                    | "??="
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "~/="
                    | "&="
                    | "|="
                    | "^="
                    | "<<="
                    | ">>="
                    | ">>>="
                    | ".."
                    | "?.."
                    | "..."
                    | "...?"
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "%"
                    | "~/"
                    | "&"
                    | "|"
                    | "^"
                    | "!"
                    | "~"
                    | "&&"
                    | "||"
                    | "=="
                    | "!="
                    | "<"
                    | ">"
                    | "<="
                    | ">="
                    | "<<"
                    | ">>"
                    | ">>>"
                    | "return"
                    | "throw"
                    | "yield"
                    | "await"
                    | "case"
                    | "when"
                    | "in"
                    | "is"
                    | "as"
            ),
        }
    }

    fn is_prefix_at(&self, i: usize) -> bool {
        let prev = self.prev_sig(i).map(|q| self.tok(q));
        match prev {
            None => true,
            Some(p) => matches!(
                p,
                "(" | "["
                    | "{"
                    | ","
                    | ";"
                    | ":"
                    | "="
                    | "=>"
                    | "?"
                    | "??"
                    | "??="
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "~/="
                    | "&="
                    | "|="
                    | "^="
                    | "<<="
                    | ">>="
                    | ">>>="
                    | ".."
                    | "?.."
                    | "..."
                    | "...?"
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "%"
                    | "~/"
                    | "&"
                    | "|"
                    | "^"
                    | "!"
                    | "~"
                    | "&&"
                    | "||"
                    | "return"
                    | "throw"
                    | "yield"
                    | "await"
                    | "case"
                    | "when"
            ),
        }
    }

    fn annotation_end(&self, i: usize) -> bool {
        let t = self.tok(i);
        if self.kind(i) == TokKind::Ident {
            return self.prev_sig(i).is_some_and(|p| self.tok(p) == "@");
        }
        if t == ")" {
            let Some(open) = self.ast.matching(i) else {
                return false;
            };
            let Some(head) = self.prev_sig(open) else {
                return false;
            };
            if self.kind(head) != TokKind::Ident {
                return false;
            }
            return self.prev_sig(head).is_some_and(|p| self.tok(p) == "@");
        }
        false
    }

    fn emit_punct(&mut self, i: usize) {
        let text = self.ast.tokens[i].text.clone();
        if text.as_ref() != "(" {
            self.tight_next = false;
        }
        match text.as_ref() {
            "{" => self.open_brace(i),
            "}" => self.close_brace(i),
            "(" => {
                if !self.tight_next && self.space_before_open_paren(i) {
                    self.space();
                }
                let is_for = self
                    .prev_sig(i)
                    .is_some_and(|p| self.tok(p) == "for");
                self.paren_kind
                    .push((is_for, self.brace_depth, self.bracket_depth));
                self.tight_next = false;
                self.write("(");
                self.paren_depth += 1;
            }
            ")" => {
                self.write(")");
                self.paren_kind.pop();
                self.paren_depth = self.paren_depth.saturating_sub(1);
                if self.annotation_end(i)
                    && self
                        .next_sig(i)
                        .is_some_and(|n| self.ast.tokens[n].newlines_before > 0)
                {
                    self.newline();
                }
            }
            "[" => {
                let tight = self.tight_index == Some(i);
                if tight {
                    self.tight_index = None;
                }
                if !tight && self.space_before_open_bracket(i) {
                    self.space();
                }
                self.write("[");
                self.bracket_depth += 1;
            }
            "]" => {
                self.write("]");
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
            }
            ";" => {
                self.write(";");
                let trailing_comment = self
                    .ast
                    .tokens
                    .get(i + 1)
                    .is_some_and(|t| t.is_comment() && t.newlines_before == 0);
                let in_for = self.paren_kind.last().is_some_and(|&(is_for, b, br)| {
                    is_for && b == self.brace_depth && br == self.bracket_depth
                });
                if in_for || trailing_comment {
                    self.space();
                } else {
                    self.newline();
                }
            }
            "," => {
                self.write(",");
                // switch 表达式的分支分隔：与 switch 块同层、且不在
                // case 头之后（表达式没有 case 关键字）→ 每个分支单独
                // 成行（dart format 同款；曾把全部分支挤成一行）。
                let arm_break = self.switches.last().is_some_and(|st| {
                    self.brace_stack.last() == Some(&st.open)
                        && st.header.is_none()
                        && !st.case_body
                        && self.paren_depth == st.paren
                        && self.bracket_depth == st.bracket
                });
                if arm_break {
                    self.newline();
                } else if self
                    .next_sig(i)
                    .is_some_and(|n| !matches!(self.tok(n), ")" | "]" | "}" | ">" | ">>" | ">>>"))
                {
                    self.space();
                }
            }
            ":" => self.colon(i),
            "." | "?." | ".." | "?.." => self.write(&text),
            "?" => self.question(i),
            "!" => {
                // `is!` 紧贴关键字；后缀空断言（`x!`）也紧贴；
                // 其余前缀位置按普通词间距处理（如 `return !x`）。
                let tight = self.prev_text().is_some_and(|p| matches!(p, "is" | "as"))
                    || !self.is_unary_at(i);
                if !tight && self.needs_space_before(i) {
                    self.space();
                }
                self.write("!");
            }
            "~" => {
                if self.is_unary_at(i) && self.needs_space_before(i) {
                    self.space();
                }
                self.write("~");
            }
            "++" | "--" => self.write(&text),
            "+" | "-" => self.plus_minus(i),
            "<" => self.less(i),
            ">" | ">>" | ">>>" => self.greater(i),
            "..." | "...?" => {
                if self
                    .prev_text()
                    .is_some_and(|p| matches!(p, "(" | "[" | "{" | "," | ":"))
                {
                    self.write(&text);
                } else {
                    self.space();
                    self.write(&text);
                }
            }
            "=" if self.is_operator_bracket_assign(i) => {
                self.write("=");
                self.tight_next = true;
            }
            "*" if self
                .prev_text()
                .is_some_and(|p| matches!(p, "async" | "sync" | "yield")) =>
            {
                // `async*`/`sync*`/`yield*` 是紧贴关键字形态。
                self.write("*");
            }
            "=>" | "=" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "??" | "??=" | "+=" | "-="
            | "*=" | "/=" | "%=" | "~/=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>="
            | "&" | "|" | "^" | "*" | "/" | "%" | "~/" | "<<" => {
                self.space();
                self.write(&text);
                self.space();
            }
            "@" => {
                if !self.at_line_start
                    && !self
                        .prev_text()
                        .is_some_and(|p| matches!(p, "(" | "[" | "{" | "," | ":" | "@"))
                {
                    self.space();
                }
                self.write("@");
            }
            "#" => self.write("#"),
            _ => self.write(&text),
        }
    }

    fn space_before_open_paren(&self, i: usize) -> bool {
        let Some(p) = self.prev_sig(i) else {
            return false;
        };
        matches!(
            self.tok(p),
            "if" | "for"
                | "while"
                | "switch"
                | "catch"
                | "=>"
                | "="
                | ","
                | ":"
                | "?"
                | "return"
                | "throw"
                | "await"
                | "yield"
                | "in"
                | "is"
                | "as"
                // 模式/声明关键字后的记录类型或记录模式 `final (i, v)`：
                // 不能粘成 `final(`（函数调用形态会被误读）。
                | "final"
                | "const"
                | "var"
                | "late"
                | "required"
                | "case"
                | "when"
                | "assert"
                | "else"
                | "do"
                | "on"
        )
    }

    fn space_before_open_bracket(&self, i: usize) -> bool {
        let Some(p) = self.prev_sig(i) else {
            return false;
        };
        // 关键字后的列表字面量/列表模式（`return [1]`、`case [_, ...]`、
        // `yield [x]`）需要空格；普通标识符后是索引访问 `a[i]`，紧凑。
        let kw = self.tok(p);
        if matches!(
            kw,
            "return" | "throw" | "yield" | "in" | "is" | "as" | "assert" | "case" | "when"
                | "await" | "else" | "do" | "on" | "final" | "const" | "var" | "late"
        ) {
            return true;
        }
        if self.ast.tokens[p].kind.is_word()
            || matches!(kw, ")" | "]" | ">" | ">>" | ">>>" | "!" | "++" | "--")
        {
            return false;
        }
        !matches!(kw, "(" | "[" | "{" | "." | "?." | "@" | "#" | "...")
    }

    fn is_operator_bracket_assign(&self, i: usize) -> bool {
        if self.tok(i) != "=" {
            return false;
        }
        let Some(close) = self.prev_sig(i) else {
            return false;
        };
        if self.tok(close) != "]" {
            return false;
        }
        let Some(open) = self.ast.matching(close) else {
            return false;
        };
        self.prev_sig(open).is_some_and(|p| self.tok(p) == "operator")
    }

    fn open_brace(&mut self, i: usize) {
        let kind = self.ast.brace_kind(i).unwrap_or(BraceKind::Block);
        if kind == BraceKind::Collection {
            if self.prev_text().is_some_and(|p| {
                !matches!(
                    p,
                    "(" | "[" | "{" | "." | "?." | "@" | "#" | "..." | ">" | ">>" | ">>>"
                )
            }) {
                self.space();
            }
            self.write("{");
            self.brace_depth += 1;
            self.brace_stack.push(i);
            return;
        }
        self.space();
        self.write("{");
        self.newline();
        self.indent += 1;
        self.brace_depth += 1;
        self.brace_stack.push(i);
        if kind == BraceKind::SwitchBlock {
            self.switches.push(SwitchState {
                open: i,
                case_body: false,
                header: None,
                paren: self.paren_depth,
                bracket: self.bracket_depth,
            });
        }
    }

    fn close_brace(&mut self, i: usize) {
        let Some(open) = self.ast.matching(i) else {
            self.write("}");
            return;
        };
        let kind = self.ast
            .brace_kind(open)
            .unwrap_or(BraceKind::Collection);
        self.brace_stack.pop();
        self.brace_depth = self.brace_depth.saturating_sub(1);
        if kind == BraceKind::Collection {
            self.write("}");
            return;
        }
        if kind == BraceKind::SwitchBlock {
            if let Some(st) = self.switches.pop() {
                if st.case_body {
                    self.indent = self.indent.saturating_sub(1);
                }
            }
        }
        self.newline();
        self.indent = self.indent.saturating_sub(1);
        self.write("}");
        // 顶层声明体收尾后必定开始新声明：`class A {}` 后面紧跟记录返回
        // 类型 `(int, String) f()` 时不能粘成 `}(int, ...)`。表达式语境
        // 的 `}(` 只有函数字面量立即调用，不会出现在顶层。
        if self.brace_depth == 0 && self.paren_depth == 0 && self.bracket_depth == 0 {
            // 例外：语句收尾 `};` / `},` 保持紧凑（switch 表达式后跟 `;`）。
            if self
                .next_sig(i)
                .is_some_and(|n| !matches!(self.tok(n), ";" | ","))
            {
                self.newline();
                return;
            }
        }
        if let Some(n) = self.next_sig(i) {
            let nt = self.tok(n);
            if matches!(nt, "else" | "catch" | "finally" | "while") {
                self.space();
            } else if !matches!(
                nt,
                ";" | "," | ")" | "]" | "." | "?." | "!" | "?" | ":" | "(" | "[" | ".."
            ) {
                self.newline();
            }
        } else {
            self.newline();
        }
    }

    fn colon(&mut self, _i: usize) {
        let case_here = self.switches.last().is_some_and(|st| {
            st.header.is_some_and(|h| {
                self.paren_depth == h.paren
                    && self.bracket_depth == h.bracket
                    && self.brace_depth == h.brace
            })
        });
        if case_here {
            self.write(":");
            self.newline();
            self.indent += 1;
            self.ternary.clear();
            if let Some(st) = self.switches.last_mut() {
                st.case_body = true;
                st.header = None;
            }
            return;
        }
        if self
            .ternary
            .last()
            .is_some_and(|&(p, b, br)| {
                self.paren_depth == p && self.bracket_depth == b && self.brace_depth == br
            })
        {
            self.ternary.pop();
            self.space();
            self.write(":");
            self.space();
            return;
        }
        self.write(":");
        // 对象模式简写 `Box(:final value)`：冒号紧跟 `(`/`,`，与字段名
        // 紧贴（`(: final value)` 合法但不是 dart format 形态）。
        let shorthand = self
            .prev_sig(_i)
            .is_some_and(|p| matches!(self.tok(p), "(" | ","));
        if !shorthand {
            self.space();
        }
    }

    fn question(&mut self, i: usize) {
        // `x?[i]` 的空索引：`?` 与 `[` 的相邻性在 Dart 里影响解析
        // （`? [` 会被当成条件表达式），必须逐字节保持紧凑。
        if let Some(n) = self.ast.next_sig(i) {
            if n == i + 1
                && self.tok(n) == "["
                && self.ast.tokens[n].line == self.ast.tokens[i].line
                && self.ast.tokens[n].col == self.ast.tokens[i].col + 1
            {
                self.write("?");
                self.tight_index = Some(n);
                return;
            }
        }
        let ternary = self.is_ternary_question(i)
            && !self.next_sig(i).is_some_and(|n| self.tok(n) == ":");
        if ternary {
            self.space();
            self.write("?");
            self.space();
            self.ternary
                .push((self.paren_depth, self.bracket_depth, self.brace_depth));
            return;
        }
        // 可空类型后缀 / null-check pattern：`int? x`, `List<int?>`, `x?`。
        self.write("?");
        if let Some(n) = self.next_sig(i) {
            if self.ast.tokens[n].kind.is_word()
                || matches!(self.tok(n), ".." | "." | "?." | "?.." | "?")
            {
                self.space();
            }
        }
    }

    fn is_ternary_question(&self, i: usize) -> bool {
        let mut j = i + 1;
        let mut depth = 0i32;
        while j < self.ast.tokens.len() {
            let t = &self.ast.tokens[j];
            if t.is_comment() {
                j += 1;
                continue;
            }
            match t.text.as_ref() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                "?" if depth == 0 => {
                    // 嵌套三元：跳过其分支，交给外层匹配。
                    j += 1;
                    continue;
                }
                ":" if depth == 0 => return true,
                ";" if depth == 0 => return false,
                _ => {}
            }
            j += 1;
        }
        false
    }

    fn plus_minus(&mut self, i: usize) {
        let unary = self.is_unary_at(i);
        if unary {
            if self
                .prev_text()
                .is_some_and(|p| matches!(p, ")" | "]" | "}" | "!" | "++" | "--"))
            {
                self.space();
            } else if self.prev_text().is_some_and(|p| {
                !matches!(p, "(" | "[" | "{" | "," | ":" | "=" | "=>" | "?" | "??")
            }) {
                self.space();
            }
            self.write(&self.ast.tokens[i].text);
        } else {
            self.space();
            self.write(&self.ast.tokens[i].text);
            self.space();
        }
    }

    fn less(&mut self, i: usize) {
        if self.looks_like_generic(i) {
            self.write("<");
            self.generic_depth += 1;
            self.generic_open = Some(i);
        } else {
            self.space();
            self.write("<");
            self.space();
        }
    }

    fn greater(&mut self, i: usize) {
        let t = self.ast.tokens[i].text.clone();
        if self.generic_depth > 0 {
            self.write(&t);
            // 原文是两个独立 `>` token（常见于换行包裹的嵌套泛型）时，
            // 中间补空格，避免输出被重新词法化成单个 `>>`。
            if self
                .next_sig(i)
                .is_some_and(|n| self.tok(n).starts_with('>'))
            {
                self.space();
            }
            let n = match t.as_ref() {
                ">>" => 2,
                ">>>" => 3,
                _ => 1,
            };
            self.generic_depth = self.generic_depth.saturating_sub(n);
        } else {
            self.space();
            self.write(&t);
            self.space();
        }
    }

    fn looks_like_generic(&self, i: usize) -> bool {
        let Some(p) = self.prev_sig(i) else {
            return false;
        };
        let pt = self.tok(p);
        if !(self.ast.tokens[p].kind == TokKind::Ident
            || matches!(
                pt,
                ">" | ">>"
                    | ">>>"
                    | "]"
                    | ")"
                    | "="
                    | "("
                    | "["
                    | "{"
                    | ","
                    | ":"
                    | "return"
                    | "=>"
                    | "?"
                    | "??"
                    | "const"
                    | "new"
            ))
        {
            return false;
        }
        let mut depth = 0i32;
        let mut j = i;
        let mut close = None;
        while j < self.ast.tokens.len() {
            let t = &self.ast.tokens[j];
            if t.is_comment() {
                j += 1;
                continue;
            }
            match t.text.as_ref() {
                "<" => depth += 1,
                ">" => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(j);
                        break;
                    }
                }
                ">>" => {
                    depth -= 2;
                    if depth <= 0 {
                        close = Some(j);
                        break;
                    }
                }
                ">>>" => {
                    depth -= 3;
                    if depth <= 0 {
                        close = Some(j);
                        break;
                    }
                }
                _ if t.kind == TokKind::Ident
                    || t.kind == TokKind::Number
                    || matches!(
                        t.text.as_ref(),
                        "." | ","
                            | "?"
                            | "*"
                            | "("
                            | ")"
                            | "["
                            | "]"
                            | "{"
                            | "}"
                            | "@"
                            | ":"
                    ) =>
                {}
                _ => return false,
            }
            j += 1;
        }
        let Some(close) = close else {
            return false;
        };
        match self.next_sig(close) {
            None => true,
            Some(n) => self.ast.tokens[n].kind == TokKind::Ident
                || matches!(
                    self.tok(n),
                    "(" | "."
                        | "?."
                        | ","
                        | ")"
                        | "]"
                        | "}"
                        | ";"
                        | "="
                        | "?"
                        | ":"
                        | "{"
                        | "=>"
                        | "!"
                        | ".."
                        | "??"
                        | "["
                        | "<"
                        | ">"
                        | ">>"
                        | ">>>"
                ),
        }
    }
}