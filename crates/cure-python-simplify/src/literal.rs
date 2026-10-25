//! 字面量辅助：只处理「无前缀、无转义、内容为可打印 ASCII」的字符串，
//! 这样 Python 2/3 的 str 语义一致，折叠结果也能用普通引号稳定表示。

use cure_python_ast::{LitKind, Literal};

const MAX_STR: usize = 100_000;

/// 提取安全可折叠的普通字符串内容。
pub fn plain_str(lit: &Literal) -> Option<&str> {
    if lit.kind != LitKind::Str {
        return None;
    }
    let raw = lit.raw.trim();
    if raw.len() < 2 {
        return None;
    }
    let b = raw.as_bytes();
    let q = b[0];
    if q != b'\'' && q != b'"' {
        return None;
    }
    if b[b.len() - 1] != q {
        return None;
    }
    // 三引号/前缀/多段拼接都不在安全集合内（按字节判断，避免切在
    // 非 ASCII 字符中间）。
    if b.len() >= 6 && b[0] == q && b[1] == q && b[2] == q
        && b[b.len() - 1] == q && b[b.len() - 2] == q && b[b.len() - 3] == q
    {
        return None;
    }
    let inner = &raw[1..raw.len() - 1];
    let qc = q as char;
    if inner
        .chars()
        .any(|c| !c.is_ascii() || c == '\\' || c == qc || c == '\'' || c == '"')
    {
        return None;
    }
    Some(inner)
}

pub fn is_plain_ascii(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii() && !c.is_control() && c != '\'' && c != '"' && c != '\\')
}

/// 生成普通字符串字面量；内容必须是安全 ASCII（见上）。
pub fn make_str(s: &str) -> Option<Literal> {
    if !is_plain_ascii(s) || s.len() > MAX_STR {
        return None;
    }
    Some(Literal::raw_str(format!("'{s}'"), LitKind::Str))
}

pub fn repeat(s: &str, n: i64) -> Option<String> {
    if n <= 0 {
        return Some(String::new());
    }
    let n = n as usize;
    if s.len().checked_mul(n)? > MAX_STR {
        return None;
    }
    Some(s.repeat(n))
}

/// Python `str.strip/lstrip/rstrip`；`chars` 为 None 时用 Python 的空白集合
/// （输入是 ASCII，ASCII 空白集合一致）。
pub fn strip(s: &str, chars: Option<&str>, left: bool, right: bool) -> String {
    let default = " \t\n\r\x0b\x0c";
    let set = chars.unwrap_or(default);
    let matches = |c: char| set.contains(c);
    let mut a = 0usize;
    let mut b = s.len();
    if left {
        while a < b {
            let c = s[a..].chars().next().unwrap();
            if matches(c) {
                a += c.len_utf8();
            } else {
                break;
            }
        }
    }
    if right {
        while b > a {
            let c = s[..b].chars().next_back().unwrap();
            if matches(c) {
                b -= c.len_utf8();
            } else {
                break;
            }
        }
    }
    s[a..b].to_string()
}

pub fn replace(s: &str, old: &str, new: &str, count: Option<i64>) -> Option<String> {
    if old.is_empty() {
        // Python 对空串的插入语义与 Rust 不同，保守跳过。
        return None;
    }
    let out = match count {
        Some(n) if n < 0 => s.replace(old, new),
        Some(n) => s.replacen(old, new, n as usize),
        None => s.replace(old, new),
    };
    if out.len() > MAX_STR {
        None
    } else {
        Some(out)
    }
}

pub fn char_at(s: &str, idx: i64) -> Option<String> {
    let len = s.len() as i64;
    let i = if idx < 0 { idx + len } else { idx };
    if i < 0 || i >= len {
        return None;
    }
    let c = s[i as usize..].chars().next()?;
    Some(c.to_string())
}

/// Python 切片下标（ASCII 时字节即字符）。
pub fn slice_indices(
    len: usize,
    lower: Option<i64>,
    upper: Option<i64>,
    step: Option<i64>,
) -> Option<Vec<usize>> {
    let len = len as i64;
    let step = step.unwrap_or(1);
    if step == 0 {
        return None;
    }
    let mut out = Vec::new();
    if step > 0 {
        let mut start = lower.unwrap_or(0);
        let mut stop = upper.unwrap_or(len);
        if start < 0 {
            start += len;
        }
        if stop < 0 {
            stop += len;
        }
        start = start.clamp(0, len);
        stop = stop.clamp(0, len);
        let mut i = start;
        while i < stop {
            out.push(i as usize);
            i += step;
        }
    } else {
        let mut start = lower.unwrap_or(len - 1);
        let stop = match upper {
            None => -1,
            Some(v) if v < 0 => v + len,
            Some(v) => v,
        };
        if start < 0 {
            start += len;
        }
        let start = start.clamp(-1, len - 1);
        let stop = stop.clamp(-1, len - 1);
        let mut i = start;
        while i > stop {
            if i >= 0 {
                out.push(i as usize);
            }
            i += step;
        }
    }
    Some(out)
}

pub fn slice(s: &str, lower: Option<i64>, upper: Option<i64>, step: Option<i64>) -> Option<String> {
    let idx = slice_indices(s.len(), lower, upper, step)?;
    if idx.len() > MAX_STR {
        return None;
    }
    let b = s.as_bytes();
    Some(idx.into_iter().map(|i| b[i] as char).collect())
}

pub fn norm_index(len: usize, idx: i64) -> Option<usize> {
    let len = len as i64;
    let i = if idx < 0 { idx + len } else { idx };
    if i < 0 || i >= len {
        None
    } else {
        Some(i as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(raw: &str) -> Literal {
        Literal::raw_str(raw, LitKind::Str)
    }

    #[test]
    fn plain_string_extraction() {
        assert_eq!(plain_str(&l("'abc'")), Some("abc"));
        assert_eq!(plain_str(&l("\"a b\"")), Some("a b"));
        assert_eq!(plain_str(&l("''")), Some(""));
        assert_eq!(plain_str(&l("u'abc'")), None);
        assert_eq!(plain_str(&l("'a\\n'")), None);
        assert_eq!(plain_str(&l("'a\"b'")), None);
        assert_eq!(plain_str(&l("'♪'")), None);
        assert_eq!(plain_str(&l("'''abc'''")), None);
    }

    #[test]
    fn python_slices() {
        assert_eq!(slice("abcdef", None, None, None).as_deref(), Some("abcdef"));
        assert_eq!(slice("abcdef", Some(1), Some(4), None).as_deref(), Some("bcd"));
        assert_eq!(slice("abcdef", Some(-2), None, None).as_deref(), Some("ef"));
        assert_eq!(slice("abcdef", None, None, Some(-1)).as_deref(), Some("fedcba"));
        assert_eq!(slice("abcdef", Some(4), Some(1), Some(-1)).as_deref(), Some("edc"));
        assert_eq!(slice("abcdef", None, None, Some(2)).as_deref(), Some("ace"));
        assert_eq!(slice("abcdef", Some(1), Some(1), None).as_deref(), Some(""));
    }

    #[test]
    fn strip_and_replace() {
        assert_eq!(strip("  a b  ", None, true, true), "a b");
        assert_eq!(strip("xxaxx", Some("x"), true, false), "axx");
        assert_eq!(replace("a-a-a", "-", "+", None).as_deref(), Some("a+a+a"));
        assert_eq!(replace("a-a-a-a", "-", "+", Some(2)).as_deref(), Some("a+a+a-a"));
        assert_eq!(replace("a-a-a", "-", "+", Some(-1)).as_deref(), Some("a+a+a"));
    }
}