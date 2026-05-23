#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::os::windows::process::CommandExt;
use std::process::Command;

// Windows 原生 MessageBox API
#[cfg(windows)]
mod win {
    #[link(name = "user32")]
    extern "system" {
        pub fn MessageBoxW(hwnd: usize, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn ShellExecuteW(
            hwnd: usize,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> isize;
    }

    pub const MB_OK: u32 = 0x0000_0000;
    pub const MB_YESNO: u32 = 0x0000_0004;
    pub const MB_ICONQUESTION: u32 = 0x0000_0020;
    pub const MB_ICONERROR: u32 = 0x0000_0010;
    pub const MB_ICONWARNING: u32 = 0x0000_0030;
    pub const IDYES: i32 = 6;

    /// 弹出原生 Windows 对话框，返回按钮 ID。
    pub fn msgbox(title: &str, text: &str, flags: u32) -> i32 {
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { MessageBoxW(0, text_wide.as_ptr(), title_wide.as_ptr(), flags) }
    }
}

/// 检测系统是否已安装 WebView2 Runtime。
fn is_webview2_installed() -> bool {
    let keys = [
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEF-ED47AE3B01D8}",
        r"HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEF-ED47AE3B01D8}",
    ];

    for key in keys {
        let output = Command::new("reg")
            .args(["query", key, "/v", "pv"])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW 隐藏 cmd 窗口
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("pv")) {
                    let val = line.split_whitespace().last().unwrap_or("");
                    if !val.is_empty() && val != "0.0.0.0" {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// 下载并安装 WebView2 Runtime。
fn install_webview2() -> bool {
    let answer = win::msgbox(
        "系统工具箱",
        "系统缺少 WebView2 运行时组件，是否自动下载安装？\n（需联网，约 2MB，安装后即可正常使用）\n\n点击\"是\"后请在弹出的安装窗口中允许安装。",
        win::MB_YESNO | win::MB_ICONQUESTION,
    );

    if answer != win::IDYES {
        return false;
    }

    let temp_dir = std::env::temp_dir();
    let bootstrapper_path = temp_dir.join("MicrosoftEdgeWebview2Setup.exe");

    // 用 PowerShell 下载（隐藏窗口）
    let download = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile '{}'",
                bootstrapper_path.display()
            ),
        ])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output();

    if download.is_err() || !bootstrapper_path.exists() {
        win::msgbox(
            "系统工具箱",
            "下载失败，请检查网络连接后重试。",
            win::MB_OK | win::MB_ICONWARNING,
        );
        return false;
    }

    // 检查下载的文件大小（防止下载了空文件或错误页面）
    let file_size = std::fs::metadata(&bootstrapper_path)
        .map(|m| m.len())
        .unwrap_or(0);
    if file_size < 1024 {
        let _ = std::fs::remove_file(&bootstrapper_path);
        win::msgbox(
            "系统工具箱",
            "下载文件异常，请检查网络连接后重试。",
            win::MB_OK | win::MB_ICONWARNING,
        );
        return false;
    }

    // 以管理员权限运行安装程序（会弹出 UAC 提示）
    let path_wide: Vec<u16> = bootstrapper_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let params: Vec<u16> = "/install\0".encode_utf16().collect();

    let result = unsafe {
        win::ShellExecuteW(
            0,
            verb.as_ptr(),
            path_wide.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };

    // ShellExecuteW 返回值 > 32 表示成功启动
    if result as usize <= 32 {
        let _ = std::fs::remove_file(&bootstrapper_path);
        win::msgbox(
            "系统工具箱",
            "启动安装程序失败，请尝试手动安装 WebView2。",
            win::MB_OK | win::MB_ICONWARNING,
        );
        return false;
    }

    // 等待安装完成（最多等 60 秒）
    win::msgbox(
        "系统工具箱",
        "WebView2 正在安装，安装完成后请点击\"确定\"继续。",
        win::MB_OK | win::MB_ICONQUESTION,
    );

    let _ = std::fs::remove_file(&bootstrapper_path);
    true
}

fn main() {
    if !is_webview2_installed() {
        if !install_webview2() {
            win::msgbox(
                "系统工具箱",
                "未安装 WebView2，程序无法运行。\n\n请手动访问以下地址下载安装：\nhttps://developer.microsoft.com/en-us/microsoft-edge/webview2/",
                win::MB_OK | win::MB_ICONERROR,
            );
            return;
        }

        // 安装后再次检测
        if !is_webview2_installed() {
            win::msgbox(
                "系统工具箱",
                "WebView2 安装似乎未成功，请重启电脑后重试。",
                win::MB_OK | win::MB_ICONWARNING,
            );
            return;
        }
    }

    system_toolbox_tauri_lib::run()
}
