//! 跨平台批量源文件读取后端。
//!
//! 平台策略（由构建目标 + feature 自动选择，调用方无 cfg）：
//!
//! | 平台 | 后端 | 理由 |
//! |------|------|------|
//! | Linux（`--features io-uring`） | 每线程一个 io_uring 实例 | 37 万文件 ≈ 110 万次 open/read/close 系统调用；三阶段批量压到每 64 文件 ~6 次。冷缓存时内核预读可与 CPU 处理重叠 |
//! | Linux 默认构建 | std 逐文件 | 保持零运行时依赖（io-uring crate 为可选依赖，默认不启用） |
//! | macOS | std 逐文件（线程池聚合） | macOS 无 io_uring；POSIX AIO 不支持 open 异步化、并发上限 AIO_LISTIO_MAX，对本负载无收益。实测（M 系，页缓存热）：16/32 线程聚合 ~0.9–1.1 GB/s，已接近逐文件 syscall I/O 的实际上限；预分配容量 / fs::read 变体均在噪声内 |
//! | Windows | std 逐文件（线程池聚合） | OVERLAPPED 批量读需 windows crate（破坏默认零依赖）；如需启用可按 io-uring 同一 feature 模式补充 `windows-io` 后端 |
//!
//! 接口统一为「读一组路径 → 按序返回 `io::Result<String>`」：
//! 文件级错误（权限/编码/…）以 `Err` 值返回（调用方告警跳过，与
//! `fs::read_to_string` 行为一致）；仅 ring 机制级故障整组回退 std，
//! 行为永不劣于默认后端。

use std::fs;
use std::io;
use std::path::Path;

/// 每批文件数 = 工作线程取号粒度（main.rs 批量 fetch_add(64)）。
pub(crate) const BATCH: usize = 64;

/// 每线程持有一个的批量读取器。非 io_uring 构建下是零状态占位
/// （read_batch 直接退化为逐文件 `fs::read_to_string`）。
pub(crate) struct BatchFileReader {
    #[cfg(all(target_os = "linux", feature = "io-uring"))]
    ring: Option<ring_backend::RingReader>,
}

impl BatchFileReader {
    pub(crate) fn new() -> Self {
        #[cfg(all(target_os = "linux", feature = "io-uring"))]
        {
            // ring 初始化失败（老内核无 io_uring、ulimit、内存压力）→
            // 静默回退 std 路径
            Self { ring: ring_backend::RingReader::new(BATCH).ok() }
        }
        #[cfg(not(all(target_os = "linux", feature = "io-uring")))]
        {
            Self {}
        }
    }

    /// 读取一组文件（内部自动按 ring 深度分块）。
    /// 返回与 `paths` 顺序一一对应的读取结果。
    pub(crate) fn read_batch(&mut self, paths: &[&Path]) -> Vec<io::Result<String>> {
        #[cfg(all(target_os = "linux", feature = "io-uring"))]
        if let Some(r) = self.ring.as_mut() {
            return r.read_batch(paths);
        }
        paths.iter().map(|p| fs::read_to_string(p)).collect()
    }
}

// ---------------------------------------------------------------------------
// Linux io_uring 后端（feature = "io-uring"）
// ---------------------------------------------------------------------------

#[cfg(all(target_os = "linux", feature = "io-uring"))]
mod ring_backend {
    use io_uring::{opcode, types, IoUring};
    use std::ffi::CString;
    use std::io;
    use std::os::fd::{FromRawFd, OwnedFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    /// `AT_FDCWD`（-100）：相对当前工作目录解析路径。
    const AT_FDCWD: i32 = -100;
    /// 初始读缓冲：Java 语料中位大小 ~4KB，64KB 覆盖 >99%；超限按需倍增。
    const INITIAL_CAP: usize = 64 * 1024;

    struct Slot {
        /// 打开成功的描述符；-1 = open 失败或已关闭。
        fd: RawFd,
        /// 本文件的 IO/编码错误（以 Err 值透传给调用方）。
        err: Option<io::Error>,
        buf: Vec<u8>,
        /// 已确认读到的字节数。
        len: usize,
        done: bool,
    }

    impl Slot {
        fn fresh() -> Self {
            Self { fd: -1, err: None, buf: Vec::with_capacity(INITIAL_CAP), len: 0, done: false }
        }

        /// 描述符所有权转移给 OwnedFd（drop 即 close）。
        fn take_fd(&mut self) -> Option<OwnedFd> {
            if self.fd >= 0 {
                let owned = unsafe { OwnedFd::from_raw_fd(self.fd) };
                self.fd = -1;
                Some(owned)
            } else {
                None
            }
        }
    }

    pub(crate) struct RingReader {
        ring: IoUring,
        depth: usize,
        slots: Vec<Slot>,
        /// ring 一旦发生机制级故障即永久弃用（在途 SQE 可能在内核侧继续
        /// 执行：其引用的路径/缓冲通过 pending_paths 与常驻 slots 保持映射，
        /// CQE 任其堆积不再收割）。此后所有读取走 std 回退。
        broken: bool,
        /// 机制级故障时承接在途 open 的路径缓冲（防 use-after-free）。
        pending_paths: Vec<CString>,
    }

    impl RingReader {
        pub(crate) fn new(depth: usize) -> io::Result<Self> {
            let depth = depth.max(1);
            let ring = IoUring::new(depth as u32)?;
            let slots = (0..depth).map(|_| Slot::fresh()).collect();
            Ok(Self { ring, depth, slots, broken: false, pending_paths: Vec::new() })
        }

        /// 机制级错误（submission/cqueue 异常）→ 整组回退 std；
        /// 文件级错误以 `Ok(vec![Err(e)])` 返回。
        pub(crate) fn read_batch(&mut self, paths: &[&Path]) -> Vec<io::Result<String>> {
            if self.broken {
                return paths.iter().map(|p| std::fs::read_to_string(p)).collect();
            }
            match self.try_read_batch(paths) {
                Ok(v) => v,
                Err(_) => paths.iter().map(|p| std::fs::read_to_string(p)).collect(),
            }
        }

        fn try_read_batch(&mut self, paths: &[&Path]) -> io::Result<Vec<io::Result<String>>> {
            let mut out: Vec<io::Result<String>> = Vec::with_capacity(paths.len());
            for chunk in paths.chunks(self.depth) {
                out.extend(self.read_chunk(chunk)?);
            }
            Ok(out)
        }

        /// 三阶段批量：A) open 全部 → B) read 全部（满缓冲 = 可能未读完 →
        /// 倍增重读）→ C) close 全部。每阶段一次 submit_and_wait。
        ///
        /// SQE 携带的用户指针（路径 CString、读缓冲）必须存活到对应 CQE
        /// 收割完毕：路径存活期为整个函数（故障时转入 pending_paths）；
        /// 缓冲仅在无在途 SQE 引用时（need_grow 的 reserve、装配 take）改变。
        ///
        /// 机制级故障不提前 return：先安全回收 fd（在途 read 持有内核侧
        /// struct file 引用，close 不影响其完成；close SQE 未提交则 fd 归
        /// 我们），再统一回退。
        fn read_chunk(&mut self, paths: &[&Path]) -> io::Result<Vec<io::Result<String>>> {
            let n = paths.len();
            let mut mach_err: Option<io::Error> = None;

            // 路径转 CString；含 NUL 的路径 → 文件级错误（不阻塞同批其他文件）
            let mut cpaths: Vec<Option<CString>> = Vec::with_capacity(n);
            for p in paths {
                cpaths.push(CString::new(p.as_os_str().as_bytes()).ok());
            }

            // 复用槽位（缓冲容量跨批保留）；重置每文件状态
            if self.slots.len() < n {
                self.slots.extend((self.slots.len()..n).map(|_| Slot::fresh()));
            }
            for i in 0..n {
                let s = &mut self.slots[i];
                s.fd = -1;
                s.len = 0;
                s.done = cpaths[i].is_none();
                s.err = if s.done {
                    Some(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "文件名包含非法 NUL 字节",
                    ))
                } else {
                    None
                };
                if s.buf.capacity() < INITIAL_CAP {
                    s.buf = Vec::with_capacity(INITIAL_CAP);
                }
                s.buf.clear();
            }

            // ---- 阶段 A：批量 open ----
            let mut submitted = 0usize;
            {
                let RingReader { ring, .. } = self;
                let mut sq = ring.submission();
                for (i, c) in cpaths.iter().enumerate() {
                    let Some(c) = c else { continue };
                    let e = opcode::OpenAt::new(types::Fd(AT_FDCWD), c.as_ptr())
                        .build()
                        .user_data(i as u64);
                    if unsafe { sq.push(&e) }.is_err() {
                        mach_err = Some(io::Error::other("io_uring SQ 满"));
                        break;
                    }
                    submitted += 1;
                }
            }
            if submitted > 0 {
                match self.ring.submit_and_wait(submitted) {
                    Err(e) => mach_err = Some(e),
                    Ok(_) => {
                        for c in self.ring.completion() {
                            let s = &mut self.slots[c.user_data() as usize];
                            if c.result() < 0 {
                                s.err = Some(io::Error::from_raw_os_error(-c.result()));
                                s.done = true;
                            } else {
                                s.fd = c.result();
                            }
                        }
                    }
                }
            }

            // ---- 阶段 B：批量 read ----
            while mach_err.is_none() {
                let mut inflight = 0usize;
                {
                    let RingReader { ring, slots, .. } = self;
                    let mut sq = ring.submission();
                    for (i, s) in slots.iter_mut().enumerate().take(n) {
                        if s.done || s.fd < 0 {
                            continue;
                        }
                        let cap = s.buf.capacity().max(INITIAL_CAP);
                        if s.buf.len() < cap {
                            s.buf.reserve(cap - s.buf.len());
                            // u8 无非法位模式：set_len 至容量后作为读缓冲可靠
                            unsafe { s.buf.set_len(cap) };
                        }
                        // buf[0..len] 为已读前缀；本次读写入尾区 [len..cap)
                        // ——addr 必须前移，否则覆盖已读数据
                        let e = opcode::Read::new(
                            types::Fd(s.fd),
                            unsafe { s.buf.as_mut_ptr().add(s.len) },
                            (s.buf.len() - s.len) as u32,
                        )
                        .offset(s.len as u64)
                        .build()
                        .user_data(i as u64);
                        if unsafe { sq.push(&e) }.is_err() {
                            mach_err = Some(io::Error::other("io_uring SQ 满"));
                            break;
                        }
                        inflight += 1;
                    }
                }
                if inflight == 0 || mach_err.is_some() {
                    break;
                }
                if let Err(e) = self.ring.submit_and_wait(inflight) {
                    mach_err = Some(e);
                    break;
                }
                let mut need_grow: Vec<usize> = Vec::new();
                for c in self.ring.completion() {
                    let i = c.user_data() as usize;
                    let res = c.result();
                    let s = &mut self.slots[i];
                    if res < 0 {
                        s.err = Some(io::Error::from_raw_os_error(-res));
                        s.done = true;
                    } else if s.len + res as usize == s.buf.len() {
                        // 读满整个缓冲：文件可能更大 → 倍增后继续读
                        s.len += res as usize;
                        need_grow.push(i);
                    } else {
                        s.len += res as usize;
                        s.buf.truncate(s.len);
                        s.done = true;
                    }
                }
                for i in need_grow {
                    let s = &mut self.slots[i];
                    let new_cap = s.buf.capacity() * 2;
                    s.buf.reserve(new_cap - s.buf.len());
                }
            }

            // ---- 阶段 C：批量 close ----
            if mach_err.is_none() {
                let mut closing = 0usize;
                let mut push_failed = false;
                {
                    let RingReader { ring, slots, .. } = self;
                    let mut sq = ring.submission();
                    for s in slots.iter_mut().take(n) {
                        if s.fd >= 0 {
                            let e = opcode::Close::new(types::Fd(s.fd)).build().user_data(0);
                            if unsafe { sq.push(&e) }.is_err() {
                                push_failed = true;
                                break;
                            }
                            closing += 1;
                        }
                    }
                }
                if push_failed {
                    // SQE 未提交：内核不会执行，fd 仍归我们所有
                    mach_err = Some(io::Error::other("io_uring SQ 满"));
                } else if closing > 0 {
                    match self.ring.submit_and_wait(closing) {
                        Ok(_) => {
                            for _ in self.ring.completion() {}
                            for s in self.slots.iter_mut().take(n) {
                                s.fd = -1; // 内核已关闭
                            }
                        }
                        Err(e) => {
                            // close SQE 可能已在途（enter 于提交后失败）：
                            // fd 交给内核；不再触碰（有限泄漏优于对可能
                            // 已复用的 fd 双重关闭）。路径缓冲就地保留。
                            self.mark_broken(&mut cpaths, &e);
                            return Err(e);
                        }
                    }
                }
            }

            if mach_err.is_some() {
                let e = mach_err.take().unwrap();
                self.mark_broken(&mut cpaths, &e);
                // close SQE 未提交（或 push 中断）：安全回收全部 fd
                for s in self.slots.iter_mut().take(n) {
                    drop(s.take_fd());
                }
                return Err(e);
            }

            // ---- 装配结果：String::from_utf8(Vec) 校验并零拷贝转 String ----
            let mut results = Vec::with_capacity(n);
            for i in 0..n {
                let s = &mut self.slots[i];
                if let Some(e) = s.err.take() {
                    s.buf.clear();
                    results.push(Err(e));
                    continue;
                }
                debug_assert!(s.done);
                let buf = std::mem::take(&mut s.buf);
                s.len = 0;
                s.done = false;
                results.push(match String::from_utf8(buf) {
                    Ok(text) => Ok(text),
                    Err(_) => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    )),
                });
            }
            Ok(results)
        }

        /// 机制级故障善后：永久弃用 ring，并承接在途 open 的路径缓冲。
        fn mark_broken(&mut self, cpaths: &mut Vec<Option<CString>>, _e: &io::Error) {
            self.broken = true;
            self.pending_paths
                .extend(cpaths.drain(..).flatten());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn tmp_files(count: usize) -> (tempfile::TempDir, Vec<std::path::PathBuf>) {
            let dir = tempfile::tempdir().unwrap();
            let mut paths = Vec::new();
            for i in 0..count {
                let p = dir.path().join(format!("f{i}.java"));
                std::fs::write(&p, format!("class A{i} {{}}\n")).unwrap();
                paths.push(p);
            }
            (dir, paths)
        }

        #[test]
        fn ring_matches_std_read() {
            let (dir, paths) = tmp_files(100);
            let refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
            let mut r = RingReader::new(16).unwrap();
            for chunk in refs.chunks(7) {
                let got = r.read_batch(chunk);
                for (g, p) in got.iter().zip(chunk) {
                    assert_eq!(g.as_ref().unwrap(), &std::fs::read_to_string(p).unwrap());
                }
            }
            drop(dir);
        }

        #[test]
        fn ring_errors_align_with_paths() {
            let (dir, paths) = tmp_files(4);
            let mut mixed: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
            mixed.push(Path::new("/nonexistent-definitely-missing/xx.java"));
            let mut r = RingReader::new(16).unwrap();
            let got = r.read_batch(&mixed);
            assert_eq!(got.len(), mixed.len());
            for (g, p) in got.iter().zip(&mixed) {
                let std_res = std::fs::read_to_string(p);
                assert_eq!(g.is_ok(), std_res.is_ok(), "path={:?}", p);
            }
            drop(dir);
        }

        #[test]
        fn ring_oversize_buffer_growth() {
            // 200KB 文件：64K → 128K → 256K 倍增链
            let dir = tempfile::tempdir().unwrap();
            let p = dir.path().join("big.java");
            let body: String = "class Big { int x; }\n".repeat(11_000);
            std::fs::write(&p, &body).unwrap();
            let mut r = RingReader::new(16).unwrap();
            let got = r.read_batch(&[p.as_path()]);
            assert_eq!(got[0].as_ref().unwrap(), &body);
        }

        #[test]
        fn ring_empty_and_non_utf8() {
            let dir = tempfile::tempdir().unwrap();
            let empty = dir.path().join("empty.java");
            std::fs::write(&empty, b"").unwrap();
            let latin = dir.path().join("latin.java");
            std::fs::write(&latin, b"class A { /* \xff\xfe */ }").unwrap();
            let mut r = RingReader::new(16).unwrap();
            let got = r.read_batch(&[empty.as_path(), latin.as_path()]);
            assert_eq!(got[0].as_ref().unwrap(), "");
            assert!(got[1].is_err());
        }

        #[test]
        fn ring_exact_buffer_multiple() {
            // 恰好 64KB（= INITIAL_CAP）：读满触发倍增，第二轮 EOF 收敛
            let dir = tempfile::tempdir().unwrap();
            let p = dir.path().join("exact.java");
            let body = vec![b'a'; INITIAL_CAP];
            std::fs::write(&p, &body).unwrap();
            let mut r = RingReader::new(16).unwrap();
            let got = r.read_batch(&[p.as_path()]);
            assert_eq!(got[0].as_ref().unwrap().as_bytes(), &body[..]);
        }
    }
}
