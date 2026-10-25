fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&path).unwrap();
    let (t, errs) = cure_python_parser::lexer::tokenize(&src);
    eprintln!("lex errors: {errs:?}");
    for (i, tok) in t.iter().enumerate() {
        eprintln!(
            "{i}: {:?} {:?} {}:{}",
            tok.kind, tok.text, tok.line, tok.col
        );
    }
}
