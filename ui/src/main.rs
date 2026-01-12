use iced::{executor, Application, Command, Element, Settings, Theme, Length};
use iced::widget::{column, text, container, scrollable};
use iced::window;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Request;
use tonic::metadata::MetadataValue;
use uuid::Uuid;
use tokio_stream::StreamExt;
use clap::Parser;

mod launcher;
use launcher::{LocalLauncher, CoreProcess};

pub mod editor {
    tonic::include_proto!("editor.v1");
}
use editor::editor_service_client::EditorServiceClient;
use editor::HandshakeRequest;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in test mode (auto-close after handshake)
    #[arg(long)]
    test_mode: bool,
}

pub fn generate_auth_token() -> String {
    Uuid::new_v4().to_string()
}

// アプリケーションの状態
struct EditorApp {
    logs: Vec<String>,
    test_mode: bool,
    _core_process: Option<Arc<Mutex<CoreProcess>>>, 
}

#[derive(Debug, Clone)]
enum Message {
    CoreHandshakeFinished(Arc<Mutex<CoreProcess>>, Vec<String>),
    Error(String),
    CloseRequested,
}

impl Application for EditorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Args;

    fn new(args: Args) -> (Self, Command<Message>) {
        (
            EditorApp {
                logs: vec![
                    "Initializing UI...".into(), 
                    format!("Test Mode: {}", args.test_mode),
                    "Launching Core process...".into()
                ],
                test_mode: args.test_mode,
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
                
                if self.test_mode {
                    self.logs.push("Test mode enabled. Closing window in 1 second...".into());
                    // テストモード時は少し待ってから閉じる
                    return Command::perform(
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)),
                        |_| Message::CloseRequested
                    );
                }
                Command::none()
            }
            Message::Error(e) => {
                self.logs.push(format!("Error: {}", e));
                if self.test_mode {
                    eprintln!("Test failed with error: {}", e);
                    return window::close(window::Id::MAIN);
                }
                Command::none()
            }
            Message::CloseRequested => {
                window::close(window::Id::MAIN)
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

async fn start_core_and_handshake() -> Result<(Arc<Mutex<CoreProcess>>, Vec<String>), String> {
    let token = generate_auth_token();
    let launcher = LocalLauncher::new(token.clone());

    let core_process = launcher.launch().await.map_err(|e| e.to_string())?;
    let port = core_process.port;
    let process_arc = Arc::new(Mutex::new(core_process));

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

    let outbound = tokio_stream::iter(vec![
        HandshakeRequest { client_message: "Hello from GUI!".into() },
        HandshakeRequest { client_message: "GUI is running in test mode!".into() },
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
    let args = Args::parse();
    
    let mut settings = Settings::with_flags(args);
    // ウィンドウサイズなどのデフォルト設定が必要な場合はここで調整
    
    EditorApp::run(settings)
}
