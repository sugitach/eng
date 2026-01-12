use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug)]
pub struct CoreProcess {
    // 将来的にプロセスハンドルが必要になったときのために保持
    // kill_on_drop(true) を設定しているので、保持しているだけで寿命管理になる
    pub child: Child,
    pub port: u16,
}

pub struct LocalLauncher {
    token: String,
}

impl LocalLauncher {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    /// eng-core バイナリのパスを解決する
    fn resolve_binary_path() -> Result<PathBuf, String> {
        // 1. 環境変数 ENG_CORE_PATH を優先
        if let Ok(path_str) = std::env::var("ENG_CORE_PATH") {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Ok(path);
            }
            eprintln!("Warning: ENG_CORE_PATH is set but file not found: {:?}", path);
        }

        // 2. 現在の実行ファイルの隣を探す
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let names = if cfg!(windows) {
                    vec!["eng-core.exe"]
                } else {
                    vec!["eng-core"]
                };

                for name in names {
                    let candidate = parent.join(name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        // 3. 開発時のディレクトリ構造を探索 (ワークスペースルート/target/debug/eng-core)
        // cargo run で実行した場合、current_exe は target/debug/deps/ui-xxxx になることがあるため、
        // 親の親などを探す必要があるかもしれないが、通常は同じ target/debug に出力される。

        Err("eng-core binary not found. Please set ENG_CORE_PATH or place it alongside the ui binary.".to_string())
    }

    pub async fn launch(&self) -> Result<CoreProcess, Box<dyn std::error::Error>> {
        let binary_path = Self::resolve_binary_path()?;
        println!("UI: Launching core from {:?}", binary_path);

        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true) // UIプロセス終了時にCoreも終了させる
            .spawn()?;

        // トークン送信
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(self.token.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }

        // ポート受信
        let mut port = 0;
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            // Coreはポート番号の後に改行を出力する
            if reader.read_line(&mut line).await? > 0 {
                port = line.trim().parse().unwrap_or(0);
            }
        }

        if port == 0 {
            // 失敗時はプロセスを殺す
            let _ = child.kill().await; 
            return Err("Failed to retrieve port from core process".into());
        }

        Ok(CoreProcess { child, port })
    }
}
