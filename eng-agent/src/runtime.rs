use std::path::PathBuf;
use std::fs;
use std::io::Write;

#[derive(Debug)]
pub struct RuntimeInfo {
    pub port: u16,
    pub token: String,
}

pub fn get_runtime_dir() -> PathBuf {
    // ユーザー単位のディレクトリ: /tmp/eng-agent-<USER>/
    // Windows対応なども本来は必要だが、まずはUnix系想定
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".into());
    let dir = std::env::temp_dir().join(format!("eng-agent-{}", user));
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }
    dir
}

pub fn get_port_file_path() -> PathBuf {
    get_runtime_dir().join("agent.port")
}

pub fn save_runtime_info(port: u16, token: &str) -> std::io::Result<()> {
    let path = get_port_file_path();
    // 他ユーザーから読めないようにパーミッション設定すべきだが、
    // std::fs::File::create は umask に従う。
    // 本番では std::os::unix::fs::OpenOptionsExt で mode(0o600) を指定すべき。
    let content = format!("{}\n{}", port, token);
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn load_runtime_info() -> Option<RuntimeInfo> {
    let path = get_port_file_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() >= 2 {
        let port = lines[0].parse().ok()?;
        let token = lines[1].to_string();
        Some(RuntimeInfo { port, token })
    } else {
        None
    }
}

pub fn cleanup_runtime_info() {
    let path = get_port_file_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}
