use std::fs;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

struct XianyuQrState {
    qr_b64: Mutex<Option<String>>,
}

#[derive(serde::Serialize)]
struct PriceResult {
    taobao: Option<f64>,
    douyin: Option<f64>,
    xianyu: Option<f64>,
    xianyu_url: Option<String>,
}

#[derive(serde::Serialize)]
struct XianyuSessionInfo {
    logged_in: bool,
    username: Option<String>,
    profile: Option<serde_json::Value>,
}

#[derive(Default)]
struct XianyuFetchResult {
    price: Option<f64>,
    url: Option<String>,
}

const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

struct CompanionFile {
    filename: &'static str,
    bytes: &'static [u8],
}

macro_rules! companion {
    ($filename:literal, $path:literal) => {
        CompanionFile {
            filename: $filename,
            bytes: include_bytes!($path),
        }
    };
}

struct BundledTool {
    key: &'static str,
    label: &'static str,
    filename: &'static str,
    bytes: &'static [u8],
    companions: &'static [CompanionFile],
    wait_for_exit: bool,
}

const BUNDLED_TOOLS: &[BundledTool] = &[
    BundledTool {
        key: "windowsActivation",
        label: "Windows激活",
        filename: "windowsActivation.exe",
        bytes: include_bytes!("../resources/tools/windows-activation/windowsActivation.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "windowsUpdateSettings",
        label: "Windows更新设置",
        filename: "Wub_x64.exe",
        bytes: include_bytes!("../resources/tools/windows-update-settings/Wub_x64.exe"),
        companions: &[
            CompanionFile {
                filename: "Wub.ini",
                bytes: include_bytes!("../resources/tools/windows-update-settings/Wub.ini"),
            },
        ],
        wait_for_exit: true,
    },
    BundledTool {
        key: "defenderSwitch",
        label: "Defender开关",
        filename: "DefenderRemover_v13.0_Chs.exe",
        bytes: include_bytes!("../resources/tools/defender-switch/DefenderRemover_v13.0_Chs.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "softwareUninstall",
        label: "软件卸载",
        filename: "geek.exe",
        bytes: include_bytes!("../resources/tools/software-uninstall/geek.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "installWinrar",
        label: "安装WinRAR",
        filename: "winrar.exe",
        bytes: include_bytes!("../resources/tools/winrar/winrar.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "killWormVirus",
        label: "杀蠕虫病毒",
        filename: "杀蠕虫病毒.exe",
        bytes: include_bytes!("../resources/tools/kill-worm-virus/杀蠕虫病毒.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "microsoftCommonRuntimeLibraries",
        label: "微软常用运行库",
        filename: "微软常用运行库.exe",
        bytes: include_bytes!(
            "../resources/tools/microsoft-common-runtime-libraries/微软常用运行库.exe"
        ),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "fixDx11",
        label: "修复DX11",
        filename: "DirectX Repair.exe",
        bytes: include_bytes!("../resources/tools/fix-dx/DirectX Repair.exe"),
        companions: &[
            companion!(
                "DirectX Repair.exe.config",
                "../resources/tools/fix-dx/DirectX Repair.exe.config"
            ),
            companion!("log.dat", "../resources/tools/fix-dx/log.dat"),
            companion!("Settings.ini", "../resources/tools/fix-dx/Settings.ini"),
            companion!("Data/A/x3daudio1_0.dll", "../resources/tools/fix-dx/Data/A/x3daudio1_0.dll"),
            companion!("Data/A/x3daudio1_1.dll", "../resources/tools/fix-dx/Data/A/x3daudio1_1.dll"),
            companion!("Data/A/x3daudio1_2.dll", "../resources/tools/fix-dx/Data/A/x3daudio1_2.dll"),
            companion!("Data/A/X3DAudio1_3.dll", "../resources/tools/fix-dx/Data/A/X3DAudio1_3.dll"),
            companion!("Data/A/X3DAudio1_4.dll", "../resources/tools/fix-dx/Data/A/X3DAudio1_4.dll"),
            companion!("Data/A/X3DAudio1_5.dll", "../resources/tools/fix-dx/Data/A/X3DAudio1_5.dll"),
            companion!("Data/A/X3DAudio1_6.dll", "../resources/tools/fix-dx/Data/A/X3DAudio1_6.dll"),
            companion!("Data/A/X3DAudio1_7.dll", "../resources/tools/fix-dx/Data/A/X3DAudio1_7.dll"),
            companion!("Data/A/xactengine2_0.dll", "../resources/tools/fix-dx/Data/A/xactengine2_0.dll"),
            companion!("Data/A/xactengine2_1.dll", "../resources/tools/fix-dx/Data/A/xactengine2_1.dll"),
            companion!("Data/A/xactengine2_2.dll", "../resources/tools/fix-dx/Data/A/xactengine2_2.dll"),
            companion!("Data/A/xactengine2_3.dll", "../resources/tools/fix-dx/Data/A/xactengine2_3.dll"),
            companion!("Data/A/xactengine2_7.dll", "../resources/tools/fix-dx/Data/A/xactengine2_7.dll"),
            companion!("Data/A/xinput1_1.dll", "../resources/tools/fix-dx/Data/A/xinput1_1.dll"),
            companion!("Data/A/xinput1_2.dll", "../resources/tools/fix-dx/Data/A/xinput1_2.dll"),
            companion!("Data/A/xinput1_3.dll", "../resources/tools/fix-dx/Data/A/xinput1_3.dll"),
            companion!("Data/A/xinput9_1_0.dll", "../resources/tools/fix-dx/Data/A/xinput9_1_0.dll"),
            companion!("Data/B/x3daudio1_0.dll", "../resources/tools/fix-dx/Data/B/x3daudio1_0.dll"),
            companion!("Data/B/x3daudio1_1.dll", "../resources/tools/fix-dx/Data/B/x3daudio1_1.dll"),
            companion!("Data/B/x3daudio1_2.dll", "../resources/tools/fix-dx/Data/B/x3daudio1_2.dll"),
            companion!("Data/B/X3DAudio1_3.dll", "../resources/tools/fix-dx/Data/B/X3DAudio1_3.dll"),
            companion!("Data/B/X3DAudio1_4.dll", "../resources/tools/fix-dx/Data/B/X3DAudio1_4.dll"),
            companion!("Data/B/X3DAudio1_5.dll", "../resources/tools/fix-dx/Data/B/X3DAudio1_5.dll"),
            companion!("Data/B/X3DAudio1_6.dll", "../resources/tools/fix-dx/Data/B/X3DAudio1_6.dll"),
            companion!("Data/B/X3DAudio1_7.dll", "../resources/tools/fix-dx/Data/B/X3DAudio1_7.dll"),
            companion!("Data/B/xactengine2_0.dll", "../resources/tools/fix-dx/Data/B/xactengine2_0.dll"),
            companion!("Data/B/xactengine2_1.dll", "../resources/tools/fix-dx/Data/B/xactengine2_1.dll"),
            companion!("Data/B/xactengine2_2.dll", "../resources/tools/fix-dx/Data/B/xactengine2_2.dll"),
            companion!("Data/B/xactengine2_3.dll", "../resources/tools/fix-dx/Data/B/xactengine2_3.dll"),
            companion!("Data/B/xactengine2_7.dll", "../resources/tools/fix-dx/Data/B/xactengine2_7.dll"),
            companion!("Data/B/xinput1_1.dll", "../resources/tools/fix-dx/Data/B/xinput1_1.dll"),
            companion!("Data/B/xinput1_2.dll", "../resources/tools/fix-dx/Data/B/xinput1_2.dll"),
            companion!("Data/B/xinput1_3.dll", "../resources/tools/fix-dx/Data/B/xinput1_3.dll"),
            companion!("Data/B/xinput9_1_0.dll", "../resources/tools/fix-dx/Data/B/xinput9_1_0.dll"),
        ],
        wait_for_exit: true,
    },
    BundledTool {
        key: "technicianReinstall",
        label: "技术员一键重装(vip0)",
        filename: "技术员一键重装.exe",
        bytes: include_bytes!("../resources/tools/system-reinstall/技术员一键重装.exe"),
        companions: &[],
        wait_for_exit: true,
    },
    BundledTool {
        key: "tianmiaoReinstall",
        label: "天喵一键重装(1788)",
        filename: "天喵一键重装.exe",
        bytes: include_bytes!("../resources/tools/system-reinstall/天喵一键重装.exe"),
        companions: &[],
        wait_for_exit: true,
    },
];

fn find_bundled_tool(tool_key: &str) -> Option<&'static BundledTool> {
    BUNDLED_TOOLS.iter().find(|tool| tool.key == tool_key)
}

fn run_hidden_command(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| format!("启动 {program} 失败：{e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Err(if detail.is_empty() {
            format!("{program} 执行失败，退出码：{}", output.status)
        } else {
            detail
        })
    }
}

fn run_elevated_cmd_script(script: &str) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("system_toolbox_elevated_task.cmd");
    fs::write(&script_path, script).map_err(|e| format!("创建提权脚本失败：{e}"))?;

    let escaped_script_path = script_path.to_string_lossy().replace('\'', "''");
    let ps_command = format!(
        "Start-Process -FilePath cmd.exe -ArgumentList '/c \"{escaped_script_path}\"' -Verb RunAs -Wait"
    );

    let result = run_hidden_command(
        "powershell",
        &["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_command],
    );

    let _ = fs::remove_file(script_path);
    result.map_err(|e| format!("提权写入失败或已取消 UAC 授权：{e}"))
}

fn parse_guid(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|token| token.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-'))
        .find(|token| {
            token.len() == 36
                && token.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
                && token.matches('-').count() == 4
        })
        .map(|token| token.to_string())
}

fn query_reg_dword(key: &str, value: &str) -> Result<u32, String> {
    let output = Command::new("reg")
        .args(["query", key, "/v", value])
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| format!("读取注册表失败：{e}"))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            format!("读取注册表失败：{key}\\{value}")
        } else {
            detail
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_value = stdout
        .lines()
        .find(|line| line.contains(value))
        .and_then(|line| line.split_whitespace().last())
        .ok_or_else(|| format!("未找到注册表值：{key}\\{value}"))?;

    if let Some(hex) = raw_value.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| format!("解析注册表值失败：{e}"))
    } else {
        raw_value
            .parse::<u32>()
            .map_err(|e| format!("解析注册表值失败：{e}"))
    }
}

/// 将嵌入的工具释放到临时目录，返回释放后的路径。
fn extract_tool(tool: &BundledTool) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("system_toolbox_tools");
    fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败：{e}"))?;

    let dest = dir.join(tool.filename);
    // 如果文件已存在且大小一致，跳过写入。
    let need_write = match fs::metadata(&dest) {
        Ok(meta) => meta.len() != tool.bytes.len() as u64,
        Err(_) => true,
    };
    if need_write {
        fs::write(&dest, tool.bytes).map_err(|e| format!("释放工具文件失败：{e}"))?;
    }

    // 释放附带文件到同目录
    for companion in tool.companions {
        let companion_dest = dir.join(companion.filename);
        if let Some(parent) = companion_dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建附带文件目录失败：{e}"))?;
        }
        let need = match fs::metadata(&companion_dest) {
            Ok(meta) => meta.len() != companion.bytes.len() as u64,
            Err(_) => true,
        };
        if need {
            fs::write(&companion_dest, companion.bytes)
                .map_err(|e| format!("释放配置文件失败：{e}"))?;
        }
    }

    Ok(dest)
}

#[tauri::command]
fn disable_uac_and_file_warning() -> Result<String, String> {
    let low_risk_types = ".exe;.bat;.cmd;.vbs;.js;.msi;.reg;.ps1;.zip;.rar;.7z";
    let current_user_cmds = [
        [
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Associations",
            "/v",
            "LowRiskFileTypes",
            "/t",
            "REG_SZ",
            "/d",
            low_risk_types,
            "/f",
        ],
        [
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Attachments",
            "/v",
            "SaveZoneInformation",
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ],
    ];

    let mut failed = Vec::new();
    for args in current_user_cmds {
        if let Err(e) = run_hidden_command("reg", &args) {
            failed.push(e);
        }
    }

    let elevated_script = format!(
        r#"@echo off
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" /v EnableLUA /t REG_DWORD /d 0 /f
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" /v ConsentPromptBehaviorAdmin /t REG_DWORD /d 0 /f
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" /v PromptOnSecureDesktop /t REG_DWORD /d 0 /f
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Associations" /v LowRiskFileTypes /t REG_SZ /d "{low_risk_types}" /f
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Attachments" /v SaveZoneInformation /t REG_DWORD /d 1 /f
"#
    );

    if let Err(e) = run_elevated_cmd_script(&elevated_script) {
        failed.push(e);
    }

    if failed.is_empty() {
        Ok("已关闭UAC通知和文件安全警告。重启电脑生效！".to_string())
    } else {
        Err(format!(
            "部分操作失败：{}。请确认已在弹出的 UAC 窗口中点“是”，并在完成后重启电脑。",
            failed.join("; ")
        ))
    }
}

#[tauri::command]
fn set_high_performance_power_plan() -> Result<String, String> {
    const HIGH_PERFORMANCE_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";

    if run_hidden_command("powercfg", &["/setactive", HIGH_PERFORMANCE_GUID]).is_ok() {
        return Ok("设置成功。".to_string());
    }

    let output = Command::new("powercfg")
        .args(["/duplicatescheme", HIGH_PERFORMANCE_GUID])
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| format!("创建高性能电源计划失败：{e}"))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            "创建高性能电源计划失败，请确认系统支持 powercfg。".to_string()
        } else {
            format!("创建高性能电源计划失败：{detail}")
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duplicated_guid = parse_guid(&stdout).ok_or_else(|| {
        "已尝试创建高性能电源计划，但无法识别新计划 GUID。请手动检查 powercfg 输出。".to_string()
    })?;

    run_hidden_command("powercfg", &["/setactive", &duplicated_guid])
        .map_err(|e| format!("设置高性能电源计划失败：{e}"))?;

    Ok("已创建并启用高性能电源计划。".to_string())
}

#[tauri::command]
fn permanently_disable_firewall_by_registry() -> Result<String, String> {
    const FIREWALL_CONFIG_PROFILES: &[&str] = &[
        r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\DomainProfile",
        r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\StandardProfile",
        r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\PublicProfile",
    ];

    const FIREWALL_POLICY_PROFILES: &[&str] = &[
        r"HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\DomainProfile",
        r"HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\StandardProfile",
        r"HKLM\SOFTWARE\Policies\Microsoft\WindowsFirewall\PublicProfile",
    ];

    let mut script = String::from("@echo off\r\n");
    for key in FIREWALL_CONFIG_PROFILES {
        script.push_str(&format!(
            r#"reg add "{key}" /v EnableFirewall /t REG_DWORD /d 0 /f"#,
        ));
        script.push_str("\r\n");
    }
    for key in FIREWALL_POLICY_PROFILES {
        script.push_str(&format!(
            r#"reg add "{key}" /v EnableFirewall /t REG_DWORD /d 0 /f"#,
        ));
        script.push_str("\r\n");
    }
    script.push_str("netsh advfirewall set allprofiles state off\r\n");

    run_elevated_cmd_script(&script)?;

    Ok("已禁用Windows防火墙。".to_string())
}

#[tauri::command]
fn open_yy_download_page() -> Result<String, String> {
    run_hidden_command(
        "powershell",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Start-Process 'https://xiaodao.lanzout.com/b032c447pe'",
        ],
    )
    .map_err(|e| format!("打开YY绿色多开版下载页面失败：{e}"))?;

    Ok("已打开YY绿色多开版下载页面。".to_string())
}

#[tauri::command]
fn open_qishui_music_page() -> Result<String, String> {
    run_hidden_command(
        "powershell",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Start-Process 'https://www.qishui.com/'",
        ],
    )
    .map_err(|e| format!("打开汽水音乐页面失败：{e}"))?;

    Ok("已打开汽水音乐页面。".to_string())
}

#[tauri::command]
fn open_google_chrome_page() -> Result<String, String> {
    run_hidden_command(
        "powershell",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Start-Process 'https://www.google.cn/chrome/index.html'",
        ],
    )
    .map_err(|e| format!("打开谷歌浏览器页面失败：{e}"))?;

    Ok("已打开谷歌浏览器页面。".to_string())
}

#[tauri::command]
fn open_external_url(url: String) -> Result<String, String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("只允许打开 http/https 链接".to_string());
    }

    Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", trimmed])
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|e| format!("打开链接失败：{e}"))?;

    Ok("已打开链接。".to_string())
}

#[tauri::command]
async fn open_bundled_tool(app: tauri::AppHandle, tool_key: String) -> Result<String, String> {
    // 只允许打开白名单里的内置工具，避免前端传入任意本地路径执行。
    let tool = find_bundled_tool(&tool_key).ok_or_else(|| "未配置该工具。".to_string())?;
    let label = tool.label.to_string();
    let wait = tool.wait_for_exit;

    let tool_path = extract_tool(tool)?;

    let tool_dir = tool_path.parent().unwrap().to_path_buf();

    if wait {
        // 需要等待的工具：最小化主窗口，等工具关闭后恢复。
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.minimize();
        }

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let status = Command::new(&tool_path)
                .current_dir(&tool_dir)
                .creation_flags(CREATE_NEW_CONSOLE)
                .status();
            let _ = tx.send(status);
        });

        let _result = rx
            .recv()
            .map_err(|e| format!("任务执行失败：{e}"))?
            .map_err(|e| format!("打开{}失败：{e}", label))?;

        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    } else {
        // 不需要等待的工具：直接启动，不最小化。
        Command::new(&tool_path)
            .current_dir(&tool_dir)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map_err(|e| format!("打开{}失败：{e}", label))?;
    }

    Ok(format!("{}已打开。", label))
}

fn build_crawler_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/125.0.0.0 Safari/537.36",
        )
        .cookie_store(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败：{e}"))
}

async fn fetch_taobao(client: &reqwest::Client, keyword: &str) -> Result<f64, String> {
    let url = format!(
        "https://s.taobao.com/search?q={}&sort=default&style=list",
        urlencoding(keyword),
    );
    let html = client
        .get(&url)
        .header("Accept-Language", "zh-CN,zh;q=0.9")
        .header("Referer", "https://www.taobao.com/")
        .send()
        .await
        .map_err(|e| format!("淘宝请求失败：{e}"))?
        .text()
        .await
        .map_err(|e| format!("淘宝响应读取失败：{e}"))?;
    parse_first_price(&html).ok_or_else(|| "淘宝未解析到价格".to_string())
}

async fn fetch_douyin(client: &reqwest::Client, keyword: &str) -> Result<f64, String> {
    let url = format!(
        "https://www.douyin.com/search/{}?type=product",
        urlencoding(keyword),
    );
    let html = client
        .get(&url)
        .header("Accept-Language", "zh-CN,zh;q=0.9")
        .send()
        .await
        .map_err(|e| format!("抖音请求失败：{e}"))?
        .text()
        .await
        .map_err(|e| format!("抖音响应读取失败：{e}"))?;
    parse_first_price(&html).ok_or_else(|| "抖音未解析到价格".to_string())
}

fn find_python() -> Option<String> {
    // 先尝试 PATH 里的命令
    for cmd in &["python", "py", "python3"] {
        let ok = Command::new(cmd)
            .arg("--version")
            .creation_flags(0x0800_0000)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return Some(cmd.to_string());
        }
    }
    // 兜底：扫描 Windows 常见安装路径
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let candidates = [
        format!(r"{local}\Programs\Python\Python314\python.exe"),
        format!(r"{local}\Programs\Python\Python313\python.exe"),
        format!(r"{local}\Programs\Python\Python312\python.exe"),
        format!(r"{local}\Programs\Python\Python311\python.exe"),
        format!(r"{local}\Programs\Python\Python310\python.exe"),
        r"C:\Python314\python.exe".to_string(),
        r"C:\Python312\python.exe".to_string(),
        r"C:\Python311\python.exe".to_string(),
        r"C:\Python310\python.exe".to_string(),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }
    None
}

fn extract_xianyu_script() -> Result<PathBuf, String> {
    static SCRIPT: &[u8] = include_bytes!("../resources/scripts/xianyu_search.py");
    static FILTER_CONFIG: &[u8] = include_bytes!("../resources/scripts/xianyu_filter_keywords.json");
    let dir = std::env::temp_dir().join("system_toolbox_scripts");
    fs::create_dir_all(&dir).map_err(|e| format!("创建脚本目录失败：{e}"))?;
    let path = dir.join("xianyu_search.py");
    let config_path = dir.join("xianyu_filter_keywords.json");
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("scripts")
        .join("xianyu_search.py");
    let source_config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("scripts")
        .join("xianyu_filter_keywords.json");
    let script_bytes = fs::read(&source_path).unwrap_or_else(|_| SCRIPT.to_vec());
    let config_bytes = fs::read(&source_config_path).unwrap_or_else(|_| FILTER_CONFIG.to_vec());
    fs::write(&path, script_bytes).map_err(|e| format!("写出爬虫脚本失败：{e}"))?;
    fs::write(&config_path, config_bytes).map_err(|e| format!("写出过滤词配置失败：{e}"))?;
    Ok(path)
}

async fn fetch_xianyu(
    _client: &reqwest::Client,
    keyword: &str,
    category: Option<&str>,
    app: tauri::AppHandle,
) -> Result<XianyuFetchResult, String> {
    let mut args = Vec::new();
    if let Some(category) = category.filter(|value| !value.trim().is_empty()) {
        args.push("--category".to_string());
        args.push(category.to_string());
    }
    args.push(keyword.to_string());
    run_xianyu_script(app, args, true).await
}

async fn run_xianyu_script(
    app: tauri::AppHandle,
    args: Vec<String>,
    expect_price: bool,
) -> Result<XianyuFetchResult, String> {
    let python = find_python().ok_or_else(|| "未找到 Python，请先安装 Python 3.x".to_string())?;
    let script = extract_xianyu_script()?;
    let script_display = script.display().to_string();

    tokio::task::spawn_blocking(move || -> Result<XianyuFetchResult, String> {
        let _ = app.emit("xianyu-log", serde_json::json!({
            "status": "log",
            "type": "info",
            "message": format!("启动咸鱼脚本：{}", script_display),
        }));

        let mut command = Command::new(&python);
        command.arg(&script);
        command.env("PYTHONIOENCODING", "utf-8");
        for arg in args {
            command.arg(arg);
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x0800_0000)
            .spawn()
            .map_err(|e| format!("执行脚本失败：{e}"))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| "无法获取脚本输出".to_string())?;
        if let Some(stderr) = child.stderr.take() {
            let stderr_app = app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let _ = stderr_app.emit("xianyu-log", serde_json::json!({
                        "status": "log",
                        "type": "error",
                        "message": format!("Python stderr：{}", trimmed),
                    }));
                }
            });
        }

        let reader = BufReader::new(stdout);
        let mut last_result = XianyuFetchResult::default();
        let mut last_error: Option<String> = None;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if !trimmed.starts_with('{') {
                continue;
            }
            let json: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => continue,
            };

            match json.get("status").and_then(|v| v.as_str()) {
                Some("log") => {
                    let _ = app.emit("xianyu-log", json.clone());
                }
                Some("need_login") => {
                    let _ = app.emit("xianyu-need-login", ());
                }
                Some("need_login_required") => {
                    last_error = Some("请先完成咸鱼扫码登录".to_string());
                }
                Some("qr_ready") => {
                    let qr_b64 = json
                        .get("qr_b64")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !qr_b64.is_empty() {
                        if let Ok(mut guard) = app.state::<XianyuQrState>().qr_b64.lock() {
                            *guard = Some(qr_b64.clone());
                        }
                        let _ = app.emit("xianyu-qr-ready", qr_b64);
                    }
                }
                Some("login_ok") => {
                    let _ = app.emit("xianyu-login-ok", json.clone());
                    if let Ok(mut guard) = app.state::<XianyuQrState>().qr_b64.lock() {
                        *guard = None;
                    }
                }
                _ => {
                    if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                        last_error = Some(err.to_string());
                    } else if let Some(price_val) = json.get("xianyu") {
                        if let Some(price) = price_val.as_f64() {
                            last_result.price = Some(price);
                        }
                        last_result.url = json
                            .get("xianyu_url")
                            .and_then(|v| v.as_str())
                            .filter(|v| !v.is_empty())
                            .map(|v| v.to_string());
                    }
                }
            }
        }

        let _ = child.wait();
        if let Some(err) = last_error {
            return Err(err);
        }
        if expect_price {
            Ok(last_result)
        } else {
            Ok(XianyuFetchResult::default())
        }
    })
    .await
    .map_err(|e| format!("spawn 失败：{e}"))?
}

/// 从 HTML 文本中用正则提取第一个形如 ¥123.00 / 123.00 的价格数字。
/// 各平台 HTML 结构不同，后续按需替换为更精确的选择器解析。
fn parse_first_price(html: &str) -> Option<f64> {
    let prefixes: &[&str] = &[
        r#""price":""#,
        "¥",
        "price\">",
    ];
    for prefix in prefixes {
        if let Some(start) = html.find(prefix) {
            let rest = &html[start + prefix.len()..];
            let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(rest.len());
            let cap = &rest[..end];
            if let Ok(v) = cap.parse::<f64>() {
                if v > 0.0 { return Some(v); }
            }
        }
    }
    None
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                vec![c]
            } else {
                let encoded = format!("%{:02X}", c as u32);
                encoded.chars().collect()
            }
        })
        .collect()
}

#[tauri::command]
fn clear_xianyu_session() -> Result<String, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "无法读取 APPDATA 环境变量".to_string())?;
    let dir = std::path::Path::new(&appdata).join("system_toolbox");
    let session_path = dir.join("xianyu_session.json");
    let profile_path = dir.join("xianyu_profile.json");
    let had_login = session_path.exists() || profile_path.exists();
    if session_path.exists() {
        fs::remove_file(&session_path).map_err(|e| format!("删除登录态失败：{e}"))?;
    }
    if profile_path.exists() {
        fs::remove_file(&profile_path).map_err(|e| format!("删除登录信息失败：{e}"))?;
    }
    if had_login {
        Ok("咸鱼登录态已清除。".to_string())
    } else {
        Ok("暂无保存的登录态。".to_string())
    }
}

fn format_xianyu_username(profile: Option<&serde_json::Value>) -> String {
    let nick = profile
        .and_then(|json| json.get("candidate_cookies"))
        .and_then(|cookies| cookies.as_object())
        .and_then(|cookies| {
            ["tracknick", "_nk_", "lgc", "sn", "unb"]
                .iter()
                .find_map(|key| cookies.get(*key).and_then(|value| value.as_str()))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match nick {
        Some(name) => format!("已登录/{name}"),
        None => "已登录".to_string(),
    }
}

#[tauri::command]
fn get_xianyu_session_info() -> Result<XianyuSessionInfo, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "无法读取 APPDATA 环境变量".to_string())?;
    let dir = std::path::Path::new(&appdata).join("system_toolbox");
    let session_path = dir.join("xianyu_session.json");
    let profile_path = dir.join("xianyu_profile.json");
    let profile = fs::read_to_string(profile_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let username = if session_path.exists() {
        Some(format_xianyu_username(profile.as_ref()))
    } else {
        None
    };

    Ok(XianyuSessionInfo {
        logged_in: session_path.exists(),
        username,
        profile,
    })
}

#[tauri::command]
fn get_xianyu_qr(state: tauri::State<XianyuQrState>) -> Option<String> {
    state.qr_b64.lock().unwrap().clone()
}

#[tauri::command]
fn clear_xianyu_qr(state: tauri::State<XianyuQrState>) {
    if let Ok(mut guard) = state.qr_b64.lock() {
        *guard = None;
    }
}

#[tauri::command]
async fn start_xianyu_login(app: tauri::AppHandle) -> Result<(), String> {
    if let Ok(mut guard) = app.state::<XianyuQrState>().qr_b64.lock() {
        *guard = None;
    }
    run_xianyu_script(app, vec!["--login".to_string()], false).await?;
    Ok(())
}

#[tauri::command]
async fn crawl_prices(app: tauri::AppHandle, keyword: String, category: Option<String>) -> Result<PriceResult, String> {
    if keyword.trim().is_empty() {
        return Ok(PriceResult { taobao: None, douyin: None, xianyu: None, xianyu_url: None });
    }
    let client = build_crawler_client()?;
    let (tb, dy, xy) = tokio::join!(
        fetch_taobao(&client, &keyword),
        fetch_douyin(&client, &keyword),
        fetch_xianyu(&client, &keyword, category.as_deref(), app),
    );
    let xy = xy?;
    Ok(PriceResult {
        taobao: tb.ok(),
        douyin: dy.ok(),
        xianyu: xy.price,
        xianyu_url: xy.url,
    })
}

pub fn run() {
    tauri::Builder::default()
        .manage(XianyuQrState { qr_b64: Mutex::new(None) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_bundled_tool,
            disable_uac_and_file_warning,
            set_high_performance_power_plan,
            permanently_disable_firewall_by_registry,
            open_yy_download_page,
            open_qishui_music_page,
            open_google_chrome_page,
            open_external_url,
            crawl_prices,
            clear_xianyu_session,
            get_xianyu_qr,
            clear_xianyu_qr,
            get_xianyu_session_info,
            start_xianyu_login,
        ])
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                // 开发模式自动打开 WebView DevTools，方便查看前端 console 和网络请求。
                if let Some(window) = _app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
