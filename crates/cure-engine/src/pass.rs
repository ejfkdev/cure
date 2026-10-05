//! Pass runner：fixed-point 循环 + 成本门槛 + 编辑账本。

use std::collections::{BTreeMap, HashSet};

use crate::rule::{apply_edit, RewriteCtx, Rule};
use crate::walk::walk;
use crate::lang::Lang;

#[derive(Clone, Debug)]
pub struct Config {
    /// 最大编辑次数保险丝（成本单调下降本身已保证终止）。
    pub max_edits: usize,
    /// 关闭指定名字的规则。
    pub disabled_rules: HashSet<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_edits: 100_000,
            disabled_rules: HashSet::new(),
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
        let mut applied = false;
        'scan: for &id in &snap.ids {
            for rule in rules {
                if !cfg.enabled(rule.name()) {
                    continue;
                }
                let ctx = RewriteCtx {
                    lang: &mut *lang,
                    walk: &snap,
                };
                let proposal = rule.check(ctx, id);
                if let Some(edit) = proposal {
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
                    if apply_edit(lang, &snap, &edit).is_ok() {
                        *report.by_rule.entry(rule.name()).or_insert(0) += 1;
                        report.edits += 1;
                        applied = true;
                        break 'scan;
                    }
                }
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
