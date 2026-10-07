//! JDK 类存在性判断——`Class.forName` 假设成功的依据。
//!
//! 两级策略：
//! 1. `--jdk-home` / `CURE_JDK_HOME` 指定 JDK：读 `lib/src.zip` 做字节
//!    检索（zip 文件名在中央目录中未压缩存储；`.java` 后缀天然锚定，
//!    无前缀误命中）——精确存在性。
//! 2. 未指定：java.base 模块包前缀白名单（JLS 强制每个 Java SE 平台
//!    包含 java.base——这些类在任何合规 JVM 上 forName 必然成功）。
//!
//! 语义合同：假设仅覆盖**控制流**（forName 不抛）；forName 语句本身
//! 作为不透明副作用保留在产物中（类初始化可观察）。

use std::sync::OnceLock;

/// java.base 模块的包集合（JLS 强制存在）。
const JAVA_BASE_PKGS: &[&str] = &[
    "java.io",
    "java.lang",
    "java.lang.annotation",
    "java.lang.constant",
    "java.lang.invoke",
    "java.lang.module",
    "java.lang.ref",
    "java.lang.reflect",
    "java.math",
    "java.net",
    "java.net.spi",
    "java.nio",
    "java.nio.channels",
    "java.nio.channels.spi",
    "java.nio.charset",
    "java.nio.charset.spi",
    "java.nio.file",
    "java.nio.file.attribute",
    "java.nio.file.spi",
    "java.security",
    "java.security.cert",
    "java.security.interfaces",
    "java.security.spec",
    "java.text",
    "java.text.spi",
    "java.time",
    "java.time.chrono",
    "java.time.format",
    "java.time.temporal",
    "java.time.zone",
    "java.util",
    "java.util.concurrent",
    "java.util.concurrent.atomic",
    "java.util.concurrent.locks",
    "java.util.function",
    "java.util.jar",
    "java.util.logging",
    "java.util.prefs",
    "java.util.random",
    "java.util.regex",
    "java.util.spi",
    "java.util.stream",
    "java.util.zip",
    "javax.crypto",
    "javax.crypto.interfaces",
    "javax.crypto.spec",
    "javax.net",
    "javax.net.ssl",
    "javax.security.auth",
    "javax.security.auth.callback",
    "javax.security.auth.login",
    "javax.security.auth.spi",
    "javax.security.auth.x500",
    "javax.security.cert",
];

fn java_base_prefix(name: &str) -> bool {
    // 嵌套类（java.util.Map$Entry）以外部类判断
    let base = name.split('$').next().unwrap_or(name);
    let base = base.trim();
    JAVA_BASE_PKGS
        .iter()
        .any(|p| base == *p || base.starts_with(p) && base.as_bytes().get(p.len()) == Some(&b'.'))
}

/// src.zip 全量字节（~54MB，仅在 --jdk-home 时加载一次）。
static JDK_SRC: OnceLock<Option<std::sync::Arc<Vec<u8>>>> = OnceLock::new();

/// 主线程（CLI）在并行 worker 启动前调用。也支持 CURE_JDK_HOME 环境变量。
pub fn set_jdk_home(path: &std::path::Path) {
    let zip = path.join("lib").join("src.zip");
    if let Ok(data) = std::fs::read(&zip) {
        let _ = JDK_SRC.set(Some(std::sync::Arc::new(data)));
    }
}

fn jdk_src() -> Option<std::sync::Arc<Vec<u8>>> {
    if let Some(v) = JDK_SRC.get() {
        return v.clone();
    }
    if let Ok(p) = std::env::var("CURE_JDK_HOME") {
        set_jdk_home(std::path::Path::new(&p));
    }
    JDK_SRC.get().cloned().flatten()
}

/// `Class.forName(name)` 是否可假设成功。
/// 未知/可判定不存在 → false（保守 abort）。
pub fn class_loads(name: &str) -> bool {
    if let Some(src) = jdk_src() {
        // 精确存在性："java/security/MessageDigest.java"
        // （.java 后缀防止前缀误命中：Col.java 不是 Collection.java 的子串）
        let slash: String = name.replace('.', "/");
        let needle = format!("{}.java", slash);
        return find_sub(&src, needle.as_bytes());
    }
    java_base_prefix(name)
}

fn find_sub(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    let n = needle.len();
    let first = needle[0];
    let mut i = 0usize;
    let last = hay.len() - n;
    while i <= last {
        if hay[i] == first && &hay[i..i + n] == needle {
            return true;
        }
        i += 1;
    }
    false
}
