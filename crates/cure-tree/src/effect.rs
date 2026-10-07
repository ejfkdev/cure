//! 聚合效果表重建（不动点 sweep）。

use cure_engine::{Effect, Lang};

/// 重建每节点聚合效果表（槽位 = arena 下标）。
///
/// 正常情况一轮升序扫描即可（children index < parent index 的解析器
/// 不变量——`Lang::id_of_index` 按下标遍历）。但 `Edit::Replace` 注入
/// 的新节点 append 在 arena 末尾、index 大于其（旧）父节点——单轮
/// sweep 会让父聚合到 Unknown 并污染祖先链。故迭代到不动点（违例深度
/// 有限，2 轮内收敛；上限 4 轮防御）。
pub fn rebuild_effects<L: Lang>(lang: &L, node_count: usize) -> Vec<Option<Effect>> {
    let mut cache: Vec<Option<Effect>> = vec![None; node_count];
    for _round in 0..4 {
        let mut changed = false;
        for i in 0..node_count {
            let id = lang.id_of_index(i);
            let mut e = lang.own_effect(id);
            for &c in lang.children(id) {
                e = e.worst(
                    cache.get(lang.node_index(c)).and_then(|x| *x).unwrap_or(Effect::Unknown),
                );
            }
            if cache[i] != Some(e) {
                cache[i] = Some(e);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    cache
}
