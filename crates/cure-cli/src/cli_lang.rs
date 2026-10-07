//! CLI 双语消息语言检测（ddc 同款协议）。
//!
//! 解析序：`CURE_LANG`（显式 `zh`/`en` 覆盖）> `LC_ALL` > `LC_MESSAGES`
//! > `LANG` > `LANGUAGE` > Windows 用户 UI 语言。主子标签以 `zh` 开头
//! （zh / zh_CN / zh-Hans / zh_TW.UTF-8 …）选中文；声明了其他语言选英文。
//! `C`、`POSIX`、空值视为未声明语言，链继续。`LANGUAGE` 是冒号分隔的
//! 优先列表（`zh:en`）——只取第一项。
//!
//! 环境变量永远优先——Win32 调用仅在没有任何变量声明语言时执行
//! （裸 cmd.exe / PowerShell 不导出 locale 变量；Git Bash / Cygwin / WSL
//! 导出 `LANG`，已由链覆盖）。

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliLang {
    En,
    Zh,
}

impl CliLang {
    pub(crate) fn detect() -> CliLang {
        static DETECTED: OnceLock<CliLang> = OnceLock::new();
        *DETECTED.get_or_init(|| {
            if let Ok(v) = std::env::var("CURE_LANG") {
                // 显式覆盖——但仅识别合法值：CURE_LANG=fr 之类打错的不钉死
                // 语言，落回 locale 链
                if let Some(p) = primary_tag(&v) {
                    if p.starts_with("zh") {
                        return CliLang::Zh;
                    }
                    if p.starts_with("en") {
                        return CliLang::En;
                    }
                }
            }
            for var in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
                let Ok(v) = std::env::var(var) else { continue };
                // LANGUAGE（"zh:en"）是优先列表：第一项定案，其余是
                // 不兑现的回退
                let first = v.split(':').find(|s| !s.is_empty()).unwrap_or("");
                if let Some(p) = primary_tag(first) {
                    return if p.starts_with("zh") { CliLang::Zh } else { CliLang::En };
                }
            }
            // 环境里没有任何语言声明。裸 cmd/PowerShell 永远落到这里——
            // 回退 Windows 用户 UI 语言（Windows 自己显示的那个）。
            #[cfg(windows)]
            {
                if let Some(l) = windows_ui_lang() {
                    return l;
                }
            }
            CliLang::En
        })
    }
}

/// Windows 用户 UI 语言（kernel32）。只区分中文；其他 UI 语言保持英文
/// 默认。`GetUserDefaultUILanguage` 返回 LANGID，低 10 位是主语言——
/// `0x04` 覆盖全部中文变体（zh-CN/zh-TW/zh-HK/…）。直接 extern 声明
/// （kernel32 总在 MSVC 链接里），保持零运行时依赖。
#[cfg(windows)]
fn windows_ui_lang() -> Option<CliLang> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    let langid = unsafe { GetUserDefaultUILanguage() };
    (langid & 0x3ff == 0x04).then_some(CliLang::Zh)
}

/// locale 标签的主语言子标签（小写）：`zh_CN.UTF-8` → `zh`、
/// `zh-Hans` → `zh`、`en_US` → `en`。字符集（`.UTF-8`）与修饰
/// （`@euro`）剔除；`C` / `POSIX` / 空串返回 None（未声明语言）。
fn primary_tag(tag: &str) -> Option<String> {
    let base = tag.split('.').next().unwrap_or("");
    let base = base.split('@').next().unwrap_or("");
    let p = base.split(&['-', '_'][..]).next().unwrap_or("");
    if p.is_empty() || p == "C" || p == "POSIX" {
        return None;
    }
    Some(p.to_ascii_lowercase())
}
