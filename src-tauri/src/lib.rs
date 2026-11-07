mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("正在启动应用...");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_tool,
            commands::install_tool,
            commands::open_tool,
            commands::uninstall_tool,
        ])
        .setup(|app| {
            println!("Setup 完成，窗口应该已创建");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}
