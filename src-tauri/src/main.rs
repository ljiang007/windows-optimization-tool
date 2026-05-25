#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
const APP_STARTUP_PASSWORD: &str = "wanzi";
const TODESK_LITE_BYTES: &[u8] = include_bytes!("../resources/tools/todesk/ToDesk_Lite.exe");

// Windows 原生 API
#[cfg(windows)]
mod win {
    use std::ptr;

    #[repr(C)]
    pub struct ShellExecuteInfoW {
        pub cb_size: u32,
        pub f_mask: u32,
        pub hwnd: usize,
        pub verb: *const u16,
        pub file: *const u16,
        pub parameters: *const u16,
        pub directory: *const u16,
        pub show: i32,
        pub inst_app: usize,
        pub id_list: *mut (),
        pub class: *const u16,
        pub hkey_class: usize,
        pub hot_key: u32,
        pub h_icon_or_monitor: usize,
        pub h_process: usize,
    }

    pub const SEE_MASK_NOCLOSEPROCESS: u32 = 0x0000_0040;

    #[link(name = "user32")]
    extern "system" {
        pub fn MessageBoxW(hwnd: usize, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn ShellExecuteExW(info: *mut ShellExecuteInfoW) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn WaitForSingleObject(handle: usize, milliseconds: u32) -> u32;
        pub fn CloseHandle(handle: usize) -> i32;
    }

    pub const MB_OK: u32 = 0x0000_0000;
    pub const MB_YESNO: u32 = 0x0000_0004;
    pub const MB_ICONQUESTION: u32 = 0x0000_0020;
    pub const MB_ICONERROR: u32 = 0x0000_0010;
    pub const MB_ICONWARNING: u32 = 0x0000_0030;
    pub const IDYES: i32 = 6;
    pub const INFINITE: u32 = 0xFFFF_FFFF;

    /// 弹出原生 Windows 对话框，返回按钮 ID。
    pub fn msgbox(title: &str, text: &str, flags: u32) -> i32 {
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { MessageBoxW(0, text_wide.as_ptr(), title_wide.as_ptr(), flags) }
    }

    /// 以管理员权限运行程序并等待其结束。返回是否成功启动并等待完毕。
    pub fn run_as_admin_and_wait(exe_path: &str, params: &str) -> bool {
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();
        let file: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
        let parameters: Vec<u16> = params.encode_utf16().chain(std::iter::once(0)).collect();

        let mut info = ShellExecuteInfoW {
            cb_size: std::mem::size_of::<ShellExecuteInfoW>() as u32,
            f_mask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: 0,
            verb: verb.as_ptr(),
            file: file.as_ptr(),
            parameters: parameters.as_ptr(),
            directory: ptr::null(),
            show: 1, // SW_SHOWNORMAL
            inst_app: 0,
            id_list: ptr::null_mut(),
            class: ptr::null(),
            hkey_class: 0,
            hot_key: 0,
            h_icon_or_monitor: 0,
            h_process: 0,
        };

        let success = unsafe { ShellExecuteExW(&mut info) };
        if success == 0 {
            return false;
        }

        if info.h_process != 0 {
            // 等待安装进程完全结束（最多等 120 秒）
            unsafe {
                WaitForSingleObject(info.h_process, 120_000);
                CloseHandle(info.h_process);
            }
        }

        true
    }
}

/// 检测系统是否已安装 WebView2 Runtime。
fn is_webview2_installed() -> bool {
    // 方法1：检查注册表（多个可能的位置）
    let keys = [
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEF-ED47AE3B01D8}",
        r"HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEF-ED47AE3B01D8}",
        r"HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEF-ED47AE3B01D8}",
    ];

    for key in keys {
        let output = Command::new("reg")
            .args(["query", key, "/v", "pv"])
            .creation_flags(0x0800_0000)
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

    // 方法2：直接检查 WebView2 文件是否存在
    let paths_to_check = [
        r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application",
        r"C:\Program Files\Microsoft\EdgeWebView\Application",
    ];
    for path in paths_to_check {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }

    // 方法3：检查用户目录下的 WebView2
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let user_path = format!(r"{}\Microsoft\EdgeWebView\Application", local_app_data);
        if std::path::Path::new(&user_path).exists() {
            return true;
        }
    }

    // 方法4：检查 Edge 浏览器是否存在（Edge 94+ 自带 WebView2）
    let edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for path in edge_paths {
        if std::path::Path::new(path).exists() {
            return true;
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

    // 以管理员权限运行安装程序并等待结束（会弹出 UAC 提示）
    let path_str = bootstrapper_path.to_string_lossy().to_string();
    let launched = win::run_as_admin_and_wait(&path_str, "/install");

    let _ = std::fs::remove_file(&bootstrapper_path);

    if !launched {
        win::msgbox(
            "系统工具箱",
            "启动安装程序失败，请尝试手动安装 WebView2。",
            win::MB_OK | win::MB_ICONWARNING,
        );
        return false;
    }

    // 安装进程已结束，等待几秒让注册表写入生效
    std::thread::sleep(std::time::Duration::from_secs(3));
    true
}

fn ps_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn verify_startup_password() -> bool {
    let title = ps_single_quote("系统工具箱");
    let prompt = ps_single_quote("请输入启动密码：");
    let wrong_password = ps_single_quote("密码错误，请重新输入。");
    let expected_password = ps_single_quote(APP_STARTUP_PASSWORD);

    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$expectedPassword = '{expected_password}'

while ($true) {{
    $form = New-Object System.Windows.Forms.Form
    $form.Text = '{title}'
    $form.StartPosition = 'CenterScreen'
    $form.Size = New-Object System.Drawing.Size(360, 170)
    $form.FormBorderStyle = 'FixedDialog'
    $form.MaximizeBox = $false
    $form.MinimizeBox = $false
    $form.TopMost = $true

    $label = New-Object System.Windows.Forms.Label
    $label.Text = '{prompt}'
    $label.Location = New-Object System.Drawing.Point(20, 20)
    $label.Size = New-Object System.Drawing.Size(300, 20)

    $textbox = New-Object System.Windows.Forms.TextBox
    $textbox.Location = New-Object System.Drawing.Point(20, 50)
    $textbox.Size = New-Object System.Drawing.Size(300, 23)
    $textbox.UseSystemPasswordChar = $true

    $okButton = New-Object System.Windows.Forms.Button
    $okButton.Text = '确定'
    $okButton.Location = New-Object System.Drawing.Point(164, 90)
    $okButton.Size = New-Object System.Drawing.Size(75, 28)
    $okButton.DialogResult = [System.Windows.Forms.DialogResult]::OK

    $cancelButton = New-Object System.Windows.Forms.Button
    $cancelButton.Text = '取消'
    $cancelButton.Location = New-Object System.Drawing.Point(245, 90)
    $cancelButton.Size = New-Object System.Drawing.Size(75, 28)
    $cancelButton.DialogResult = [System.Windows.Forms.DialogResult]::Cancel

    $form.Controls.Add($label)
    $form.Controls.Add($textbox)
    $form.Controls.Add($okButton)
    $form.Controls.Add($cancelButton)
    $form.AcceptButton = $okButton
    $form.CancelButton = $cancelButton

    $result = $form.ShowDialog()
    if ($result -ne [System.Windows.Forms.DialogResult]::OK) {{
        exit 1
    }}

    if ($textbox.Text -eq $expectedPassword) {{
        exit 0
    }}

    [System.Windows.Forms.MessageBox]::Show(
        '{wrong_password}',
        '{title}',
        [System.Windows.Forms.MessageBoxButtons]::OK,
        [System.Windows.Forms.MessageBoxIcon]::Warning
    ) | Out-Null
}}
"#
    );

    match Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .creation_flags(0x0800_0000)
        .status()
    {
        Ok(status) => status.success(),
        Err(err) => {
            win::msgbox(
                "系统工具箱",
                &format!("启动密码验证失败：{err}"),
                win::MB_OK | win::MB_ICONERROR,
            );
            false
        }
    }
}

fn extract_startup_tool(filename: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("system_toolbox_startup_tools");
    fs::create_dir_all(&dir).map_err(|e| format!("创建启动工具目录失败：{e}"))?;

    let dest = dir.join(filename);
    let need_write = match fs::metadata(&dest) {
        Ok(meta) => meta.len() != bytes.len() as u64,
        Err(_) => true,
    };

    if need_write {
        fs::write(&dest, bytes).map_err(|e| format!("释放启动工具失败：{e}"))?;
    }

    Ok(dest)
}

fn prompt_and_open_todesk() {
    let answer = win::msgbox(
        "系统工具箱",
        "是否打开 ToDesk 受控远程协助？",
        win::MB_YESNO | win::MB_ICONQUESTION,
    );

    if answer != win::IDYES {
        return;
    }

    let tool_path = match extract_startup_tool("ToDesk_Lite.exe", TODESK_LITE_BYTES) {
        Ok(path) => path,
        Err(err) => {
            win::msgbox(
                "系统工具箱",
                &format!("ToDesk 准备失败：{err}"),
                win::MB_OK | win::MB_ICONWARNING,
            );
            return;
        }
    };

    let tool_dir = match tool_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => std::env::temp_dir(),
    };

    if let Err(err) = Command::new(&tool_path)
        .current_dir(tool_dir)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
    {
        win::msgbox(
            "系统工具箱",
            &format!("打开 ToDesk 失败：{err}"),
            win::MB_OK | win::MB_ICONWARNING,
        );
    }
}

fn main() {
    if !verify_startup_password() {
        return;
    }

    if !is_webview2_installed() {
        if !install_webview2() {
            win::msgbox(
                "系统工具箱",
                "未安装 WebView2，程序无法运行。\n\n请手动访问以下地址下载安装：\nhttps://developer.microsoft.com/en-us/microsoft-edge/webview2/",
                win::MB_OK | win::MB_ICONERROR,
            );
            return;
        }
        // 安装进程已正常结束，直接启动应用
    }

    prompt_and_open_todesk();

    system_toolbox_tauri_lib::run()
}
