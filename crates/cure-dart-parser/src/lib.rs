//! 容错 Dart 解析入口：词法 + 括号结构。从不拒绝文件。
//!
//! Dart 的版本差异全部在 token/结构层吸收：解析器只报告未闭合字符串/
//! 注释与括号不配对，其余（含未建模的新语法）原样留在 token 流里。

pub mod lexer;

use cure_dart_ast::DartAst;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

pub struct ParseOutcome {
    pub ast: DartAst,
    pub errors: Vec<ParseError>,
}

pub fn parse(src: &str) -> ParseOutcome {
    let (tokens, lex_errors) = lexer::tokenize(src);
    let (ast, bracket_errors) = DartAst::from_tokens(tokens);
    let mut errors: Vec<ParseError> = lex_errors
        .into_iter()
        .map(|e| ParseError {
            line: e.line,
            col: e.col,
            message: e.message,
        })
        .collect();
    for e in bracket_errors {
        let (line, col) = ast
            .tokens
            .get(e.index)
            .map(|t| (t.line, t.col))
            .unwrap_or((1, 1));
        errors.push(ParseError {
            line,
            col,
            message: e.message,
        });
    }
    ParseOutcome { ast, errors }
}