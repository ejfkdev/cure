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
    // 类别分派桶：kind → 关注该类别的规则下标（含 kinds()==空 的通用规则）。
    // 惰性建桶（每类别至多一次，O(规则数)）；Vec 按 discriminant 索引，
    // 新变体自动扩容——没有"漏桶"风险。
    let mut dispatch: Vec<Option<Box<[u32]>>> = vec![None; NodeKind::SLOT_COUNT];
    loop {
        let mut snap = walk(lang, root);
        // touched = 立即应用的编辑足迹（真已脱离树）。位图索引=node_index
        // （旧版 IdMap 哈希集：扫描循环每节点一次哈希查找 × 30k×136 轮）。
        // 已排队（未应用）的编辑不改树 → 其足迹不进 touched。
        let mut touched: Vec<u64> = Vec::new();
        // 搬移集：立即 Replace 把既有节点挪进新位置（with 复用旧节点）。
        // 这些节点的快照 parent 已过期——后续提案不得以其为目标（应用会
        // 写进幽灵父=半应用）。下一 pass 新鲜 walk 后自然解除。
        let mut moved: Vec<u64> = Vec::new();
        struct Queued<L: Lang> {
            /// 规则下标/提案节点：级联重查模式遗留（现按提案直接校验应用），
            /// 保留字段供 CURE_DEBUG_PASS 追踪与未来级联复用
            #[allow(dead_code)]
            rule_idx: usize,
            #[allow(dead_code)]
            node: L::Id,
            rule: &'static str,
            edit: Edit<L>,
            expects: Vec<Vec<L::Id>>,
        }
        let mut queued: Vec<Queued<L>> = Vec::new();
        let mut applied = false;
        #[allow(unused_mut)]
        let applied_immediate = false;

        'scan: for &id in &snap.ids {
            if bit_get(&touched, lang.node_index(id)) {
                continue;
            }
            let kind = lang.kind(id);
            let ki = kind.slot();
            debug_assert!(ki < NodeKind::SLOT_COUNT);
            let bucket: &[u32] = {
                let slot = &mut dispatch[ki];
                if slot.is_none() {
                    *slot = Some(
                        rules
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| {
                                let ks = r.kinds();
                                ks.is_empty() || ks.contains(&kind)
                            })
                            .map(|(i, _)| i as u32)
                            .collect::<Vec<u32>>()
                            .into_boxed_slice(),
                    );
                }
                slot.as_deref().unwrap()
            };
            for &ri in bucket {
                let rule = &rules[ri as usize];
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
                if crate::rule::is_replace_only(&edit) {
                    // 足迹冲突：立即应用会真改树，与已应用足迹/搬移节点相交 →
                    // 放弃本轮（下一 pass 新鲜快照重提）
                    let mut fp = Vec::new();
                    crate::rule::collect_footprint(&*lang, &edit, &mut fp);
                    if fp
                        .iter()
                        .any(|n| bit_get(&touched, lang.node_index(*n)) || bit_get(&moved, lang.node_index(*n)))
                    {
                        continue;
                    }
                    if apply_edit(lang, &snap, &edit).is_ok() {
                        // with 子树里的既有节点（不在快照中）已被搬移
                        let mut stack = with_roots_of(&edit);
                        while let Some(n) = stack.pop() {
                            if n != snap.root && !snap.parents.contains_key(&n) {
                                bit_set(&mut moved, lang.node_index(n));
                            }
                            for &c in lang.children(n) {
                                stack.push(c);
                            }
                        }
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
                            bit_set(&mut touched, lang.node_index(n));
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
                    // 足迹冲突：与其他已排队提案相交 → 放弃（守卫对提案时树成立，
                    // 应用时不再重验——无重叠 + 单次新鲜重建保证一致性）
                    let mut fp = Vec::new();
                    crate::rule::collect_footprint(&*lang, &edit, &mut fp);
                    if fp
                        .iter()
                        .any(|n| bit_get(&touched, lang.node_index(*n)) || bit_get(&moved, lang.node_index(*n)))
                    {
                        continue;
                    }
                    let mut expects = Vec::new();
                    crate::rule::collect_splice_expectations(&*lang, &edit, &mut expects);
                    queued.push(Queued {
                        rule_idx: ri as usize,
                        node: id,
                        rule: rule.name(),
                        edit,
                        expects,
                    });
                    for n in fp {
                        bit_set(&mut touched, lang.node_index(n));
                    }
                    continue 'scan;
                }
            }
        }

        // pass 末：应用结构编辑队列。
        // 立即路径的编辑已改树（含既有节点搬移到新位置——其快照 parent 已
        // 过期）：先重建 walk+prepare 使一切新鲜，再逐条校验应用。
        // 校验：Replace/Delete 按节点身份搜索定位；Splice 要求被删区间仍
        // 等于提案时记录的节点序列。失配（提案假设不再成立）→ 丢弃，
        // 下一 pass 重新提案——丢弃不降成本，单调性不受影响。
        // 重建条件：仅本 pass 有立即编辑（搬移过既有节点）时快照才过期；
        // 校验是纯结构操作（孩子表查找），不需要效果/类型缓存——只建 walk。
        if !queued.is_empty() {
            if applied_immediate {
                snap = walk(lang, root);
            }
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
        }

        report.iterations += 1;
        if !applied || report.edits >= cfg.max_edits {
            break;
        }
        lang.prepare(root);
    }
    report
}

// 【教训】脏式索引重建（B2：只重建被失效的条目）经 cff_diamond 差分
// 当场抓获失效不完备（队列编辑搬移 with 子树时，其旧容器的条目不在目标
// 祖先链上 → 陈旧事件驱动错误决策，如误删初始化）。补目标祖先链失效
// 仍不够——with 来源容器的失效需要跨树追踪。已放弃 B2，保留全量重建
// + 容量保留（B1）。

/// 位图置位（按需扩容）。
fn bit_set(bits: &mut Vec<u64>, idx: usize) {
    if idx >= bits.len() * 64 {
        bits.resize(idx / 64 + 1, 0);
    }
    bits[idx / 64] |= 1u64 << (idx % 64);
}

/// 位图读位（越界 = false）。
fn bit_get(bits: &[u64], idx: usize) -> bool {
    bits.get(idx / 64)
        .map_or(false, |w| (w >> (idx % 64)) & 1 == 1)
}

/// 编辑全部 with 根（Replace / 全 Replace Multi）。
fn with_roots_of<L: Lang>(edit: &Edit<L>) -> Vec<L::Id> {
    match edit {
        Edit::Replace { with, .. } => vec![*with],
        Edit::Multi(es) => es
            .iter()
            .filter_map(|e| match e {
                Edit::Replace { with, .. } => Some(*with),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
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
