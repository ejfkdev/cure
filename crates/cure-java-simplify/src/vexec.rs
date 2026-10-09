//! 虚拟执行 V2：对**纯计算语句段**的静态求值与整体折叠。
//!
//! 目标形态：混淆器/反编译器的静态字符串解密机（fernflower 语料中
//! 12/66 文件、占残留噪声行数 ~90%）——`"密文".toCharArray()` +
//! `switch (i % 5)` 常量键 + `(char)(c ^ k)` 循环 + `new String(arr)
//! .intern()`，整体是一个**无外部读、仅写字段/局部**的纯计算段。
//!
//! 方法：小型树遍历解释器，对语句段逐步求值；任何不可证明的东西
//! （字段读、未知调用、try/throw、数组越界、除零、步数/长度超预算）
//! 立即保守 abort（Err(())），段保持原样。求值成功的段被重写为
//! 「逃逸局部 = 常量」+「写字段 = 常量」的直线赋值。
//!
//! 正确性边界（全部保守）：
//! - 字段**写**被记录为段输出（按首次写顺序、末值为准）；字段**读**
//!   一律 abort（外部状态未知）；
//! - 变量按名字键寻址——遇同名遮蔽（VarDecl 名已在作用域）直接 abort；
//! - 算术按 JLS i32/i64 环绕；除/余零 abort（可能抛异常）；
//! - 数组下标越界 abort（可能抛 AIOOBE）；
//! - 循环带步数预算（默认 10 万步）；字符串长度预算（默认 1 MB）。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cure_engine::kind::{BinOp, NodeKind, UnOp};
use cure_engine::lang::Lang;
use cure_java_ast::{JavaAst, JavaId, JType, Lit, NodeData, NumVal};

/// 执行预算。
const MAX_STEPS: usize = 2_000_000;
/// 字符串（及 char[]）长度预算：超过即跳过（特别长的串折叠无谓耗性能）。
const MAX_STR_LEN: usize = 1 << 20;

/// 值的嵌套结构中是否（直接或间接）引用目标数组（按 Rc 地址）。
/// 带 visited 防环——输入本身可能已含环（防御性）。
fn value_references_array(v: &VVal, target_addr: usize) -> bool {
    let mut visited: std::collections::HashSet<usize> = Default::default();
    fn walk(v: &VVal, target: usize, visited: &mut std::collections::HashSet<usize>) -> bool {
        match v {
            VVal::CA(_) => false, // CA 元素是 char 不嵌套
            VVal::SA(a) => {
                let addr = Rc::as_ptr(a) as usize;
                if addr == target {
                    return true;
                }
                if !visited.insert(addr) {
                    return false; // 已走过的环
                }
                for e in a.borrow().iter() {
                    if walk(e, target, visited) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
    walk(v, target_addr, &mut visited)
}

/// 值域（足够覆盖解密机家族；浮点/引用语义一律 abort）。
#[derive(Clone, Debug)]
pub(crate) enum VVal {
    /// int/byte/short/char 算术域（JLS 提升后 i32）
    I(i32),
    /// long
    L(i64),
    /// boolean
    B(bool),
    /// String
    S(String),
    /// char[]（**引用语义**：Java 数组别名——Rc 共享底层缓冲）
    CA(Rc<RefCell<Vec<char>>>),
    /// 引用数组（String[] 等，同引用语义）；元素可为 Undef
    SA(Rc<RefCell<Vec<VVal>>>),
    /// 数组槽未初始化（new String[n] 的默认值——读取 abort）
    Undef,
}

impl VVal {
    fn as_i32(&self) -> Option<i32> {
        match self {
            VVal::I(v) => Some(*v),
            _ => None,
        }
    }
}

/// 控制流。
#[derive(Clone, Debug)]
pub(crate) enum Flow {
    Normal,
    /// break（可带标签名）
    Break(Option<String>),
    /// continue（可带标签名）
    Continue(Option<String>),
    /// return（静态块/方法终止；载荷 = 返回值——重写须物化保留）
    Return(Option<VVal>),
}

type R<T> = Result<T, ()>;

/// 执行器。
pub(crate) struct Exec<'a> {
    ast: &'a JavaAst,
    steps: usize,
    /// 局部变量（名字键 → 值）。键由 var_key 取。
    vars: HashMap<u32, VVal>,
    /// 段内字段写（名字 → 末值；插入顺序保留）。
    field_writes: Vec<(u32, VVal)>,
    /// 局部声明序（首次声明顺序——重写按此序重排）。
    decl_order: Vec<u32>,
    /// 块作用域栈（每层 = 该块声明的名字键集合；退出弹层并清值）。
    /// 循环体的声明随迭代弹层——每轮重新声明不是遮蔽。
    scopes: Vec<Vec<u32>>,
    /// 步数耗尽（见 budget_exhausted）。
    exhausted: bool,
    /// 局部声明的窄域（名字键 → 2=char/3=byte/4=short；0=宽）。
    /// JLS 复合赋值隐式收窄：`byte b=100; b+=100` → (byte)200 = -56
    /// （javac 真值对拍抓获：vexec 曾输出 200）。
    var_width: HashMap<u32, u8>,
    /// 局部声明类型（名字键 → JType）：逃逸局部材料化按**原声明类型**
    /// 恢复——`Object o = i + j` 曾材料化成 `var o = 3`（var 推断 int →
    /// 接收位不可解引用——R14 族 A 抓获：R13 的接收位守卫只在
    /// LocalPropagation，vexec 物化路径同型破坏未覆盖）
    decl_tys: HashMap<u32, JType>,
    /// 执行效应日志（按发生序）：字段写与不透明 Class.forName 假设。
    /// 材料化按日志序重放——语句位置/顺序与原执行一致。
    effect_log: Vec<EffectEvent>,
    /// 屏障穿越字段集（OpaqueFieldWrite 的目标——值 Undef，参与运算 → abort）
    opaque_fields: std::collections::HashSet<u32>,
    /// try 语境深度（exec_try 体/finally 与 run_level Try 下降层内 >0）。
    /// 屏障重放是「假设调用成功」的裸赋值——原语句在 try/catch/finally
    /// 语境内时运行时可能抛出被 catch 改道（P1g/P1h/P1l 攻击抓获：
    /// try/catch/finally 整体消失，输入 caught|3 输出 EIIE）。深度 >0
    /// 时屏障拒绝 → 语句保守失败 → try 留 rest 原样保真。
    try_depth: usize,
}

/// 执行效应（材料化重放的单位）。
#[derive(Clone, Debug)]
pub(crate) enum EffectEvent {
    /// 裸名赋值 → 字段写（名字键）。
    FieldWrite(u32),
    /// `Class.forName("<类名>")` 假设成功的不透明语句
    ///（副作用 = 类初始化；重放为原语句 + 字面量实参）。
    /// stmt/call 是原节点 id（材料化时克隆改造）。
    ClassLoad { stmt: JavaId, call: JavaId, name: String },
    /// 裸名字段 = <不透明调用>(实参) 屏障（P1 简化空间——探针实验证明
    /// a3/a5/a4/r 的解密级联被这类语句截断，替换后 -3,060 行/-53.4%）。
    /// 语义：字段值不可知 → Undef；副作用 = 原调用执行。材料化重放为
    /// 「name = <原调用子树，可求值实参代常量>」。stmt/call 原节点 id，
    /// k = 字段名字键，args = 已求值实参（None = 不可求值保留原文）。
    OpaqueFieldWrite {
        k: u32,
        stmt: JavaId,
        call: JavaId,
        args: Vec<Option<VVal>>,
    },
}

impl<'a> Exec<'a> {
    pub(crate) fn new(ast: &'a JavaAst) -> Self {
        Exec { ast, steps: 0, exhausted: false, vars: HashMap::new(), field_writes: Vec::new(), decl_order: Vec::new(), scopes: vec![Vec::new()], var_width: HashMap::new(), effect_log: Vec::new(), opaque_fields: Default::default(), try_depth: 0, decl_tys: HashMap::new() }
    }

    /// 步数预算耗尽标志（规则据此放弃整段重写——中途状态不是
    /// 收敛值，回写即捏造常量。差分抓获：2^31 次循环被截断后
    /// i=142855/k=142863 当入口常量，违反 k−i=7 循环不变量）
    pub(crate) fn budget_exhausted(&self) -> bool {
        self.steps > MAX_STEPS
    }

    fn tick(&mut self) -> R<()> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            self.exhausted = true;
            return Err(());
        }
        Ok(())
    }

    fn key(&self, id: JavaId) -> R<u32> {
        self.ast.var_key(id).ok_or(())
    }


    // ---- 语句 ----

    /// 执行语句序列；Return/Break 传播到调用方判断。
    pub(crate) fn exec_stmts(&mut self, stmts: &[JavaId]) -> R<Flow> {
        for &s in stmts {
            match self.exec_stmt(s)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    pub(crate) fn exec_stmt(&mut self, s: JavaId) -> R<Flow> {
        self.tick()?;
        match self.ast.data(s).clone() {
            NodeData::VarDecl { ty, .. } => {
                let k = self.key(s)?;
                if self.scopes.iter().any(|sc| sc.contains(&k)) {
                    return Err(()); // 同名遮蔽（活跃作用域内）——名字键寻址失效
                }
                let v = match self.ast.children(s).first() {
                    Some(&init) => self.eval(init)?,
                    None => VVal::Undef, // 无 init：未初始化局部（读取即 abort）
                };
                let v = self.coerce_decl(v, &ty);
                let w = match ty {
                    JType::Char => 2,
                    JType::Byte => 3,
                    JType::Short => 4,
                    // long=5：非窄化（narrow 的 _ 臂直通），供材料化侧
                    // 恢复声明域（Int 字面量 → 5L——t06d 同源）
                    JType::Long => 5,
                    _ => 0,
                };
                self.var_width.insert(k, w);
                self.decl_tys.insert(k, ty.clone());
                self.vars.insert(k, v);
                self.decl_order.push(k);
                self.scopes.last_mut().unwrap().push(k);
                Ok(Flow::Normal)
            }
            NodeData::Assign { op } => {
                let ch = self.ast.children(s).to_vec();
                if ch.len() != 2 {
                    return Err(());
                }
                let (target, value) = (ch[0], ch[1]);
                match op {
                    Some(op) => self.exec_compound_assign(target, value, op),
                    None => {
                        // P1 屏障穿越：`裸名字段 = <未知调用>(实参)` 且实参
                        // 全部可求值 → 原语句保留为不透明副作用、字段值置
                        // Undef-不透明、**继续执行**（a3/a5/a4/r 的解密
                        // 级联被这类出口语句截断——探针 -3,060 行/-53.4%）
                        if let Some(ev) = self.try_opaque_field_barrier(s, target, value)? {
                            self.effect_log.push(ev);
                            return Ok(Flow::Normal);
                        }
                        let v = self.eval(value)?;
                        self.store(target, v)
                    }
                }
            }
            NodeData::ExprStmt => {
                let inner = *self.ast.children(s).first().ok_or(())?;
                match self.ast.kind(inner) {
                    // 赋值语句（b = x; / arr[i] = x;）走语句分发
                    NodeKind::Assign => self.exec_stmt(inner),
                    // 自增/自减作为表达式语句（++i / i++）
                    NodeKind::Unary => {
                        let op = self.ast.un_op(inner).ok_or(())?;
                        if !op.is_incdec() {
                            if self.try_class_forname(s, inner)? {
                                return Ok(Flow::Normal);
                            }
                            self.eval(inner)?;
                            return Ok(Flow::Normal);
                        }
                        let tgt = *self.ast.children(inner).first().ok_or(())?;
                        self.exec_incdec(tgt, op)
                    }
                    _ => {
                        if self.try_class_forname(s, inner)? {
                            return Ok(Flow::Normal);
                        }
                        self.eval(inner)?;
                        Ok(Flow::Normal)
                    }
                }
            }
            NodeData::If => {
                let ch = self.ast.children(s).to_vec();
                let cond = self.eval(ch[0])?;
                let b = match cond {
                    VVal::B(b) => b,
                    _ => return Err(()),
                };
                if b {
                    self.exec_stmt(ch[1])
                } else if let Some(&els) = ch.get(2) {
                    self.exec_stmt(els)
                } else {
                    Ok(Flow::Normal)
                }
            }
            NodeData::While => {
                let ch = self.ast.children(s).to_vec();
                loop {
                    self.tick()?;
                    let cond = self.eval(ch[0])?;
                    match cond {
                        VVal::B(true) => {}
                        VVal::B(false) => return Ok(Flow::Normal),
                        _ => return Err(()),
                    }
                    match self.exec_stmt(ch[1])? {
                        Flow::Normal | Flow::Continue(None) => {}
                        Flow::Break(None) => return Ok(Flow::Normal),
                        // 带标签 break/continue 目标本循环之外——向上传递给
                        // Label 层捕获（曾转 Err 中止整段执行：解密风暴的
                        // `break label316` 使 static_exec 前缀在首个含标签
                        // break 的循环处截断——a5 抓获）
                        f => return Ok(f),
                    }
                }
            }
            NodeData::DoWhile => {
                let ch = self.ast.children(s).to_vec();
                loop {
                    self.tick()?;
                    match self.exec_stmt(ch[0])? {
                        Flow::Normal | Flow::Continue(None) => {}
                        Flow::Break(None) => return Ok(Flow::Normal),
                        f => return Ok(f),
                    }
                    let cond = self.eval(ch[1])?;
                    match cond {
                        VVal::B(true) => {}
                        VVal::B(false) => return Ok(Flow::Normal),
                        _ => return Err(()),
                    }
                }
            }
            NodeData::For { inits, has_cond, .. } => {
                // for-init 声明的作用域 = 整个 for（Java 语义）——不同段
                // 复用同名循环变量不是遮蔽（bd.java：106 段 for(int var2…)）
                self.scopes.push(Vec::new());
                let r = self.exec_for(s, inits, has_cond);
                for k in self.scopes.pop().unwrap() {
                    self.vars.remove(&k);
                    self.var_width.remove(&k);
                }
                r
            }
            NodeData::Switch => self.exec_switch(s),
            NodeData::Label { name } => {
                // label: stmt —— break/continue <label> 在此捕获（按名字）。
                // continue <label>：重跑循环体（While 内部重判条件——
                // `continue outer` 语义；tick 预算防病态非循环体循环）
                let body = *self.ast.children(s).first().ok_or(())?;
                loop {
                    self.tick()?;
                    match self.exec_stmt(body)? {
                        Flow::Break(Some(l)) if l == *name => return Ok(Flow::Normal),
                        Flow::Continue(Some(l)) if l == *name => continue,
                        other => return Ok(other),
                    }
                }
            }
            NodeData::Break { label } => {
                Ok(Flow::Break(label.clone()))
            }
            NodeData::Continue { label } => {
                Ok(Flow::Continue(label.clone()))
            }
            NodeData::Try => self.exec_try(s),
            NodeData::Return => {
                let v = match self.ast.children(s).first() {
                    Some(&e) => Some(self.eval(e)?),
                    None => None, // void return / 静态块裸 return
                };
                Ok(Flow::Return(v))
            }
            NodeData::Block | NodeData::Group => {
                let ch = self.ast.children(s).to_vec();
                self.scopes.push(Vec::new());
                let r = self.exec_stmts(&ch);
                // 弹层：该块声明的局部出作用域（值清除——后续读即 abort）
                for k in self.scopes.pop().unwrap() {
                    self.vars.remove(&k);
                    self.var_width.remove(&k);
                }
                r
            }
            NodeData::Empty => Ok(Flow::Normal),
            // 裸表达式语句（for 步进等不包 ExprStmt 的场景；eval 含
            // 存储副作用——自增/自减在 eval 的 Unary 分支处理）
            NodeData::Call => {
                if self.try_class_forname(s, s)? {
                    return Ok(Flow::Normal);
                }
                self.eval(s)?;
                Ok(Flow::Normal)
            }
            NodeData::Unary { .. }
            | NodeData::Binary { .. }
            | NodeData::New { .. } => {
                self.eval(s)?;
                Ok(Flow::Normal)
            }
            // 声明层/不支持的语句：保守 abort
            _ => Err(()),
        }
    }

    fn coerce_decl(&self, v: VVal, ty: &JType) -> VVal {
        // 声明窄化（char c = <int 字面量> 等）：I → char 域裁剪。
        // L 值按声明域截断（long 声明除外——保持）
        match (v, ty) {
            // 拓宽方向（I → long）：int 字面量 init 的 long 声明曾存 I，
            // 后续 g+1 在 i32 环绕（T6D/H1 抓获：2147483647+1 → -2147483648，
            // 静默错值——收窄方向修了拓宽漏了）
            (VVal::I(x), JType::Long) => VVal::L(x as i64),
            (VVal::I(x), JType::Char) => VVal::I(x as u16 as i32),
            (VVal::I(x), JType::Byte) => VVal::I(x as i8 as i32),
            (VVal::I(x), JType::Short) => VVal::I(x as i16 as i32),
            (VVal::L(x), JType::Char) => VVal::I(x as u16 as i32),
            (VVal::L(x), JType::Byte) => VVal::I(x as i8 as i32),
            (VVal::L(x), JType::Short) => VVal::I(x as i16 as i32),
            (VVal::L(x), JType::Int) => VVal::I(x as i32),
            (v, _) => v,
        }
    }

    /// 存储到目标（VarRef / Index / Member）。
    fn store(&mut self, target: JavaId, v: VVal) -> R<Flow> {
        self.tick()?;
        match self.ast.data(target).clone() {
            NodeData::VarRef { .. } => {
                let k = self.key(target)?;
                if self.scopes.iter().any(|sc| sc.contains(&k)) {
                    // 按声明域收窄（JLS 复合赋值/赋值的隐式收窄）。
                    // L 值必须按目标域截断：`byte b=100; b+=100L` →
                    // (byte)200 = -56；`int i; i+=3000000000L` → (int)…
                    //（边界攻击 t04e/t04i 抓获：L 曾直接穿透成 L(200)，
                    // 材料化 201L 不可编译且值错——原程序 -55）
                    let w = self.var_width.get(&k).copied().unwrap_or(0);
                    let v = match (w, v) {
                        // long 域（5）保持
                        (5, v) => v,
                        // int 域（0）：L → I 截断
                        (0, VVal::L(x)) => VVal::I(x as i32),
                        // 窄域（2/3/4）：I 收窄；L 先 as i32 再收窄
                        (wn, VVal::I(x)) if wn != 0 => VVal::I(narrow(x, wn)),
                        (wn, VVal::L(x)) if wn != 0 => VVal::I(narrow(x as i32, wn)),
                        (_, v) => v,
                    };
                    self.vars.insert(k, v);
                    Ok(Flow::Normal)
                } else {
                    // 未声明裸名 = 字段写（静态初始化语境；方法参数由
                    // 规则侧 param 守卫拦截）
                    self.record_field_write_by_key(k, v)
                }
            }
            NodeData::Index => {
                let ch = self.ast.children(target).to_vec();
                let (base, idx) = (ch[0], ch[1]);
                let arr = self.eval_ref_value(base)?;
                let iv = self.eval(idx)?.as_i32().ok_or(())?;
                let i = usize::try_from(iv).map_err(|_| ())?;
                match arr {
                    VVal::CA(a) => {
                        let c = match v {
                            VVal::I(x) => char::from_u32(x as u32 as u16 as u32).ok_or(())?,
                            _ => return Err(()),
                        };
                        *a.borrow_mut().get_mut(i).ok_or(())? = c;
                        Ok(Flow::Normal)
                    }
                    VVal::SA(a) => {
                        // 自引用数组环检测：把 v 存进 a 时若 a 已在 v 的
                        // 结构里（netty MessageFormatterTest 的
                        // cyclicA[0]=cyclicA）→ SA(Rc) 自包含 →
                        // deep_snapshot/materialize 的递归展开无限栈溢出
                        //（整文件崩溃）。拒绝该存储（保守 abort 段）
                        if value_references_array(&v, Rc::as_ptr(&a) as usize) {
                            return Err(());
                        }
                        *a.borrow_mut().get_mut(i).ok_or(())? = v;
                        Ok(Flow::Normal)
                    }
                    _ => Err(()),
                }
            }
            NodeData::Member { .. } => {
                // this.f = v —— v1 abort（解密机的字段写是裸名 VarRef）
                Err(())
            }
            _ => Err(()),
        }
    }

    /// 字段写（裸名 VarRef 未声明 → 记录为段输出；末值为准）。
    fn record_field_write_by_key(&mut self, k: u32, v: VVal) -> R<Flow> {
        // long 字段的 I 值提升（G4 抓获：f=2147483647; f=f+1 曾按 i32
        // 环绕——字段读回后无域提升）
        let v = match v {
            VVal::I(x) => {
                let is_long = self
                    .ast
                    .name_of_key(k)
                    .and_then(|n| self.ast.field_types.get(n.as_str()))
                    .is_some_and(|t| matches!(t, JType::Long));
                if is_long {
                    VVal::L(x as i64)
                } else {
                    VVal::I(x)
                }
            }
            other => other,
        };
        if let Some(slot) = self.field_writes.iter_mut().find(|(n, _)| *n == k) {
            slot.1 = v;
        } else {
            self.field_writes.push((k, v));
        }
        self.effect_log.push(EffectEvent::FieldWrite(k));
        Ok(Flow::Normal)
    }

    fn exec_compound_assign(&mut self, target: JavaId, value: JavaId, op: BinOp) -> R<Flow> {
        // x op= v：读旧值、运算、写回（JLS 复合赋值隐式收窄）
        let old = match self.ast.kind(target) {
            NodeKind::VarRef => {
                let k = self.key(target)?;
                match self.vars.get(&k) {
                    Some(VVal::Undef) => return Err(()),
                    Some(v) => v.clone(),
                    None => return Err(()), // 字段读——未知状态
                }
            }
            NodeKind::Index => {
                let idx_val = self.eval(*self.ast.children(target).get(1).ok_or(())?)?;
                let base = *self.ast.children(target).first().ok_or(())?;
                let arr = self.eval_ref_value(base)?;
                let iv = idx_val.as_i32().ok_or(())?;
                let i = usize::try_from(iv).map_err(|_| ())?;
                match arr {
                    VVal::CA(a) => VVal::I(*a.borrow().get(i).ok_or(())? as u32 as i32),
                    VVal::SA(a) => a.borrow().get(i).ok_or(())?.clone(),
                    _ => return Err(()),
                }
            }
            _ => return Err(()),
        };
        if matches!(old, VVal::Undef) {
            return Err(());
        }
        let rhs = self.eval(value)?;
        let combined = self.binop(op, &old, &rhs)?;
        // JLS 复合赋值收窄：目标域窄化（char/byte/short）
        let combined = match (&combined, self.target_width(target)) {
            (VVal::I(x), Some(w)) => VVal::I(narrow(*x, w)),
            (v, _) => v.clone(),
        };
        self.store(target, combined)
    }

    fn target_width(&self, target: JavaId) -> Option<u8> {
        // 1=int/long? 2=char 3=byte 4=short
        match self.ast.data(target).clone() {
            NodeData::VarRef { .. } => None, // 局部域已由 coerce_decl 维护
            NodeData::Index => {
                let base = *self.ast.children(target).first()?;
                match self.ast.kind(base) {
                    NodeKind::VarRef => {
                        let k = self.ast.var_key(base)?;
                        match self.vars.get(&k)? {
                            VVal::CA(_) => Some(2),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn exec_incdec(&mut self, target: JavaId, op: UnOp) -> R<Flow> {
        let old = match self.ast.kind(target) {
            NodeKind::VarRef => {
                let k = self.key(target)?;
                self.vars.get(&k).cloned().ok_or(())?
            }
            _ => return Err(()),
        };
        let one = VVal::I(1);
        let newv = match op {
            UnOp::PreInc | UnOp::PostInc => self.binop(BinOp::Add, &old, &one)?,
            UnOp::PreDec | UnOp::PostDec => self.binop(BinOp::Sub, &old, &one)?,
            _ => return Err(()),
        };
        self.store(target, newv)
    }

    /// try 的确定性执行：children = [resource…, try_block, catch…, (finally)?]。
    ///
    /// **可靠性论证**：解释器只执行全常量程序（任何字段读/未知调用/
    /// 除零/越界/不可表示值都立即 abort）——body 执行成功即在**确定性
    /// 语义**下证明了本路径无异常 ⇒ catch 不可达（bd.java 的
    /// `try { var = "常量"; } catch (Exception e) { break label; }`）。
    /// finally 恒执行（body 的 Flow 传播前先走 finally）。
    /// try-with-resources（有 resource 前置孩子）→ abort（close 语义）。
    fn exec_try(&mut self, s: JavaId) -> R<Flow> {
        let ch = self.ast.children(s).to_vec();
        if ch.is_empty() {
            return Err(());
        }
        // 布局识别：resource…（非 Block 非 Catch 的头部）→ abort
        let mut i = 0usize;
        if i < ch.len()
            && self.ast.kind(ch[i]) != NodeKind::Block
            && self.ast.kind(ch[i]) != NodeKind::Catch
        {
            return Err(()); // try-with-resources 资源 —— close 语义不可模拟
        }
        let try_block = ch.get(i).copied().ok_or(())?;
        if self.ast.kind(try_block) != NodeKind::Block {
            return Err(());
        }
        i += 1;
        // catch… + 可选 finally（catch 之后若还有孩子，最后一个是 finally）
        let mut finally_block: Option<JavaId> = None;
        for &c in &ch[i..] {
            match self.ast.kind(c) {
                NodeKind::Catch => {}
                NodeKind::Block => finally_block = Some(c),
                _ => return Err(()),
            }
        }
        // body 成功 ⇒ catch 不可达；Flow 传播前先执行 finally。
        // try 语境计数：体内/finally 的屏障被拒绝（重放裸赋值会丢异常
        // 窗口——P1g/P1h/P1l 攻击抓获）
        self.try_depth += 1;
        let flow = self.exec_stmt(try_block);
        self.try_depth -= 1;
        let flow = flow?;
        if let Some(f) = finally_block {
            self.try_depth += 1;
            let fflow = self.exec_stmt(f);
            self.try_depth -= 1;
            match fflow? {
                Flow::Normal => {}
                // finally 里的 break/return 覆盖 body 的 Flow（Java 语义）
                other => return Ok(other),
            }
        }
        Ok(flow)
    }

    fn exec_for(&mut self, s: JavaId, inits: u8, has_cond: bool) -> R<Flow> {
        let ch = self.ast.children(s).to_vec();
        let (init_part, rest) = ch.split_at(inits as usize);
        self.exec_stmts(init_part)?;

        let cond_part = if has_cond { &rest[..1] } else { &rest[..0] };
        let step_start = inits as usize + has_cond as usize;
        let step_part = &ch[step_start..ch.len() - 1];
        let body = ch[ch.len() - 1];
        loop {
            self.tick()?;
            if has_cond {
                let cond = self.eval(cond_part[0])?;
                match cond {
                    VVal::B(true) => {}
                    VVal::B(false) => return Ok(Flow::Normal),
                    _ => return Err(()),
                }
            }
            match self.exec_stmt(body)? {
                Flow::Normal | Flow::Continue(None) => {}
                Flow::Break(None) => return Ok(Flow::Normal),
                // 带标签 break/continue 目标本循环之外——向上传递（同 While）
                f => return Ok(f),
            }
            for &st in step_part {
                self.eval(st)?; // 裸表达式步进（++i 等——eval 含存储副作用）
            }
        }
    }

    fn exec_switch(&mut self, s: JavaId) -> R<Flow> {
        let ch = self.ast.children(s).to_vec();
        let subject = self.eval(ch[0])?;
        let sv = subject.as_i32().ok_or(())?;
        // children: [subject, case…]；Case{labels, is_default, arrow}
        // labels = 前 labels 个孩子是 case 标签表达式
        let mut entry: Option<usize> = None;
        let mut default_idx: Option<usize> = None;
        for (ci, &case) in ch.iter().enumerate().skip(1) {
            let (labels, is_default) = match self.ast.data(case).clone() {
                NodeData::Case { labels, is_default, .. } => (labels as usize, is_default),
                _ => return Err(()),
            };
            if is_default {
                default_idx = Some(ci);
            }
            let lch = self.ast.children(case).to_vec();
            for &l in lch.iter().take(labels) {
                let lv = self.eval(l)?.as_i32().ok_or(())?;
                if lv == sv {
                    entry = Some(ci);
                    break;
                }
            }
            if entry.is_some() {
                break;
            }
        }
        let entry = match entry.or(default_idx) {
            Some(e) => e,
            None => return Ok(Flow::Normal), // 无匹配无 default
        };
        // 从 entry 顺序执行（落穿），直到 break / 段尾
        let mut ci = entry;
        while ci < ch.len() {
            let case = ch[ci];
            let (labels, arrow) = match self.ast.data(case).clone() {
                NodeData::Case { labels, arrow, .. } => (labels as usize, arrow),
                _ => return Err(()),
            };
            let body = self.ast.children(case).to_vec();
            let stmts = &body[labels..];
            // 同一 case 的多条语句为直接孩子
            match self.exec_stmts(stmts)? {
                Flow::Normal => {}
                // 裸 break：退出 switch；带标签 break：目标在 switch 之外
                // ——向上传递给 Label 层捕获（曾整体吞为 Normal：case 内
                // `break label` 跳转丢失）
                Flow::Break(None) => return Ok(Flow::Normal),
                other => return Ok(other),
            }
            if arrow {
                return Ok(Flow::Normal);
            }
            ci += 1;
        }
        Ok(Flow::Normal)
    }

    // ---- 表达式 ----

    fn eval(&mut self, id: JavaId) -> R<VVal> {
        self.tick()?;
        match self.ast.data(id).clone() {
            NodeData::Literal(l) => self.of_lit(l),
            NodeData::VarRef { .. } => {
                let k = self.key(id)?;
                // P1 不透明字段：屏障穿越写入的字段值不可知——参与任何
                // 运算/下标/条件判定 → abort（保守）
                if self.opaque_fields.contains(&k) {
                    return Err(());
                }
                match self.vars.get(&k) {
                    Some(VVal::Undef) => Err(()),
                    Some(v) => Ok(v.clone()),
                    None => {
                        // 段内先写后读：纯段中无任何中间可观察点，读到的
                        // 必是本段写入的值（<clinit> 单线程 + 中间语句全纯）
                        if let Some((_, v)) =
                            self.field_writes.iter().find(|(n, _)| *n == k)
                        {
                            if matches!(v, VVal::Undef) {
                                return Err(()); // 不透明
                            }
                            return Ok(v.clone());
                        }
                        Err(())
                    }
                }
            }
            NodeData::Binary { op } => {
                let ch = self.ast.children(id).to_vec();
                if op.is_short_circuit() {
                    let l = self.eval(ch[0])?;
                    let lb = match l {
                        VVal::B(b) => b,
                        _ => return Err(()),
                    };
                    // 右支可能短路——但两支都必须是纯的（eval 已保证）
                    let r = self.eval(ch[1])?;
                    let rb = match r {
                        VVal::B(b) => b,
                        _ => return Err(()),
                    };
                    Ok(VVal::B(match op {
                        BinOp::And => lb && rb,
                        BinOp::Or => lb || rb,
                        _ => return Err(()),
                    }))
                } else {
                    let l = self.eval(ch[0])?;
                    let r = self.eval(ch[1])?;
                    self.binop(op, &l, &r)
                }
            }
            NodeData::Unary { op } => {
                let inner = *self.ast.children(id).first().ok_or(())?;
                if op.is_incdec() {
                    // 前后缀自增作为表达式——值/副作用区分（标量与数组下标通用）
                    let old = self.eval(inner)?;
                    let one = VVal::I(1);
                    let newv = match op {
                        UnOp::PreInc | UnOp::PostInc => self.binop(BinOp::Add, &old, &one)?,
                        _ => self.binop(BinOp::Sub, &old, &one)?,
                    };
                    self.store(inner, newv.clone())?;
                    return Ok(match op {
                        UnOp::PreInc | UnOp::PreDec => newv,
                        _ => old,
                    });
                }
                let v = self.eval(inner)?;
                match (op, v) {
                    (UnOp::Not, VVal::B(b)) => Ok(VVal::B(!b)),
                    (UnOp::Neg, VVal::I(x)) => Ok(VVal::I(x.wrapping_neg())),
                    (UnOp::Neg, VVal::L(x)) => Ok(VVal::L(x.wrapping_neg())),
                    // 一元 + = 数值提升（VVal 的 char/byte/short 已是 I
                    // 域——恒等）；I/L/D/F 同域直通
                    (UnOp::Plus, ref x @ (VVal::I(_) | VVal::L(_))) => Ok(x.clone()),
                    (UnOp::BitNot, VVal::I(x)) => Ok(VVal::I(!x)),
                    (UnOp::BitNot, VVal::L(x)) => Ok(VVal::L(!x)),
                    _ => Err(()),
                }
            }
            NodeData::Paren => {
                let inner = *self.ast.children(id).first().ok_or(())?;
                self.eval(inner)
            }
            NodeData::Cast { ty } => {
                let inner = *self.ast.children(id).first().ok_or(())?;
                let v = self.eval(inner)?;
                self.cast(v, &ty)
            }
            NodeData::Ternary => {
                let ch = self.ast.children(id).to_vec();
                let c = self.eval(ch[0])?;
                match c {
                    VVal::B(true) => self.eval(ch[1]),
                    VVal::B(false) => self.eval(ch[2]),
                    _ => Err(()),
                }
            }
            NodeData::Index => {
                let ch = self.ast.children(id).to_vec();
                let base = self.eval_ref_value(ch[0])?;
                let iv = self.eval(ch[1])?.as_i32().ok_or(())?;
                let i = usize::try_from(iv).map_err(|_| ())?;
                match base {
                    VVal::CA(a) => Ok(VVal::I(*a.borrow().get(i).ok_or(())? as u32 as i32)),
                    VVal::SA(a) => Ok(a.borrow().get(i).ok_or(())?.clone()),
                    _ => Err(()),
                }
            }
            NodeData::Member { name } => {
                // arr.length
                let recv = *self.ast.children(id).first().ok_or(())?;
                if self.ast.sn(name) != "length" {
                    return Err(());
                }
                let arr = self.eval_ref_value(recv)?;
                match arr {
                    VVal::CA(a) => Ok(VVal::I(a.borrow().len() as i32)),
                    VVal::SA(a) => Ok(VVal::I(a.borrow().len() as i32)),
                    _ => Err(()),
                }
            }
            NodeData::Call => {
                let ch = self.ast.children(id).to_vec();
                self.eval_call(&ch)
            }
            NodeData::New { .. } => {
                // 裸 new String(...)：**身份承载**——每次求值产生独立堆
                // 对象，==/same()/assertNotSame/identityHashMap 敏感。折叠
                // 成驻留字面量 = 身份坍缩（mockito MatchersTest 抓获：
                // assertNotSame(one,two) 曾转成 assertNotSame("1243","1243")
                // 直接 AssertionError）。驻留形态 `new String(x).intern()`
                // 在 eval_call 的 intern 分支展开求值
                Err(())
            }
            NodeData::NewArray { dims, sized, .. } => {
                // new T[n]（一维带尺寸）；new T[]{…} 由 ArrayLit 路径
                let ch = self.ast.children(id).to_vec();
                let _ = dims;
                let n = if sized > 0 {
                    let v = self.eval(*ch.first().ok_or(())?)?;
                    usize::try_from(v.as_i32().ok_or(())?).map_err(|_| ())?
                } else {
                    return Err(()); // 无尺寸分配（new T[]… 非法形态）
                };
                if n > MAX_STR_LEN {
                    return Err(());
                }
                Ok(VVal::SA(Rc::new(RefCell::new(vec![VVal::Undef; n]))))
            }
            NodeData::ArrayLit => {
                let ch = self.ast.children(id).to_vec();
                let mut elems = Vec::with_capacity(ch.len());
                for &e in &ch {
                    elems.push(self.eval(e)?);
                }
                Ok(VVal::SA(Rc::new(RefCell::new(elems))))
            }
            // this / super / 方法引用 / lambda / Raw / 赋值表达式（AST 不产）
            _ => Err(()),
        }
    }

    fn eval_ref_value(&mut self, id: JavaId) -> R<VVal> {
        // 数组**引用**读（Index/Member 的 base 必须是 VarRef 指向 CA/SA）
        if self.ast.kind(id) != NodeKind::VarRef {
            return Err(());
        }
        let k = self.key(id)?;
        match self.vars.get(&k).ok_or(())? {
            VVal::CA(_) | VVal::SA(_) => Ok(self.vars.get(&k).unwrap().clone()),
            _ => Err(()),
        }
    }

    fn eval_call(&mut self, ch: &[JavaId]) -> R<VVal> {
        // 白名单：S.toCharArray() / S.length() / S.charAt(i) / S.intern() /
        // S.isEmpty() / S.equals(S) / String.valueOf(...)
        let (callee, args) = ch.split_first().ok_or(())?;
        match self.ast.data(*callee).clone() {
            NodeData::Member { name } => {
                let recv_id = *self.ast.children(*callee).first().ok_or(())?;
                // new String(x).intern()：驻留结果与字面量同实例——直接
                // 展开求值（绕过裸 New 的身份拒绝）。bd/ferns 解密模式的
                // `b[i] = new String(chars).intern()` 走此路径
                if self.ast.sn(name) == "intern" {
                    if let NodeData::New { ty, anon_raw } = self.ast.data(recv_id).clone() {
                        if anon_raw.is_none() && matches!(ty, JType::Ref(r) if r == "String") {
                            let nch = self.ast.children(recv_id).to_vec();
                            // 元数守卫：仅单参构造（String/String→String、
                            // char[]→String）。(char[],int,int)/(byte[],…)
                            // 重载有 offset/count 语义——折成整组数组即值变
                            //（第 12 轮攻击抓获：new String(ca,1,3).intern()
                            // 曾折成 ca 全量）
                            if nch.len() != 1 {
                                return Err(());
                            }
                            let arg = self.eval(*nch.first().ok_or(())?)?;
                            return match arg {
                                VVal::CA(a) => {
                                    let n = a.borrow().len();
                                    self.str_budget(n)?;
                                    Ok(VVal::S(a.borrow().iter().collect()))
                                }
                                VVal::S(s) => Ok(VVal::S(s)),
                                _ => Err(()),
                            };
                        }
                        return Err(());
                    }
                }
                let recv = self.eval(recv_id)?;
                match (self.ast.sn(name), recv) {
                    ("toCharArray", VVal::S(s)) => {
                        self.str_budget_chars(s.chars().count())?;
                        Ok(VVal::CA(Rc::new(RefCell::new(s.chars().collect()))))
                    }
                    ("length", VVal::S(s)) => Ok(VVal::I(s.chars().count() as i32)),
                    ("charAt", VVal::S(s)) => {
                        let iv = self.eval(args[0])?.as_i32().ok_or(())?;
                        let i = usize::try_from(iv).map_err(|_| ())?;
                        Ok(VVal::I(s.chars().nth(i).ok_or(())? as u32 as i32))
                    }
                    ("intern", VVal::S(s)) => Ok(VVal::S(s)),
                    ("isEmpty", VVal::S(s)) => Ok(VVal::B(s.is_empty())),
                    ("equals", VVal::S(a)) => {
                        if args.len() != 1 {
                            return Err(());
                        }
                        let b = self.eval(args[0])?;
                        match b {
                            VVal::S(b) => Ok(VVal::B(a == b)),
                            _ => Err(()),
                        }
                    }
                    _ => Err(()),
                }
            }
            NodeData::VarRef { name } => {
                // 静态调用：String.valueOf(...)
                let n = self.ast.sn(name);
                if n == "valueOf" {
                    let v = self.eval(args[0])?;
                    match v {
                        VVal::I(x) => Ok(VVal::S(x.to_string())),
                        VVal::L(x) => Ok(VVal::S(x.to_string())),
                        VVal::S(s) => Ok(VVal::S(s)),
                        VVal::CA(a) => Ok(VVal::S(a.borrow().iter().collect())),
                        _ => Err(()),
                    }
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }

    fn of_lit(&mut self, l: Lit) -> R<VVal> {
        match l {
            Lit::Bool(b) => Ok(VVal::B(b)),
            Lit::Int(v) => Ok(VVal::I(v as i32)),
            Lit::Long(v) => Ok(VVal::L(v)),
            Lit::Char(c) => Ok(VVal::I(c as u32 as i32)),
            Lit::Str(s) => {
                self.str_budget_chars(s.chars().count())?;
                Ok(VVal::S(s))
            }
            Lit::NumRaw { val, .. } => match val {
                NumVal::Int(v) => Ok(VVal::I(v as i32)),
                NumVal::Long(v) => Ok(VVal::L(v)),
                _ => Err(()),
            },
            _ => Err(()), // 浮点/TextBlock/Null
        }
    }

    fn str_budget(&self, n: usize) -> R<()> {
        if n > MAX_STR_LEN {
            Err(())
        } else {
            Ok(())
        }
    }
    fn str_budget_chars(&self, n: usize) -> R<()> {
        self.str_budget(n)
    }

    fn binop(&mut self, op: BinOp, a: &VVal, b: &VVal) -> R<VVal> {
        // i32 域（int/char/byte/short 混合）；long 域当任一为 L
        use BinOp::*;
        let wide = matches!(a, VVal::L(_)) || matches!(b, VVal::L(_));
        if op.is_comparison() {
            let ord = match (a, b) {
                (VVal::I(x), VVal::I(y)) => (*x).cmp(y),
                (VVal::L(x), VVal::L(y)) => (*x).cmp(y),
                (VVal::I(x), VVal::L(y)) => (*x as i64).cmp(y),
                (VVal::L(x), VVal::I(y)) => (*x).cmp(&(*y as i64)),
                _ => return Err(()),
            };
            return Ok(VVal::B(match op {
                Lt => ord.is_lt(),
                Le => ord.is_le(),
                Gt => ord.is_gt(),
                Ge => ord.is_ge(),
                Eq => ord.is_eq(),
                Ne => ord.is_ne(),
                _ => return Err(()),
            }));
        }
        if matches!(op, Eq | Ne) {
            // 布尔/字符串相等
            return match (a, b) {
                (VVal::B(x), VVal::B(y)) => Ok(VVal::B(if op == Eq { x == y } else { x != y })),
                (VVal::S(x), VVal::S(y)) => Ok(VVal::B(if op == Eq { x == y } else { x != y })),
                _ => Err(()),
            };
        }
        let (ai, bi): (i64, i64) = match (a, b) {
            (VVal::I(x), VVal::I(y)) => (*x as i64, *y as i64),
            (VVal::L(x), VVal::L(y)) => (*x, *y),
            (VVal::I(x), VVal::L(y)) => (*x as i64, *y),
            (VVal::L(x), VVal::I(y)) => (*x, *y as i64),
            _ => return Err(()),
        };
        let r: i64 = match op {
            Add => ai.wrapping_add(bi),
            Sub => ai.wrapping_sub(bi),
            Mul => ai.wrapping_mul(bi),
            Div => {
                if bi == 0 {
                    return Err(());
                }
                ai.wrapping_div(bi)
            }
            Rem => {
                if bi == 0 {
                    return Err(());
                }
                ai.wrapping_rem(bi)
            }
            Shl => ai.wrapping_shl((bi as u64 & if wide { 63 } else { 31 }) as u32),
            Shr => ai.wrapping_shr((bi as u64 & if wide { 63 } else { 31 }) as u32),
            UShr => ((ai as u64).wrapping_shr((bi as u64 & if wide { 63 } else { 31 }) as u32))
                as i64,
            BitAnd => ai & bi,
            BitOr => ai | bi,
            BitXor => ai ^ bi,
            _ => return Err(()),
        };
        Ok(if wide { VVal::L(r) } else { VVal::I(r as i32) })
    }

    fn cast(&mut self, v: VVal, ty: &JType) -> R<VVal> {
        match (v, ty) {
            (VVal::I(x), JType::Char) => Ok(VVal::I(x as u16 as i32)),
            (VVal::I(x), JType::Byte) => Ok(VVal::I(x as i8 as i32)),
            (VVal::I(x), JType::Short) => Ok(VVal::I(x as i16 as i32)),
            (VVal::I(x), JType::Int) => Ok(VVal::I(x)),
            (VVal::I(x), JType::Long) => Ok(VVal::L(x as i64)),
            (VVal::L(x), JType::Int) => Ok(VVal::I(x as i32)),
            (VVal::L(x), JType::Char) => Ok(VVal::I(x as u32 as u16 as i32)),
            (VVal::L(x), JType::Byte) => Ok(VVal::I(x as i8 as i32)),
            (VVal::L(x), JType::Short) => Ok(VVal::I(x as i16 as i32)),
            (VVal::L(x), JType::Long) => Ok(VVal::L(x)),
            _ => Err(()),
        }
    }

    // ---- 结果提取 ----

    /// 段内声明的局部名字键集合。

    pub(crate) fn var_value(&self, k: u32) -> Option<&VVal> {
        self.vars.get(&k)
    }

    pub(crate) fn field_writes(&self) -> &[(u32, VVal)] {
        &self.field_writes
    }

    /// 局部声明序（首次声明顺序）。
    pub(crate) fn decl_order(&self) -> &[u32] {
        &self.decl_order
    }
    pub(crate) fn effect_log(&self) -> &[EffectEvent] {
        &self.effect_log
    }
    /// 局部声明域（var_width：2=char/3=byte/4=short/5=long/0=宽或未知）。
    /// 逃逸材料化按域恢复声明类型与字面量种类。
    pub(crate) fn var_decl_ty(&self, k: u32) -> Option<JType> {
        self.decl_tys.get(&k).cloned()
    }
    pub(crate) fn var_domain(&self, k: u32) -> u8 {
        self.var_width.get(&k).copied().unwrap_or(0)
    }

    // ---- Class.forName 假设 + 深快照 + 前缀遍历（子块截断）----

    /// 语句位置的 `Class.forName(<常量串>)`：类存在性可判定且为 JDK
    /// 自带 → 假设成功（不抛）。类初始化副作用保留：以 effect_log 记录，
    /// 材料化时重放为「原语句 + 字面量实参」。仅值被丢弃的语句位置；
    /// 表达式位置的返回值不可表示 → Err（保守 abort）。
    /// P1 屏障检测：`target = <Call>(实参...)`——target 是**裸名**（未
    /// 声明 = 字段）、call 是未知调用（eval_call 会 Err 的形态）、全部
    /// 实参可求值（Literal 直接过 / 其余 eval 成功）。返回事件供日志
    /// 重放；字段值记 Undef（不透明——参与运算即 abort，见 eval 的
    /// opaque_fields 检查）。**表达式位置**的未知调用仍 abort（只有
    /// 语句位置的整句赋值才穿越——副作用边界清晰）。
    #[allow(clippy::too_many_arguments)]
    fn try_opaque_field_barrier(
        &mut self,
        stmt: JavaId,
        target: JavaId,
        value: JavaId,
    ) -> R<Option<EffectEvent>> {
        // target 必须是裸名（未声明 → 字段写语境）
        if self.ast.kind(target) != NodeKind::VarRef {
            return Ok(None);
        }
        // try 语境拒绝（P1g/P1h/P1l 攻击抓获）：屏障重放 = 假设调用成功
        // 的裸赋值——原语句的异常窗口（catch 改道字段写/finally 副作用）
        // 会被材料化丢弃（输入 caught|3 → 输出 EIIE）。返回 None = 无
        // 屏障 → 赋值求值失败 → 语句保守失败 → try 留 rest 原样保真
        if self.try_depth > 0 {
            return Ok(None);
        }
        let k = match self.key(target) {
            Ok(k) => k,
            Err(()) => return Ok(None),
        };
        // 已声明局部 → 普通赋值路径
        if self.scopes.iter().any(|sc| sc.contains(&k)) {
            return Ok(None);
        }
        // value 必须是 Call（未知调用——否则走普通路径）
        if self.ast.kind(value) != NodeKind::Call {
            return Ok(None);
        }
        // Class.forName 已有专门路径
        let callee = match self.ast.children(value).first().copied() {
            Some(c) => c,
            None => return Ok(None),
        };
        if let NodeData::Member { name } = self.ast.data(callee) {
            if self.ast.sn(*name) == "forName" {
                return Ok(None);
            }
        }
        // 先试整条求值——成功的是**已知调用**（new String(...).intern()
        // 等 String 白名单），走普通赋值路径（BR1 抓获：b 曾被误判屏障
        // 导致材料化丢失）
        if self.eval(value).is_ok() {
            return Ok(None);
        }
        // 全部实参求值（Literal 直接过；失败 → 不是屏障——保守 abort）
        let ch = self.ast.children(value).to_vec();
        if ch.is_empty() {
            return Ok(None);
        }
        let mut args: Vec<Option<VVal>> = Vec::new();
        for &arg in &ch[1..] {
            if self.ast.kind(arg) == NodeKind::Literal {
                // 字面量实参：保留 None——重放时 copy 原子树（精确原文）
                args.push(None);
                continue;
            }
            match self.eval(arg) {
                Ok(v) => args.push(Some(v)),
                Err(()) => return Ok(None), // 不可求值实参 → 拒绝穿越
            }
        }
        // 屏障确认：字段值 Undef（不透明）+ 字段写记账
        if std::env::var("CURE_DBG_P1").is_ok() {
            let nm = self.ast.name_of_key(k).unwrap_or_default();
            eprintln!("[P1] barrier fired: field={:?} args={:?}", nm, args.len());
        }
        self.opaque_fields.insert(k);
        // 直接记账不进 effect_log（record_field_write_by_key 会 push
        // FieldWrite 事件——材料化循环先遇 FW(Undef) → materialize(Undef)
        // → None 提前退出整个重写。OFW 事件自身承载重放）
        if let Some(slot) = self.field_writes.iter_mut().find(|(n, _)| *n == k) {
            slot.1 = VVal::Undef;
        } else {
            self.field_writes.push((k, VVal::Undef));
        }
        Ok(Some(EffectEvent::OpaqueFieldWrite { k, stmt, call: value, args }))
    }

    fn try_class_forname(&mut self, stmt: JavaId, call: JavaId) -> R<bool> {
        if self.ast.kind(call) != NodeKind::Call {
            return Ok(false);
        }
        let ch = self.ast.children(call).to_vec();
        let Some((callee, args)) = ch.split_first() else {
            return Ok(false);
        };
        if self.ast.kind(*callee) != NodeKind::Member {
            return Ok(false);
        }
        let mname = match self.ast.data(*callee) {
            NodeData::Member { name } => self.ast.sn(*name),
            _ => return Ok(false),
        };
        if mname != "forName" || args.len() != 1 {
            return Ok(false);
        }
        // 接收方链尾必须是 Class（Class.forName / java.lang.Class.forName）
        let recv = self.ast.children(*callee).first().copied();
        let recv_tail = match recv.map(|r| self.ast.data(r)) {
            Some(NodeData::VarRef { name }) => Some(self.ast.sn(*name)),
            Some(NodeData::Member { name }) => Some(self.ast.sn(*name)),
            _ => None,
        };
        if recv_tail != Some("Class") {
            return Ok(false);
        }
        let name = match self.eval(args[0])? {
            VVal::S(s) => s,
            _ => return Err(()), // 非常量类名 → 不可判定 → abort
        };
        if !crate::jdk::class_loads(&name) {
            return Err(()); // 可判定不存在 / 未知类 → 保守 abort
        }
        self.effect_log.push(EffectEvent::ClassLoad { stmt, call, name });
        Ok(true)
    }

    /// 深快照：vars/field_writes/decl_order/scopes/var_width/effect_log
    /// 全量克隆，数组内容深拷贝（别名保持——var10004 = var10003 的共享
    /// 在快照/恢复后仍共享）。steps/exhausted 不回滚（预算语义）。
    /// 用途：子块遍历中「失败语句的部分效应」撤销（数组内容污染不可由
    /// 浅回滚恢复）。
    fn deep_snapshot(&self) -> Snap {
        let mut seen: HashMap<usize, VVal> = HashMap::new();
        Snap {
            vars: self
                .vars
                .iter()
                .map(|(k, v)| (*k, snap_value(v, &mut seen)))
                .collect(),
            field_writes: self
                .field_writes
                .iter()
                .map(|(k, v)| (*k, snap_value(v, &mut seen)))
                .collect(),
            decl_order: self.decl_order.clone(),
            scopes: self.scopes.clone(),
            var_width: self.var_width.clone(),
            effect_log: self.effect_log.clone(),
            opaque_fields: self.opaque_fields.clone(),
            try_depth: self.try_depth,
            decl_tys: self.decl_tys.clone(),
        }
    }
    fn deep_restore(&mut self, s: Snap) {
        let mut seen: HashMap<usize, VVal> = HashMap::new();
        self.vars = s
            .vars
            .into_iter()
            .map(|(k, v)| (k, snap_value(&v, &mut seen)))
            .collect();
        self.field_writes = s
            .field_writes
            .into_iter()
            .map(|(k, v)| (k, snap_value(&v, &mut seen)))
            .collect();
        self.decl_order = s.decl_order;
        self.scopes = s.scopes;
        self.var_width = s.var_width;
        self.effect_log = s.effect_log;
        self.opaque_fields = s.opaque_fields;
        self.try_depth = s.try_depth;
        self.decl_tys = s.decl_tys;
    }

    /// 前缀遍历（子块截断核心）：逐语句执行，遇失败语句时若它是
    /// Block/Label/Try 则**下降进它的孩子**继续（不整条执行），语句级
    /// 失败前深快照、失败后回滚——遍历后的状态只含已完成语句的效应
    /// （无「半途倾倒」）。返回各级截断层与真实剩余语句。
    /// cuts 顺序：最深在前、根最后（根始终记录；嵌套层仅在部分完成时
    /// 记录——完整完成的嵌套层效应归父层区间，避免双发）。
    pub(crate) fn run_prefix(&mut self, stmts: &[JavaId]) -> R<PrefixRun> {
        let mut run = PrefixRun {
            cuts: Vec::new(),
            rest_stack: Vec::new(),
            early_exit: None,
            pending_break: None,
        };
        self.run_level(stmts, None, true, &mut run, None, true)?;
        // 根层未消费的带标签 break：有效 Java 的 label 必然在根内被
        // Label 层捕获——未消费 = 病态/外溢 → 整段拒绝（保守）
        if run.pending_break.is_some() {
            return Err(());
        }
        Ok(run)
    }

    /// 单层遍历。根调用（is_root）不压作用域（外层 scopes[0] 即根域）；
    /// 嵌套层压域、完整完成时弹层（同 exec_stmt(Block) 语义），部分完成
    /// 停止时不弹（遍历即终止，状态供逃逸分析）。
    fn run_level(
        &mut self,
        stmts: &[JavaId],
        block: Option<JavaId>,
        is_root: bool,
        run: &mut PrefixRun,
        exit_label: Option<String>,
        allow_pending: bool,
    ) -> R<()> {
        let log_start = self.effect_log.len();
        let decl_start = self.decl_order.len();
        let scopes_at_entry = self.scopes.len();
        if !is_root {
            self.scopes.push(Vec::new());
        }
        let mut completed = 0usize;
        let mut stopped_at: Option<usize> = None;
        // 普通失败（非下降有进展）：rest 必须含失败语句本身（旧语义——
        // 逃逸/再赋值分析需要看见它：CFF 的 switch(s) 状态机引用 s）
        let mut include_failed_stmt = false;
        // 失败语句前的日志位置（失败语句的效应已回滚/归内层）
        let mut stop_log_end = self.effect_log.len();
        'lvl: for (i, &st) in stmts.iter().enumerate() {
            stop_log_end = self.effect_log.len();
            let snap = self.deep_snapshot();
            let cuts_len = run.cuts.len();
            let rest_len = run.rest_stack.len();
            let scopes_len = self.scopes.len();
            let decl_len = self.decl_order.len();
            if let Some(inner) = self.descend_target(st) {
                // 下降层的标签语境：Label(L) → 本层可消费 break L；
                // Block/Group 继承外层标签；Try 体层 sealed（break 到达
                // 即停入 rest——finally 是 Try 的兄弟孩子，跳层会跳过
                // finally 执行，语义破坏）
                let (el, ap, in_try) = match self.ast.data(st).clone() {
                    NodeData::Label { name } => (Some(name), true, false),
                    NodeData::Try => (None, false, true),
                    _ => (exit_label.clone(), true, false),
                };
                let inner_stmts = self.ast.children(inner).to_vec();
                // Try 体下降层进 try 语境（体内屏障拒绝——异常窗口保守）
                self.try_depth += in_try as usize;
                let r = self.run_level(&inner_stmts, Some(inner), false, run, el, ap);
                self.try_depth -= in_try as usize;
                // 子层带标签 break 上抛：命中本层标签 → 消费；否则保留
                // pending 继续上抛。两种情形本层剩余均跳过（break 语义）
                if run.pending_break.is_some() {
                    if !allow_pending {
                        // sealed 层（try 体）：按失败停——try 语句留 rest，
                        // finally 语义保守
                        run.pending_break = None;
                        self.deep_restore(snap);
                        run.cuts.truncate(cuts_len);
                        run.rest_stack.truncate(rest_len);
                        self.truncate_state(decl_len, scopes_len);
                        stopped_at = Some(i);
                        include_failed_stmt = true;
                        break 'lvl;
                    }
                    let l = run.pending_break.clone().unwrap();
                    if exit_label.as_deref() == Some(l.as_str()) {
                        run.pending_break = None; // 消费
                    }
                    completed = i + 1;
                    break 'lvl; // 无 rest 无 cut——父层 splice 覆盖
                }
                let inner_stopped = run.rest_stack.len() > rest_len;
                match (r, inner_stopped) {
                    (Ok(()), false) => {
                        // Try：体块完整完成后 finally 仍须执行（效应入
                        // 日志随材料化重放）——否则整条 try 替换时 finally
                        // 副作用丢失（t03 抓获：f="fin" 消失、println 整条吞）
                        if let NodeData::Try = self.ast.data(st) {
                            if let Some(fin) = self.try_finally_block(st) {
                                self.try_depth += 1;
                                let fflow = self.exec_stmt(fin);
                                self.try_depth -= 1;
                                match fflow {
                                    Ok(Flow::Normal) => {}
                                    _ => {
                                        // finally 失败 → 整条 try 按失败处理
                                        self.deep_restore(snap);
                                        run.cuts.truncate(cuts_len);
                                        run.rest_stack.truncate(rest_len);
                                        self.truncate_state(decl_len, scopes_len);
                                        stopped_at = Some(i);
                                        include_failed_stmt = true;
                                        break 'lvl;
                                    }
                                }
                            }
                        }
                        // 整条语句完成：效应已在日志中，语句可被材料化替换
                        completed = i + 1;
                    }
                    (Ok(()), true) | (Err(()), _) => {
                        // 内层停止/失败于 m：
                        //   m > 0 → 内层 cut 已记录，本语句留在本层 rest
                        //   m == 0 → 撤销下降，整条语句按失败处理
                        let inner_completed =
                            run.cuts.get(cuts_len).map(|c| c.completed).unwrap_or(0);
                        if inner_completed == 0 {
                            run.cuts.truncate(cuts_len);
                            run.rest_stack.truncate(rest_len);
                            self.truncate_state(decl_len, scopes_len);
                            self.deep_restore(snap);
                            include_failed_stmt = true;
                        } else if matches!(self.ast.data(st), NodeData::Try) {
                            // Try 部分完成：catch/finally 仍是可执行剩余——
                            // 追加进最内层 rest（逃逸/再赋值扫描可见）。
                            // t06b 抓获：catch 引用体块前缀声明的局部 s，
                            // 漏扫 → s 不材料化 → 输出不可编译
                            let ch = self.ast.children(st).to_vec();
                            let extra: Vec<JavaId> = ch
                                .iter()
                                .copied()
                                .skip(1)
                                .filter(|&c| {
                                    matches!(self.ast.kind(c), NodeKind::Catch | NodeKind::Block)
                                })
                                .collect();
                            if let Some(last) = run.rest_stack.last_mut() {
                                last.extend(extra);
                            } else {
                                run.rest_stack.push(extra);
                            }
                        }
                        stopped_at = Some(i);
                        break 'lvl;
                    }
                }
            } else {
                match self.exec_stmt(st) {
                    Ok(Flow::Normal) => {
                        completed = i + 1;
                    }
                    // 带标签 break 到达语句层：本层标签命中 → 消费跳过剩余；
                    // 否则置 pending 上抛（层间传播）。sealed 层（try 体内）
                    // 落入 Ok(other) 按失败保守处理
                    Ok(Flow::Break(Some(ref l))) if allow_pending => {
                        if exit_label.as_deref() != Some(l.as_str()) {
                            run.pending_break = Some(l.clone());
                        }
                        completed = i + 1;
                        break 'lvl; // 本层剩余跳过（无 rest——break 语义）
                    }
                    Ok(other) => {
                        if is_root && i + 1 == stmts.len() {
                            // 末尾 return/break 逃逸：现有根层语义
                            run.early_exit = Some(other);
                            completed = i + 1;
                        } else {
                            self.deep_restore(snap);
                            stopped_at = Some(i);
                            include_failed_stmt = true;
                            break 'lvl;
                        }
                    }
                    Err(()) => {
                        self.deep_restore(snap);
                        stopped_at = Some(i);
                        include_failed_stmt = true;
                        if std::env::var("CURE_DBG_P1").is_ok() {
                            eprintln!(
                                "[P1-rl] root-level stop at stmt {} kind={:?}",
                                i,
                                self.ast.kind(st)
                            );
                        }
                        break 'lvl;
                    }
                }
            }
        }
        let log_end = match stopped_at {
            Some(_) => stop_log_end,
            None => self.effect_log.len(),
        };
        if let Some(c) = stopped_at {
            // 本层剩余：普通失败 → 含失败语句本身（旧语义）；下降有进展
            // → 失败语句的内部剩余已由更深层压栈，这里只压后续兄弟
            if include_failed_stmt {
                let mut lvl = vec![stmts[c]];
                lvl.extend_from_slice(&stmts[c + 1..]);
                run.rest_stack.push(lvl);
            } else {
                run.rest_stack.push(stmts[c + 1..].to_vec());
            }
        } else if !is_root {
            // 嵌套层完整完成：弹域清值（后续语句看不到其局部）
            for k in self.scopes.pop().unwrap() {
                self.vars.remove(&k);
                self.var_width.remove(&k);
            }
        }
        if is_root || stopped_at.is_some() {
            run.cuts.push(LevelCut {
                block_hint: block, // 根层 None → static_exec 回填根块 id
                completed,
                log_start,
                log_end,
                decl_start,
            });
        }
        let _ = scopes_at_entry;
        Ok(())
    }

    /// 下降撤销：弹出下降期间压入的作用域层并清值。
    fn truncate_state(&mut self, decl_len: usize, scopes_len: usize) {
        while self.scopes.len() > scopes_len {
            for k in self.scopes.pop().unwrap() {
                self.vars.remove(&k);
                self.var_width.remove(&k);
            }
        }
        self.decl_order.truncate(decl_len);
    }

    /// 失败语句的可下降目标：Label→子块 / Block(含 Group) / Try→体块。
    fn descend_target(&self, st: JavaId) -> Option<JavaId> {
        match self.ast.data(st) {
            NodeData::Label { .. } => {
                let child = self.ast.children(st).first().copied()?;
                match self.ast.kind(child) {
                    NodeKind::Block => Some(child),
                    _ => None,
                }
            }
            NodeData::Block | NodeData::Group => Some(st),
            NodeData::Try => {
                // try-with-resources 资源头（非 Block/Catch 的头部孩子）
                // —— close 语义不可模拟（exec_try 同一合同）。下降路径曾
                // 静默跳过资源头把整条 try 连 close() 副作用一起删掉
                //（边界攻击 t02 抓获：closed=1 → 0）
                let ch = self.ast.children(st).to_vec();
                let first = ch.first().copied()?;
                if self.ast.kind(first) != NodeKind::Block {
                    return None;
                }
                let mut i = 1usize;
                let mut finally_block: Option<JavaId> = None;
                while i < ch.len() {
                    match self.ast.kind(ch[i]) {
                        NodeKind::Catch => {}
                        NodeKind::Block => finally_block = Some(ch[i]),
                        _ => return None,
                    }
                    i += 1;
                }
                // first = 体块
                if finally_block.is_some() {
                    // 带 finally：体块外的兄弟（finally）不能整体替换——
                    // 下降会把 finally 随体块的完成一起丢掉（t03 抓获：
                    // f="fin" 消失）。返回体块但 run_level 的 Try 分支
                    // 负责 finally 的执行；这里标记需要 finally 处理：
                    // 用元组形态（体块, finally）——改为调用方处理
                }
                Some(first)
            }
            _ => None,
        }
    }

    /// Try 语句的 finally 体块（catch 之后的 Block；无 → None）。
    fn try_finally_block(&self, st: JavaId) -> Option<JavaId> {
        let ch = self.ast.children(st).to_vec();
        if ch.is_empty() || self.ast.kind(ch[0]) != NodeKind::Block {
            return None;
        }
        let mut finally = None;
        for &c in &ch[1..] {
            match self.ast.kind(c) {
                NodeKind::Catch => {}
                NodeKind::Block => finally = Some(c),
                _ => return None,
            }
        }
        finally
    }
}

/// 深快照载荷。
struct Snap {
    vars: HashMap<u32, VVal>,
    field_writes: Vec<(u32, VVal)>,
    decl_order: Vec<u32>,
    scopes: Vec<Vec<u32>>,
    var_width: HashMap<u32, u8>,
    effect_log: Vec<EffectEvent>,
    opaque_fields: std::collections::HashSet<u32>,
    try_depth: usize,
    decl_tys: HashMap<u32, JType>,
}

/// 深拷贝一个值（别名保持：seen 以 Rc 地址识别同一数组）。
fn snap_value(v: &VVal, seen: &mut HashMap<usize, VVal>) -> VVal {
    match v {
        VVal::CA(a) => {
            let addr = Rc::as_ptr(a) as usize;
            if let Some(c) = seen.get(&addr) {
                return c.clone();
            }
            let c = VVal::CA(Rc::new(RefCell::new(a.borrow().clone())));
            seen.insert(addr, c.clone());
            c
        }
        VVal::SA(a) => {
            let addr = Rc::as_ptr(a) as usize;
            if let Some(c) = seen.get(&addr) {
                return c.clone();
            }
            let elems: Vec<VVal> = a
                .borrow()
                .iter()
                .map(|e| snap_value(e, seen))
                .collect();
            let c = VVal::SA(Rc::new(RefCell::new(elems)));
            seen.insert(addr, c.clone());
            c
        }
        other => other.clone(),
    }
}

/// 前缀遍历结果。
pub(crate) struct PrefixRun {
    /// 截断层（**最深在前、根最后**；根始终记录，嵌套层仅部分完成时记录）。
    /// block_hint：嵌套层 = splice 目标块 id；根层 = None（static_exec
    /// 以根块 id 回填）。
    pub cuts: Vec<LevelCut>,
    /// 各停止层的剩余兄弟语句（最深在前，长度 = 停止层数 = 有 cut 的
    /// 嵌套层数）。
    pub rest_stack: Vec<Vec<JavaId>>,
    /// 根层末尾 return/break 逃逸（现有语义）。
    pub early_exit: Option<Flow>,
    /// 层间带标签 break 传播：语句/子层产生 `break <l>` 且 l 非本层标签
    /// 时置入——本层剩余跳过（无 rest 无 cut，父层覆盖效应），父层继续
    /// 消费/上抛。根层未消费（病态——有效 Java 的 label 必在根内）→
    /// run_prefix 整体拒绝。
    pub pending_break: Option<String>,
}

impl PrefixRun {
    /// 真实剩余语句全集（各层拼接——守卫用）。
    pub fn true_rest(&self) -> Vec<JavaId> {
        let mut all = Vec::new();
        for lvl in &self.rest_stack {
            all.extend_from_slice(lvl);
        }
        all
    }
    /// 第 j 个 cut（0=最深 … len-1=根）的剩余：
    /// rest_stack[0..=j]（该层失败语句内部及以下的全部剩余）。
    pub fn rest_for_cut(&self, j: usize) -> Vec<JavaId> {
        let mut all = Vec::new();
        for lvl in self.rest_stack.iter().take(j + 1) {
            all.extend_from_slice(lvl);
        }
        all
    }
    /// 根 cut 的索引。
    pub fn root_cut_index(&self) -> usize {
        self.cuts.len() - 1
    }
}

/// 单个截断层。
pub(crate) struct LevelCut {
    /// splice 目标块（None = 根层占位，static_exec 回填根块 id）。
    pub block_hint: Option<JavaId>,
    /// 该层已完成的孩子数。
    pub completed: usize,
    /// 该层拥有的日志区间 [start, end)。
    pub log_start: usize,
    pub log_end: usize,
    /// 该层进入时的 decl_order 长度（逃逸局部范围）。
    pub decl_start: usize,
}

fn narrow(x: i32, w: u8) -> i32 {
    match w {
        2 => x as u16 as i32,
        3 => x as i8 as i32,
        4 => x as i16 as i32,
        _ => x,
    }
}
