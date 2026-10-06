use cure_java_parser::parse;

#[test]
fn formfeed_unicode_escape_roundtrip() {
    // ImportOrderer.java 形态：" \t\u000C" —— JLS 3.3 预解码后字符串字面量
    // 内含原始 0x0C；词法器必须接受（JLS：formfeed 不是行终结符）
    let src = r#"class A{boolean f(String s){return s.isEmpty() ? false : " \t\u000C".indexOf(1) >= 0;}}"#;
    let o = parse(src);
    assert!(o.errors.is_empty(), "解析失败: {:?}", o.errors);
}

#[test]
fn formfeed_stage_isolation() {
    // 阶段定位：A) 纯 \t 转义 B) 原始 0x0C 字节 C) \u000C 逃逸
    let a = r#"class A{String s = " \t";}"#;
    let b = "class A{String s = \" \u{0C}\";}";
    let c = r#"class A{String s = " \u000C";}"#;
    for (name, src) in [("A-backslash-t", a), ("B-raw-formfeed", b), ("C-escape", c)] {
        let o = parse(src);
        println!("{name}: errors={}", o.errors.len());
        assert!(o.errors.is_empty(), "{name}: {:?}", o.errors);
    }
}
