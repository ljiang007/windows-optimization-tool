use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

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
            .spawn()
            .map_err(|e| format!("打开{}失败：{e}", label))?;
    }

    Ok(format!("{}已打开。", label))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_bundled_tool,
            disable_uac_and_file_warning,
            set_high_performance_power_plan,
            permanently_disable_firewall_by_registry,
            open_yy_download_page,
            open_qishui_music_page,
            open_google_chrome_page
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
