use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug)]
pub struct ProcessHandle {
    pub child: Child,
    pub port: Option<u16>,
}

pub struct Launcher;

impl Launcher {
    /// バイナリのパスを解決する
    fn resolve_binary_path(name: &str, env_var: &str) -> Result<PathBuf, String> {
        // 1. 環境変数を優先
        if let Ok(path_str) = std::env::var(env_var) {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. 現在の実行ファイルの隣を探す
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let bin_name = if cfg!(windows) {
                    format!("{}.exe", name)
                } else {
                    name.to_string()
                };
                
                let candidate = parent.join(&bin_name);
                if candidate.exists() {
                    return Ok(candidate);
                }
                
                // cargo run 時の target/debug/deps フォールバック
                if let Some(grandparent) = parent.parent() {
                    let candidate = grandparent.join(&bin_name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        Err(format!("Binary '{}' not found. Please set {} or place it in the same directory.", name, env_var))
    }

    /// eng-core を起動する
    pub async fn launch_core(token: &str) -> Result<ProcessHandle, Box<dyn std::error::Error>> {
        let binary_path = Self::resolve_binary_path("eng-core", "ENG_CORE_PATH")?;
        eprintln!("Agent: Launching core from {:?}", binary_path);

        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;

        // トークン送信
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(token.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }

        // ポート受信
        let mut port = 0;
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            if reader.read_line(&mut line).await? > 0 {
                port = line.trim().parse().unwrap_or(0);
            }
        }

        if port == 0 {
            let _ = child.kill().await;
            return Err("Failed to retrieve port from core".into());
        }

        Ok(ProcessHandle { child, port: Some(port) })
    }

    /// eng-ui を起動する
    pub async fn launch_ui(agent_port: u16, agent_token: &str, test_mode: bool) -> Result<ProcessHandle, Box<dyn std::error::Error>> {
        let binary_path = Self::resolve_binary_path("ui", "ENG_UI_PATH")?;
        eprintln!("Agent: Launching UI from {:?}", binary_path);

        // UIには引数で接続情報を渡す
        let mut command = Command::new(binary_path);
        command.arg("--agent-port")
            .arg(agent_port.to_string())
            .arg("--agent-token")
            .arg(agent_token)
            .kill_on_drop(true);

        if test_mode {
            command.arg("--test-mode");
        }

        let child = command.spawn()?;

        Ok(ProcessHandle { child, port: None })
    }
}
