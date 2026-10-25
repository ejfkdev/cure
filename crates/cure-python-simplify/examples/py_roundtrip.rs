//! 解析 → 打印（不简化）：用于格式往返/AST 等价测试。
use cure_python_parser::parse;
use cure_python_print::print_module;

fn main() {
    let path = std::env::args().nth(1).expect("usage: py_roundtrip FILE");
    let src = std::fs::read_to_string(&path).unwrap();
    let out = parse(&src);
    for e in &out.errors {
        eprintln!("{}:{}:{}: {}", path, e.line, e.col, e.message);
    }
    print!("{}", print_module(&out.ast, out.module));
}
