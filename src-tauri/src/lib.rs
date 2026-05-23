use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct BundledTool {
    key: &'static str,
    label: &'static str,
    filename: &'static str,
    bytes: &'static [u8],
}

const BUNDLED_TOOLS: &[BundledTool] = &[
    BundledTool {
        key: "windowsActivation",
        label: "Windows激活",
        filename: "windowsActivation.exe",
        bytes: include_bytes!("../resources/tools/windows-activation/windowsActivation.exe"),
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
    Ok(dest)
}

#[tauri::command]
fn open_bundled_tool(tool_key: String) -> Result<String, String> {
    // 只允许打开白名单里的内置工具，避免前端传入任意本地路径执行。
    let tool = find_bundled_tool(&tool_key).ok_or_else(|| "未配置该工具。".to_string())?;

    let tool_path = extract_tool(tool)?;

    // 用独立进程启动 exe，启动后的窗口行为由外部工具自身决定。
    Command::new(&tool_path)
        .spawn()
        .map_err(|error| format!("打开{}失败：{error}", tool.label))?;

    Ok(format!("{}已打开。", tool.label))
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


