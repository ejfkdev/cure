//! 调试/测试用：parse → simplify → print；诊断输出到 stderr。
use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

fn main() {
    let path = std::env::args().nth(1).expect("usage: py_pipeline FILE");
    let src = std::fs::read_to_string(&path).unwrap();
    let mut out = parse(&src);
    for e in &out.errors {
        eprintln!("{}:{}:{}: {}", path, e.line, e.col, e.message);
    }
    let report = simplify_module(&mut out.ast, out.module, &Config::default());
    eprintln!(
        "py: {} edits / {} rounds / {:?}",
        report.edits, report.iterations, report.by_rule
    );
    print!("{}", print_module(&out.ast, out.module));
}
