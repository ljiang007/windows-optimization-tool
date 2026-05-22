use std::process::Command;
use tauri::{path::BaseDirectory, Manager};

struct BundledTool {
    key: &'static str,
    label: &'static str,
    resource_path: &'static str,
}

const BUNDLED_TOOLS: &[BundledTool] = &[
    BundledTool {
        key: "windowsActivation",
        label: "Windows激活",
        resource_path: "resources/tools/windows-activation/windowsActivation.exe",
    },
];

fn find_bundled_tool(tool_key: &str) -> Option<&'static BundledTool> {
    BUNDLED_TOOLS.iter().find(|tool| tool.key == tool_key)
}

#[tauri::command]
fn open_bundled_tool(app: tauri::AppHandle, tool_key: String) -> Result<String, String> {
    // 只允许打开白名单里的内置工具，避免前端传入任意本地路径执行。
    let tool = find_bundled_tool(&tool_key).ok_or_else(|| "未配置该工具。".to_string())?;

    // 开发和打包后都通过 Tauri Resource 目录解析资源路径。
    let tool_path = app
        .path()
        .resolve(tool.resource_path, BaseDirectory::Resource)
        .map_err(|error| format!("资源路径解析失败：{error}"))?;

    if !tool_path.exists() {
        return Err(format!("未找到工具文件：{}", tool_path.display()));
    }

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


