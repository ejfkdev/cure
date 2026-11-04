//! 0.4.3 regression: a long constant xor chain folds to an oversized
//! decimal literal. The printer must keep the value representable.

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

#[test]
fn long_xor_fold_keeps_long_suffix() {
    let src = r#"
class LongLiteral {
    static String f(String account2) {
        return String.valueOf(Long.valueOf(account2).longValue()
            ^ 662347895234L ^ 456752456285L ^ 789454421545L);
    }
}
"#;
    let mut outcome = parse(src);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    let _ = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);
    assert!(
        cleaned.contains("307638947254L"),
        "folded long literal lost its L suffix:\n{cleaned}"
    );
}
