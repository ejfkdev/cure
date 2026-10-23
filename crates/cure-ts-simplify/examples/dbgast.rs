use cure_ts_ast::{NodeData, TopItem};
fn dump(ast: &cure_ts_ast::TsAst, n: cure_ts_ast::TsId, d: usize) {
    let pad = "  ".repeat(d);
    match ast.data(n) {
        NodeData::Literal(_) => println!("{pad}Lit"),
        NodeData::VarRef { name } => println!("{pad}Var {}", ast.sn(*name)),
        NodeData::Raw { text } => println!("{pad}Raw {:?}", &text[..text.len().min(40)]),
        other => println!("{pad}{other:?}"),
    }
    for &c in ast.children(n) { dump(ast, c, d + 1); }
}
fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let out = cure_ts_parser::parse(&src);
    println!("ERR {:?}", out.errors);
    for it in &out.unit.items {
        match it {
            TopItem::Stmt(s) | TopItem::ExportStmt(s) => dump(&out.ast, *s, 0),
            other => println!("ITEM {other:?}"),
        }
    }
}
