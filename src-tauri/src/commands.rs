use serde::{Deserialize, Serialize};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolStatus {
    pub installed: bool,
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub error: Option<String>,
}

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", cmd])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_status(cmd: &str, args: &[&str]) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", cmd])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    Ok(status.success())
}

fn get_npm_latest_version(package: &str) -> String {
    match run_command("npm", &["view", package, "version"]) {
        Ok(version) => version.trim().to_string(),
        Err(_) => String::new(),
    }
}

fn normalize_version(version: &str) -> String {
    let line = version.lines().next().unwrap_or("").trim();
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts.iter().rev() {
        let cleaned = part.trim_start_matches('v').trim();
        if cleaned.contains('.') && cleaned.chars().any(|c| c.is_ascii_digit()) {
            return cleaned.to_string();
        }
    }
    line.trim_start_matches('v').trim().to_string()
}

fn check_tool_with_version(cmd: &str, package: &str) -> ToolStatus {
    let installed = run_command(cmd, &["--version"]);
    let latest = get_npm_latest_version(package);

    match installed {
        Ok(output) => {
            let current = normalize_version(&output);
            let latest_normalized = normalize_version(&latest);
            let needs_update = !latest_normalized.is_empty() && !current.is_empty() && current != latest_normalized;
            ToolStatus {
                installed: true,
                current_version: current,
                latest_version: latest,
                needs_update,
            }
        }
        Err(_) => ToolStatus {
            installed: false,
            current_version: String::new(),
            latest_version: latest,
            needs_update: false,
        },
    }
}

fn check_git() -> ToolStatus {
    match run_command("git", &["--version"]) {
        Ok(output) => {
            let version = output.trim().trim_start_matches("git version ").to_string();
            ToolStatus {
                installed: true,
                current_version: version,
                latest_version: String::new(),
                needs_update: false,
            }
        }
        Err(_) => ToolStatus {
            installed: false,
            current_version: String::new(),
            latest_version: String::new(),
            needs_update: false,
        },
    }
}

fn check_node() -> ToolStatus {
    match run_command("node", &["--version"]) {
        Ok(output) => {
            let version = output.trim().trim_start_matches('v').to_string();
            ToolStatus {
                installed: true,
                current_version: version,
                latest_version: String::new(),
                needs_update: false,
            }
        }
        Err(_) => ToolStatus {
            installed: false,
            current_version: String::new(),
            latest_version: String::new(),
            needs_update: false,
        },
    }
}

fn check_claude() -> ToolStatus {
    check_tool_with_version("claude", "@anthropic-ai/claude-code")
}

fn check_codex() -> ToolStatus {
    check_tool_with_version("codex", "@openai/codex")
}

fn check_gemini() -> ToolStatus {
    check_tool_with_version("gemini", "@google/gemini-cli")
}

fn check_opencode() -> ToolStatus {
    check_tool_with_version("opencode", "opencode-ai")
}

fn check_openclaw() -> ToolStatus {
    check_tool_with_version("openclaw", "openclaw")
}

fn check_hermes() -> ToolStatus {
    match run_command("hermes", &["--version"]) {
        Ok(output) => {
            let version = normalize_version(&output);
            ToolStatus {
                installed: true,
                current_version: version,
                latest_version: String::new(),
                needs_update: false,
            }
        }
        Err(_) => ToolStatus {
            installed: false,
            current_version: String::new(),
            latest_version: String::new(),
            needs_update: false,
        },
    }
}

fn check_mimo() -> ToolStatus {
    check_tool_with_version("mimo", "@mimo-ai/cli")
}

fn check_kilocode() -> ToolStatus {
    check_tool_with_version("kilocode", "@kilocode/cli")
}

fn check_ccswitch() -> ToolStatus {
    #[cfg(target_os = "windows")]
    {
        // Check via uninstall registry first (fastest)
        let output = run_command("powershell", &["-Command",
            "Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like '*CC-Switch*' -or $_.DisplayName -like '*CC Switch*' } | Select-Object -First 1 -ExpandProperty DisplayVersion"
        ]);
        if let Ok(ver) = output {
            if !ver.is_empty() {
                return ToolStatus {
                    installed: true,
                    current_version: ver,
                    latest_version: String::new(),
                    needs_update: false,
                };
            }
        }
        // Fallback: find the exe and get version from file properties
        if let Some(path) = find_ccswitch_path() {
            let version = run_command("powershell", &["-Command", &format!(
                "(Get-Item '{}' | Select-Object -ExpandProperty VersionInfo).ProductVersion", path
            )])
            .unwrap_or_default();
            return ToolStatus {
                installed: true,
                current_version: if version.is_empty() { "已安装".to_string() } else { version },
                latest_version: String::new(),
                needs_update: false,
            };
        }
    }
    ToolStatus {
        installed: false,
        current_version: String::new(),
        latest_version: String::new(),
        needs_update: false,
    }
}

fn install_npm_package(package: &str) -> Result<(), String> {
    if run_status("npm", &["install", "-g", package])? {
        return Ok(());
    }
    if run_status("pnpm", &["add", "-g", package])? {
        return Ok(());
    }
    Err(format!("安装 {} 失败，请确保已安装 Node.js", package))
}

fn uninstall_npm_package(package: &str) -> Result<(), String> {
    if run_status("npm", &["uninstall", "-g", package])? {
        return Ok(());
    }
    if run_status("pnpm", &["remove", "-g", package])? {
        return Ok(());
    }
    Err("卸载失败，请手动卸载".to_string())
}

fn install_git() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if run_status("winget", &["install", "--id", "Git.Git", "-e", "--accept-source-agreements", "--accept-package-agreements"])? {
            return Ok(());
        }
        if run_status("scoop", &["install", "git"])? {
            return Ok(());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if run_status("scoop", &["install", "git"])? {
            return Ok(());
        }
    }
    Err("请手动安装 Git: https://git-scm.com/download/win".to_string())
}

fn install_node() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if run_status("winget", &["install", "--id", "OpenJS.NodeJS.LTS", "-e", "--accept-source-agreements", "--accept-package-agreements"])? {
            return Ok(());
        }
        if run_status("nvm", &["install", "lts"])? {
            return Ok(());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if run_status("nvm", &["install", "lts"])? {
            return Ok(());
        }
    }
    Err("请手动安装 Node.js: https://nodejs.org/en/download".to_string())
}

fn install_hermes() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if run_status("powershell", &["-NoProfile", "-ExecutionPolicy", "Bypass", "-EncodedCommand", "aQByAG0AIABoAHQAdABwAHMAOgAvAC8AcgBhAHcALgBnAGkAdABoAHUAYgB1AHMAZQByAGMAbwBuAHQAZQBuAHQALgBjAG8AbQAvAE4AbwB1AHMAUgBlAHMAZQBhAHIAYwBoAC8AaABlAHIAbQBlAHMALQBhAGcAZQBuAHQALwBtAGEAaQBuAC8AcwBjAHIAaQBwAHQAcwAvAGkAbgBzAHQAYQBsAGwALgBwAHMAMQAgAHwAIABpAGUAeAA="])? {
            return Ok(());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if run_status("bash", &["-c", "curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash"])? {
            return Ok(());
        }
    }
    Err("安装 Hermes 失败，请手动安装: https://github.com/NousResearch/hermes-agent".to_string())
}

fn uninstall_hermes() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if run_status("hermes", &["uninstall"])? {
            return Ok(());
        }
        if run_status("powershell", &["-Command", "Remove-Item -Path \"$env:USERPROFILE\\.hermes\" -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item -Path \"$env:LOCALAPPDATA\\hermes\" -Recurse -Force -ErrorAction SilentlyContinue"])? {
            return Ok(());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if run_status("bash", &["-c", "hermes uninstall 2>/dev/null || rm -rf ~/.hermes ~/.local/bin/hermes"])? {
            return Ok(());
        }
    }
    Err("卸载 Hermes 失败，请手动删除".to_string())
}

fn install_ccswitch() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Download and install CC-Switch .msi
        let ps_script = r#"
            $ProgressPreference = 'SilentlyContinue'
            $url = 'https://github.com/farion1231/cc-switch/releases/latest/download/CC-Switch-Windows.msi'
            $msi = Join-Path $env:TEMP 'CC-Switch-Setup.msi'
            try {
                Invoke-WebRequest -Uri $url -OutFile $msi -UseBasicParsing -ErrorAction Stop
                Start-Process msiexec.exe -ArgumentList '/i',$msi,'/quiet','/norestart' -Wait -NoNewWindow
                Remove-Item $msi -Force -ErrorAction SilentlyContinue
                exit 0
            } catch {
                exit 1
            }
        "#;
        if run_status("powershell", &["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", ps_script])? {
            return Ok(());
        }
    }
    Err("安装 CC-Switch 失败，请手动安装: https://github.com/farion1231/cc-switch/releases".to_string())
}

fn uninstall_ccswitch() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Try to find and uninstall via msiexec
        let output = run_command("powershell", &["-Command",
            "Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like '*CC-Switch*' -or $_.DisplayName -like '*CC Switch*' } | Select-Object -First 1 -ExpandProperty UninstallString"
        ]);
        if let Ok(uninstall_str) = output {
            if !uninstall_str.is_empty() {
                // Extract product code from uninstall string and run msiexec /x
                let status = Command::new("cmd")
                    .args(["/C", "msiexec", "/x", &uninstall_str.replace("MsiExec.exe /I", "").replace("MsiExec.exe /X", "").trim(), "/quiet", "/norestart"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status();
                if let Ok(s) = status {
                    if s.success() {
                        return Ok(());
                    }
                }
            }
        }
        // Fallback: try winget uninstall
        if run_status("winget", &["uninstall", "--id", "CC-Switch", "-e", "--accept-source-agreements"])? {
            return Ok(());
        }
        // Fallback: delete install directory
        if run_status("powershell", &["-Command", "Remove-Item -Path 'C:\\Program Files\\CC-Switch' -Recurse -Force -ErrorAction SilentlyContinue"])? {
            return Ok(());
        }
    }
    Err("卸载 CC-Switch 失败，请手动卸载".to_string())
}

#[tauri::command]
pub async fn check_tool(tool: String) -> Result<ToolStatus, String> {
    tokio::task::spawn_blocking(move || {
        match tool.as_str() {
            "git" => Ok(check_git()),
            "node" => Ok(check_node()),
            "claude" => Ok(check_claude()),
            "codex" => Ok(check_codex()),
            "gemini" => Ok(check_gemini()),
            "opencode" => Ok(check_opencode()),
            "openclaw" => Ok(check_openclaw()),
            "hermes" => Ok(check_hermes()),
            "mimo" => Ok(check_mimo()),
            "kilocode" => Ok(check_kilocode()),
            "ccswitch" => Ok(check_ccswitch()),
            _ => Err(format!("未知工具: {}", tool)),
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn install_tool(tool: String) -> Result<InstallResult, String> {
    tokio::task::spawn_blocking(move || {
        let result = match tool.as_str() {
            "git" => install_git(),
            "node" => install_node(),
            "claude" => install_npm_package("@anthropic-ai/claude-code"),
            "codex" => install_npm_package("@openai/codex"),
            "gemini" => install_npm_package("@google/gemini-cli"),
            "opencode" => install_npm_package("opencode-ai@latest"),
            "openclaw" => install_npm_package("openclaw@latest"),
            "hermes" => install_hermes(),
            "mimo" => install_npm_package("@mimo-ai/cli"),
            "kilocode" => install_npm_package("@kilocode/cli"),
            "ccswitch" => install_ccswitch(),
            _ => return Err(format!("未知工具: {}", tool)),
        };

        match result {
            Ok(()) => Ok(InstallResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(InstallResult {
                success: false,
                error: Some(e),
            }),
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn find_ccswitch_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // Try common install paths
        let common_paths = [
            "C:\\Program Files\\CC-Switch\\CC-Switch.exe",
            "C:\\Program Files (x86)\\CC-Switch\\CC-Switch.exe",
            "D:\\Program Files\\CC-Switch\\CC-Switch.exe",
            "D:\\CC-Switch\\CC-Switch.exe",
        ];
        for path in common_paths.iter() {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        // Search via registry InstallLocation
        let output = run_command("powershell", &["-Command",
            "Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like '*CC-Switch*' -or $_.DisplayName -like '*CC Switch*' } | Select-Object -First 1 -ExpandProperty InstallLocation"
        ]);
        if let Ok(loc) = output {
            if !loc.is_empty() {
                let exe_path = format!("{}\\CC-Switch.exe", loc.trim_end_matches('\\'));
                if std::path::Path::new(&exe_path).exists() {
                    return Some(exe_path);
                }
            }
        }
        // Search via registry UninstallString
        let output = run_command("powershell", &["-Command",
            "Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKCU:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like '*CC-Switch*' -or $_.DisplayName -like '*CC Switch*' } | Select-Object -First 1 -ExpandProperty UninstallString"
        ]);
        if let Ok(uninstall_str) = output {
            if !uninstall_str.is_empty() {
                // Extract directory from uninstall string
                let dir = uninstall_str.trim().trim_matches('"');
                if let Some(parent) = std::path::Path::new(dir).parent() {
                    let exe_path = format!("{}\\CC-Switch.exe", parent.display());
                    if std::path::Path::new(&exe_path).exists() {
                        return Some(exe_path);
                    }
                }
            }
        }
        // Search entire drives using where.exe
        let output = run_command("where", &["/R", "C:\\", "CC-Switch.exe"]);
        if let Ok(paths) = output {
            if let Some(first_line) = paths.lines().next() {
                let trimmed = first_line.trim();
                if !trimmed.is_empty() && std::path::Path::new(trimmed).exists() {
                    return Some(trimmed.to_string());
                }
            }
        }
        let output = run_command("where", &["/R", "D:\\", "CC-Switch.exe"]);
        if let Ok(paths) = output {
            if let Some(first_line) = paths.lines().next() {
                let trimmed = first_line.trim();
                if !trimmed.is_empty() && std::path::Path::new(trimmed).exists() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

#[tauri::command]
pub async fn open_tool(tool: String) -> Result<(), String> {
    match tool.as_str() {
        "ccswitch" => {
            #[cfg(target_os = "windows")]
            {
                if let Some(path) = find_ccswitch_path() {
                    Command::new("cmd")
                        .args(["/C", "start", "", &path])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn()
                        .map_err(|e| format!("无法打开 CC-Switch: {}", e))?;
                } else {
                    return Err("未找到 CC-Switch，请确认已安装".to_string());
                }
            }
            return Ok(());
        }
        _ => {}
    }

    let cmd = match tool.as_str() {
        "claude" => "claude",
        "codex" => "codex",
        "gemini" => "gemini",
        "opencode" => "opencode",
        "openclaw" => "openclaw",
        "hermes" => "hermes chat",
        "mimo" => "mimo",
        "kilocode" => "kilocode",
        _ => return Err(format!("未知工具: {}", tool)),
    };

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("无法打开工具: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("x-terminal-emulator")
            .args(["-e", cmd])
            .spawn()
            .map_err(|e| format!("无法打开工具: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn uninstall_tool(tool: String) -> Result<InstallResult, String> {
    tokio::task::spawn_blocking(move || {
        if tool == "hermes" {
            return match uninstall_hermes() {
                Ok(()) => Ok(InstallResult {
                    success: true,
                    error: None,
                }),
                Err(e) => Ok(InstallResult {
                    success: false,
                    error: Some(e),
                }),
            };
        }

        if tool == "ccswitch" {
            return match uninstall_ccswitch() {
                Ok(()) => Ok(InstallResult {
                    success: true,
                    error: None,
                }),
                Err(e) => Ok(InstallResult {
                    success: false,
                    error: Some(e),
                }),
            };
        }

        let package = match tool.as_str() {
            "claude" => "@anthropic-ai/claude-code",
            "codex" => "@openai/codex",
            "gemini" => "@google/gemini-cli",
            "opencode" => "opencode-ai",
            "openclaw" => "openclaw",
            "mimo" => "@mimo-ai/cli",
            "kilocode" => "@kilocode/cli",
            _ => return Err(format!("未知工具: {}", tool)),
        };

        match uninstall_npm_package(package) {
            Ok(()) => Ok(InstallResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(InstallResult {
                success: false,
                error: Some(e),
            }),
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
