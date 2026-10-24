//! 调试：比较两个文件的 AST 结构等价（忽略 Paren/字面量拼写/Group 展开）。
//! 用法：`equiv_check <a.js> <b.js>`；`CURE_EQUIV_DEBUG=1` 打印首个差异节点。

use std::path::PathBuf;

use cure_ts_parser::parse;

fn main() {
    let mut args = std::env::args().skip(1);
    let a = PathBuf::from(args.next().expect("usage: equiv_check <a.js> <b.js>"));
    let b = PathBuf::from(args.next().expect("usage: equiv_check <a.js> <b.js>"));
    let sa = std::fs::read_to_string(a).expect("read a");
    let sb = std::fs::read_to_string(b).expect("read b");
    let pa = parse(&sa);
    let pb = parse(&sb);
    eprintln!("parse errors: a={:?} b={:?}", pa.errors, pb.errors);
    println!(
        "equiv = {}",
        pa.ast.unit_equiv(&pa.unit, &pb.ast, &pb.unit)
    );
}