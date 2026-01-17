use iced::{executor, Application, Command, Element, Settings, Theme, Length};
use iced::widget::{column, text, container, scrollable};
use iced::window;
use tonic::transport::Channel;
use tonic::Request;
use tonic::metadata::MetadataValue;
use tokio_stream::StreamExt;
use clap::Parser;

pub mod editor {
    tonic::include_proto!("editor.v1");
}
use editor::editor_service_client::EditorServiceClient;
use editor::HandshakeRequest;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Agent port to connect to
    #[arg(long)]
    agent_port: Option<u16>,

    /// Agent token for authentication
    #[arg(long)]
    agent_token: Option<String>,

    /// Run in test mode (auto-close after handshake)
    #[arg(long)]
    test_mode: bool,
}

// アプリケーションの状態
struct EditorApp {
    logs: Vec<String>,
    test_mode: bool,
}

#[derive(Debug, Clone)]
enum Message {
    HandshakeFinished(Vec<String>),
    Error(String),
    CloseRequested,
}

impl Application for EditorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Args;

    fn new(args: Args) -> (Self, Command<Message>) {
        let mut logs = vec!["Initializing UI...".into()];
        
        let command = if let (Some(port), Some(token)) = (args.agent_port, args.agent_token) {
            logs.push(format!("Connecting to Agent on port {}...", port));
            Command::perform(connect_to_agent_and_handshake(port, token), |res| match res {
                Ok(logs) => Message::HandshakeFinished(logs),
                Err(e) => Message::Error(e),
            })
        } else {
            logs.push("Error: Agent port or token not provided.".into());
            Command::none()
        };

        (
            EditorApp {
                logs,
                test_mode: args.test_mode,
            },
            command
        )
    }

    fn title(&self) -> String {
        String::from("Eng Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::HandshakeFinished(logs) => {
                self.logs.push("Handshake with Agent successful.".into());
                self.logs.extend(logs);
                
                if self.test_mode {
                    self.logs.push("Test mode enabled. Closing window...".into());
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
                    std::process::exit(1);
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

async fn connect_to_agent_and_handshake(port: u16, token: String) -> Result<Vec<String>, String> {
    // 接続待ち（AgentがgRPCサーバーを起動する猶予）
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let uri = format!("http://[::1]:{}", port);
    let channel = Channel::from_shared(uri).map_err(|e| e.to_string())?
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(std::time::Duration::from_secs(10))
        .keep_alive_timeout(std::time::Duration::from_secs(5))
        .connect()
        .await.map_err(|e| format!("Failed to connect to agent: {}", e))?;

    let token_metadata: MetadataValue<_> = token.parse().unwrap();
    let mut client = EditorServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token_metadata.clone());
        Ok(req)
    });

    let outbound = tokio_stream::iter(vec![
        HandshakeRequest { client_message: "Hello from UI to Agent!".into() },
    ]);

    let response = client.handshake(Request::new(outbound)).await.map_err(|e| format!("RPC failed: {}", e))?;
    let mut inbound = response.into_inner();

    let mut logs = Vec::new();
    while let Some(resp) = inbound.next().await {
        match resp {
            Ok(msg) => logs.push(format!("Agent: {}", msg.server_message)),
            Err(e) => logs.push(format!("RPC Error: {}", e)),
        }
    }

    Ok(logs)
}

fn main() -> iced::Result {
    let args = Args::parse();
    let settings = Settings::with_flags(args);
    EditorApp::run(settings)
}
