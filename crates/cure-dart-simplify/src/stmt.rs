//! Token-level statement boundaries and branch extraction for Dart rewrites.

use cure_dart_ast::{BraceKind, DartAst, TokKind};

#[derive(Clone, Copy, Debug)]
pub struct Stmt {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ReturnInfo {
    pub expr: (usize, usize),
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct AssignInfo {
    pub target: (usize, usize),
    pub rhs: (usize, usize),
    pub end: usize,
}

fn next(ast: &DartAst, i: usize, limit: usize) -> Option<usize> {
    let n = ast.next_sig(i)?;
    (n < limit).then_some(n)
}

fn matching(ast: &DartAst, i: usize, limit: usize) -> Option<usize> {
    let m = ast.matching(i)?;
    (m < limit).then_some(m)
}

/// End of a statement starting at `start` (exclusive), bounded by `limit`.
pub fn stmt_end(ast: &DartAst, start: usize, limit: usize) -> Option<usize> {
    if start >= limit {
        return None;
    }
    // Labels: `label: <statement>`.
    if ast.tokens[start].kind == TokKind::Ident
        && ast
            .next_sig(start)
            .is_some_and(|n| n < limit && ast.text(n) == ":")
    {
        let colon = ast.next_sig(start)?;
        let body = next(ast, colon, limit)?;
        return stmt_end(ast, body, limit);
    }
    match ast.text(start) {
        "{" => matching(ast, start, limit).map(|c| c + 1),
        "if" => {
            let open = next(ast, start, limit)?;
            if ast.text(open) != "(" {
                return None;
            }
            let close = matching(ast, open, limit)?;
            let then_start = next(ast, close, limit)?;
            let then_end = stmt_end(ast, then_start, limit)?;
            if let Some(e) = next(ast, then_end - 1, limit) {
                if ast.text(e) == "else" {
                    let else_start = next(ast, e, limit)?;
                    return stmt_end(ast, else_start, limit);
                }
            }
            Some(then_end)
        }
        "for" | "while" => {
            let open = next(ast, start, limit)?;
            if ast.text(open) != "(" {
                return None;
            }
            let close = matching(ast, open, limit)?;
            let body = next(ast, close, limit)?;
            stmt_end(ast, body, limit)
        }
        "do" => {
            let body = next(ast, start, limit)?;
            let body_end = stmt_end(ast, body, limit)?;
            let w = next(ast, body_end - 1, limit)?;
            if ast.text(w) != "while" {
                return None;
            }
            let open = next(ast, w, limit)?;
            if ast.text(open) != "(" {
                return None;
            }
            let close = matching(ast, open, limit)?;
            let semi = next(ast, close, limit)?;
            (ast.text(semi) == ";").then_some(semi + 1)
        }
        "switch" => {
            let open = next(ast, start, limit)?;
            if ast.text(open) != "(" {
                return None;
            }
            let close = matching(ast, open, limit)?;
            let block = next(ast, close, limit)?;
            if ast.text(block) != "{" || ast.brace_kind(block) != Some(BraceKind::SwitchBlock) {
                return None;
            }
            matching(ast, block, limit).map(|c| c + 1)
        }
        "try" => stmt_end_try(ast, start, limit),
        _ => scan_simple(ast, start, limit),
    }
}

fn stmt_end_try(ast: &DartAst, start: usize, limit: usize) -> Option<usize> {
    let block = next(ast, start, limit)?;
    if ast.text(block) != "{" {
        return None;
    }
    let mut i = matching(ast, block, limit)? + 1;
    loop {
        let Some(n) = next(ast, i.saturating_sub(1), limit) else {
            return Some(i);
        };
        match ast.text(n) {
            "catch" => {
                let mut j = next(ast, n, limit)?;
                if ast.text(j) == "(" {
                    j = matching(ast, j, limit)? + 1;
                }
                let b = next(ast, j, limit)?;
                if ast.text(b) != "{" {
                    return None;
                }
                i = matching(ast, b, limit)? + 1;
            }
            "on" => {
                let mut j = next(ast, n, limit)?;
                loop {
                    if j >= limit {
                        return None;
                    }
                    if ast.text(j) == "{" {
                        i = matching(ast, j, limit)? + 1;
                        break;
                    }
                    if ast.text(j) == ";" {
                        return None;
                    }
                    j = next(ast, j, limit)?;
                }
            }
            "finally" => {
                let b = next(ast, n, limit)?;
                if ast.text(b) != "{" {
                    return None;
                }
                i = matching(ast, b, limit)? + 1;
            }
            _ => return Some(i),
        }
    }
}

fn scan_simple(ast: &DartAst, start: usize, limit: usize) -> Option<usize> {
    let mut i = start;
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut saw_eq = false;
    let expression_block = matches!(
        ast.text(start),
        "return" | "throw" | "var" | "final" | "const" | "late" | "await" | "yield"
    );
    while i < limit {
        let t = ast.text(i);
        match t {
            "(" => paren += 1,
            ")" => paren = paren.saturating_sub(1),
            "[" => bracket += 1,
            "]" => bracket = bracket.saturating_sub(1),
            "=" if paren == 0 && bracket == 0 => saw_eq = true,
            "{" if paren == 0 && bracket == 0 => {
                let close = matching(ast, i, limit)?;
                if ast.brace_kind(i) == Some(BraceKind::Collection) || saw_eq || expression_block {
                    i = close + 1;
                    continue;
                }
                return Some(close + 1);
            }
            ";" if paren == 0 && bracket == 0 => return Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Split a block into direct statements. `None` when a statement cannot be
/// bounded conservatively (the caller skips the block).
pub fn split_block(ast: &DartAst, open: usize, close: usize) -> Option<Vec<Stmt>> {
    let mut out = Vec::new();
    let mut i = open + 1;
    while i < close {
        while i < close && ast.tokens[i].is_comment() {
            i += 1;
        }
        if i >= close {
            break;
        }
        let end = stmt_end(ast, i, close)?;
        if end <= i || end > close {
            return None;
        }
        out.push(Stmt { start: i, end });
        i = end;
    }
    Some(out)
}

pub fn is_labeled(ast: &DartAst, stmt: Stmt) -> bool {
    ast.tokens[stmt.start].kind == TokKind::Ident
        && ast
            .next_sig(stmt.start)
            .is_some_and(|n| n < stmt.end && ast.text(n) == ":")
}

pub fn is_terminal(ast: &DartAst, stmt: Stmt) -> bool {
    ast.tokens[stmt.start].kind == TokKind::Ident
        && matches!(
            ast.text(stmt.start),
            "return" | "break" | "continue" | "throw"
        )
}

fn return_stmt(ast: &DartAst, start: usize, limit: usize) -> Option<ReturnInfo> {
    if ast.text(start) != "return" {
        return None;
    }
    let end = stmt_end(ast, start, limit)?;
    if ast.text(end - 1) != ";" || ast.has_comment_between(start, end - 1) {
        return None;
    }
    let expr_start = next(ast, start, end)?;
    if expr_start >= end - 1 {
        return None;
    }
    Some(ReturnInfo {
        expr: (expr_start, end - 1),
        end,
    })
}

pub fn branch_return(ast: &DartAst, start: usize, limit: usize) -> Option<ReturnInfo> {
    if ast.text(start) == "return" {
        return return_stmt(ast, start, limit);
    }
    if ast.text(start) == "{" && ast.brace_kind(start) == Some(BraceKind::Block) {
        let close = matching(ast, start, limit)?;
        if ast.has_comment_between(start, close) {
            return None;
        }
        let stmts = split_block(ast, start, close)?;
        if stmts.len() == 1 {
            let mut info = return_stmt(ast, stmts[0].start, close)?;
            // 语句本身只到 `;`；分支是块时要消费整个块的 `}`。
            info.end = close + 1;
            return Some(info);
        }
    }
    None
}

fn assign_stmt(ast: &DartAst, start: usize, limit: usize) -> Option<AssignInfo> {
    let end = stmt_end(ast, start, limit)?;
    if ast.text(end - 1) != ";" || ast.has_comment_between(start, end - 1) {
        return None;
    }
    let semi = end - 1;
    let mut eq = None;
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut i = start;
    while i < semi {
        match ast.text(i) {
            "(" => paren += 1,
            ")" => paren = paren.saturating_sub(1),
            "[" => bracket += 1,
            "]" => bracket = bracket.saturating_sub(1),
            "=" if paren == 0 && bracket == 0 => {
                eq = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let eq = eq?;
    let target_start = start;
    let target_end = eq;
    if target_end != target_start + 1 || ast.tokens[target_start].kind != TokKind::Ident {
        return None;
    }
    if matches!(
        ast.text(target_start),
        "var" | "final" | "const" | "late" | "this" | "super"
    ) {
        return None;
    }
    let rhs_start = next(ast, eq, semi)?;
    if rhs_start >= semi {
        return None;
    }
    Some(AssignInfo {
        target: (target_start, target_end),
        rhs: (rhs_start, semi),
        end,
    })
}

pub fn branch_assign(ast: &DartAst, start: usize, limit: usize) -> Option<AssignInfo> {
    if ast.tokens[start].kind == TokKind::Ident {
        return assign_stmt(ast, start, limit);
    }
    if ast.text(start) == "{" && ast.brace_kind(start) == Some(BraceKind::Block) {
        let close = matching(ast, start, limit)?;
        if ast.has_comment_between(start, close) {
            return None;
        }
        let stmts = split_block(ast, start, close)?;
        if stmts.len() == 1 {
            let mut info = assign_stmt(ast, stmts[0].start, close)?;
            info.end = close + 1;
            return Some(info);
        }
    }
    None
}

pub fn token_text(ast: &DartAst, range: (usize, usize)) -> String {
    ast.tokens[range.0..range.1]
        .iter()
        .map(|t| t.text.as_ref())
        .collect()
}

/// True when `i` is in statement position (inside a block), not inside a
/// collection literal or argument list where `if` may be an element.
pub fn statement_context(ast: &DartAst, i: usize) -> bool {
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut j = i;
    while j > 0 {
        j -= 1;
        let t = ast.text(j);
        match t {
            ")" => paren += 1,
            "(" => {
                if paren == 0 {
                    return false;
                }
                paren -= 1;
            }
            "]" => bracket += 1,
            "[" => {
                if bracket == 0 {
                    return false;
                }
                bracket -= 1;
            }
            "}" => brace += 1,
            "{" => {
                if brace == 0 {
                    return matches!(
                        ast.brace_kind(j),
                        Some(BraceKind::Block) | Some(BraceKind::SwitchBlock)
                    );
                }
                brace -= 1;
            }
            _ => {}
        }
    }
    true
}