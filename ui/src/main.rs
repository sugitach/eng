use iced::{executor, Application, Command, Element, Settings, Theme, Length};
use iced::widget::{column, text, container, scrollable};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Request;
use tonic::metadata::MetadataValue;
use uuid::Uuid;
use tokio_stream::StreamExt;

mod launcher;
use launcher::{LocalLauncher, CoreProcess};

pub mod editor {
    tonic::include_proto!("editor.v1");
}
use editor::editor_service_client::EditorServiceClient;
use editor::HandshakeRequest;

pub fn generate_auth_token() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_auth_token() {
        let token = generate_auth_token();
        assert!(!token.is_empty());
        assert!(uuid::Uuid::parse_str(&token).is_ok());
    }
}

// アプリケーションの状態
struct EditorApp {
    logs: Vec<String>,
    // プロセスハンドルを保持して終了まで生存させる
    _core_process: Option<Arc<Mutex<CoreProcess>>>, 
}

#[derive(Debug, Clone)]
enum Message {
    CoreHandshakeFinished(Arc<Mutex<CoreProcess>>, Vec<String>),
    Error(String),
}

impl Application for EditorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            EditorApp {
                logs: vec!["Initializing UI...".into(), "Launching Core process...".into()],
                _core_process: None,
            },
            Command::perform(start_core_and_handshake(), |res| match res {
                Ok((process, logs)) => Message::CoreHandshakeFinished(process, logs),
                Err(e) => Message::Error(e),
            })
        )
    }

    fn title(&self) -> String {
        String::from("Eng Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CoreHandshakeFinished(process, logs) => {
                self.logs.push("Core process started successfully.".into());
                self.logs.extend(logs);
                self._core_process = Some(process);
                Command::none()
            }
            Message::Error(e) => {
                self.logs.push(format!("Error: {}", e));
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column(
            self.logs.iter().map(|l| text(l).into()).collect::<Vec<_>>()
        ).spacing(5);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

// Core起動とHandshakeを行う非同期関数
async fn start_core_and_handshake() -> Result<(Arc<Mutex<CoreProcess>>, Vec<String>), String> {
    let token = generate_auth_token();
    let launcher = LocalLauncher::new(token.clone());

    let core_process = launcher.launch().await.map_err(|e| e.to_string())?;
    let port = core_process.port;
    let process_arc = Arc::new(Mutex::new(core_process));

    // gRPC接続
    // サーバー起動待ち
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let uri = format!("http://[::1]:{}", port);
    let channel = Channel::from_shared(uri).map_err(|e| e.to_string())?
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(std::time::Duration::from_secs(10))
        .keep_alive_timeout(std::time::Duration::from_secs(5))
        .connect()
        .await.map_err(|e| format!("Failed to connect to core: {}", e))?;

    let token_metadata: MetadataValue<_> = token.parse().unwrap();
    let mut client = EditorServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token_metadata.clone());
        Ok(req)
    });

    // Handshake実行
    let outbound = tokio_stream::iter(vec![
        HandshakeRequest { client_message: "Hello from GUI!".into() },
        HandshakeRequest { client_message: "GUI is running with iced!".into() },
    ]);

    let response = client.handshake(Request::new(outbound)).await.map_err(|e| format!("RPC failed: {}", e))?;
    let mut inbound = response.into_inner();

    let mut logs = Vec::new();
    logs.push(format!("Connected to Core on port {}", port));

    while let Some(resp) = inbound.next().await {
        match resp {
            Ok(msg) => logs.push(format!("Server: {}", msg.server_message)),
            Err(e) => logs.push(format!("RPC Error: {}", e)),
        }
    }

    Ok((process_arc, logs))
}

fn main() -> iced::Result {
    EditorApp::run(Settings::default())
}