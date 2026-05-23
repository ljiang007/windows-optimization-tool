use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

struct CompanionFile {
    filename: &'static str,
    bytes: &'static [u8],
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
];

fn find_bundled_tool(tool_key: &str) -> Option<&'static BundledTool> {
    BUNDLED_TOOLS.iter().find(|tool| tool.key == tool_key)
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
        .invoke_handler(tauri::generate_handler![open_bundled_tool])
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


