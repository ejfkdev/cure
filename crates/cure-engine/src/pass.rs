//! Pass runner：fixed-point 循环 + 成本门槛 + 编辑账本。

use std::collections::{BTreeMap, HashSet};

use crate::kind::NodeKind;
use crate::rule::{apply_edit, Edit, RewriteCtx, Rule};
use crate::walk::walk;
use crate::lang::Lang;

#[derive(Clone, Debug)]
pub struct Config {
    /// 最大编辑次数保险丝（成本单调下降本身已保证终止）。
    pub max_edits: usize,
    /// 关闭指定名字的规则。
    pub disabled_rules: HashSet<String>,
    /// 删除全单元零引用的 private 方法（解密器/内联后的死 helper）。
    /// opt-in：反射（getDeclaredMethod）无法静态排除；名字匹配不分重载，
    /// 只会更保守地保留。默认 false。
    pub remove_dead_methods: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_edits: 100_000,
            disabled_rules: HashSet::new(),
            remove_dead_methods: false,
        }
    }
}

impl Config {
    pub fn enabled(&self, rule: &str) -> bool {
        !self.disabled_rules.contains(rule)
    }
}

#[derive(Debug, Default)]
pub struct Report {
    /// 成功应用的编辑总数。
    pub edits: usize,
    /// fixed-point 扫描轮数。
    pub iterations: usize,
    /// 按规则名统计的编辑数。
    pub by_rule: BTreeMap<&'static str, usize>,
}

/// 对 `root`（通常是方法体 Block）运行规则集直到 fixed point。
///
/// 终止性：每条被应用的编辑都严格降低总成本（`Edit::cost_reduction` 校验），
/// 成本单调下降 + 非负 ⇒ 必然终止；`max_edits` 只是防御性保险丝。
///
/// 同 pass 批量应用（正确性论证见 rule.rs 尾部注释）：
/// - 纯 Replace 立即应用（原位换孩子不移动兄弟下标，快照索引全程有效），
///   并沿祖先链失效效果缓存后继续本 pass 扫描；
/// - 含 Delete/Splice 的结构编辑记录足迹与 Splice 区间快照，pass 末按
///   (parent, 位置) 降序校验应用，校验失配（树不再符合提案假设）→ 丢弃，
///   下一 pass 重新提案——丢弃不降成本，单调性不受影响；
/// - 足迹（含子树）进 touched 集：已承诺区域不再扫描、不再被新提案触碰。
pub fn simplify<L: Lang>(
    lang: &mut L,
    root: L::Id,
    rules: &[Box<dyn Rule<L>>],
    cfg: &Config,
) -> Report {
    let mut report = Report::default();
    lang.prepare(root);
    loop {
        let snap = walk(lang, root);
        let mut touched: crate::walk::IdMap<L::Id, ()> = std::collections::HashMap::default();
        struct Queued<L: Lang> {
            rule: &'static str,
            edit: Edit<L>,
            expects: Vec<Vec<L::Id>>,
        }
        let mut queued: Vec<Queued<L>> = Vec::new();
        let mut applied = false;

        'scan: for &id in &snap.ids {
            if touched.contains_key(&id) {
                continue;
            }
            for rule in rules {
                if !cfg.enabled(rule.name()) {
                    continue;
                }
                let ctx = RewriteCtx {
                    lang: &mut *lang,
                    walk: &snap,
                };
                let proposal = rule.check(ctx, id);
                let Some(edit) = proposal else {
                    continue;
                };
                let structural = rule.structural();
                let reduction = edit.cost_reduction(lang);
                debug_assert!(
                    structural || reduction > 0,
                    "rule `{}` proposed a non-simplifying edit",
                    rule.name()
                );
                if !structural && reduction == 0 {
                    continue;
                }
                // 足迹冲突：与已承诺编辑（立即应用或排队中）相交 → 本轮放弃
                let mut fp = Vec::new();
                crate::rule::collect_footprint(&*lang, &edit, &mut fp);
                if fp.iter().any(|n| touched.contains_key(n)) {
                    continue;
                }
                if crate::rule::is_replace_only(&edit) {
                    if apply_edit(lang, &snap, &edit).is_ok() {
                        if std::env::var("CURE_DEBUG_PASS").is_ok() {
                            eprintln!(
                                "[pass] immediate {} :: {}",
                                rule.name(),
                                describe_edit(&*lang, &edit)
                            );
                        }
                        *report.by_rule.entry(rule.name()).or_insert(0) += 1;
                        report.edits += 1;
                        applied = true;
                        for n in fp {
                            touched.insert(n, ());
                        }
                        // 祖先链效果失效：被换子树的祖先聚合效果已陈旧
                        let mut cur = Some(id);
                        while let Some(a) =
                            cur.and_then(|c| snap.parents.get(&c).map(|p| p.0))
                        {
                            lang.invalidate_effect(a);
                            cur = Some(a);
                        }
                        continue 'scan;
                    }
                } else {
                    let mut expects = Vec::new();
                    crate::rule::collect_splice_expectations(&*lang, &edit, &mut expects);
                    queued.push(Queued {
                        rule: rule.name(),
                        edit,
                        expects,
                    });
                    for n in fp {
                        touched.insert(n, ());
                    }
                    continue 'scan;
                }
            }
        }

        // pass 末：延迟应用结构编辑（降序 + 校验，失配丢弃）
        queued.sort_by(|a, b| {
            let ka = crate::rule::structural_sort_key(&snap, &a.edit);
            let kb = crate::rule::structural_sort_key(&snap, &b.edit);
            kb.cmp(&ka)
        });
        for q in queued {
            let mut expects = q.expects.as_slice();
            let ok = crate::rule::verify_deferred(&*lang, &snap, &q.edit, &mut expects);
            if std::env::var("CURE_DEBUG_PASS").is_ok() {
                eprintln!(
                    "[pass] deferred {} {} :: {}",
                    if ok { "apply" } else { "DISCARD" },
                    q.rule,
                    describe_edit(&*lang, &q.edit)
                );
            }
            if ok {
                crate::rule::apply_deferred(&mut *lang, &snap, &q.edit);
                *report.by_rule.entry(q.rule).or_insert(0) += 1;
                report.edits += 1;
                applied = true;
            }
        }

        report.iterations += 1;
        if !applied || report.edits >= cfg.max_edits {
            break;
        }
        lang.prepare(root);
    }
    report
}

/// 调试用：编辑的紧凑描述（节点 kind + 变体形态，不要求 L: Debug）。
fn describe_edit<L: Lang>(lang: &L, edit: &Edit<L>) -> String {
    fn kind_of<L: Lang>(lang: &L, id: L::Id) -> &'static str {
        match lang.kind(id) {
            NodeKind::Block => "Block",
            NodeKind::VarDecl => "VarDecl",
            NodeKind::Assign => "Assign",
            NodeKind::ExprStmt => "ExprStmt",
            NodeKind::Return => "Return",
            NodeKind::If => "If",
            NodeKind::While => "While",
            NodeKind::VarRef => "VarRef",
            NodeKind::Literal => "Literal",
            NodeKind::Binary => "Binary",
            NodeKind::Call => "Call",
            NodeKind::Index => "Index",
            NodeKind::Member => "Member",
            _ => "…",
        }
    }
    fn go<L: Lang>(lang: &L, e: &Edit<L>, out: &mut String) {
        match e {
            Edit::Replace { target, with } => {
                out.push_str(&format!(
                    "Replace {}→{} ",
                    kind_of(lang, *target),
                    kind_of(lang, *with)
                ));
            }
            Edit::Delete { node } => {
                out.push_str(&format!("Delete {} ", kind_of(lang, *node)));
            }
            Edit::Splice { index, remove, .. } => {
                out.push_str(&format!("Splice @{index} -{remove} "));
            }
            Edit::Multi(es) => {
                out.push('[');
                for e in es {
                    go(lang, e, out);
                }
                out.push(']');
            }
        }
    }
    let mut s = String::new();
    go(lang, edit, &mut s);
    s
}
