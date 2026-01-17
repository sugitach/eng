mod launcher;
mod runtime;
use launcher::Launcher;
use runtime::{save_runtime_info, load_runtime_info, cleanup_runtime_info};
use uuid::Uuid;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Server, Channel};
use eng_core::auth::AuthInterceptor;
use tonic::{Request, Response, Status};
use std::pin::Pin;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use clap::Parser;

// eng-core で生成されたコードを使用
use eng_core::editor::editor_service_server::{EditorService, EditorServiceServer};
use eng_core::editor::agent_service_server::{AgentService, AgentServiceServer};
use eng_core::editor::agent_service_client::AgentServiceClient;
use eng_core::editor::{HandshakeRequest, HandshakeResponse, SpawnUiRequest, SpawnUiResponse};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct Args {
    /// Run in test mode (propagate to UI)
    #[arg(long)]
    test_mode: bool,
    
    /// Run as daemon (do not exit when UI closes)
    #[arg(long)]
    daemon: bool,
}

#[derive(Debug)]
struct GlobalState {
    agent_port: u16,
    agent_token: String,
    test_mode: bool,
}

#[derive(Debug, Default)]
struct MyEditorServiceImpl;

#[tonic::async_trait]
impl EditorService for MyEditorServiceImpl {
    type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send + Sync + 'static>>;

    async fn handshake(
        &self,
        request: Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(req) => {
                        let response = HandshakeResponse {
                            server_message: format!("Agent Proxy: {}", req.client_message),
                        };
                        if tx.send(Ok(response)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Agent: Handshake error: {:?}", e);
                        break;
                    }
                }
            }
        });

        let out_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream)))
    }
}

#[derive(Debug)]
struct MyAgentServiceImpl {
    state: Arc<GlobalState>,
}

#[tonic::async_trait]
impl AgentService for MyAgentServiceImpl {
    async fn spawn_ui(&self, _request: Request<SpawnUiRequest>) -> Result<Response<SpawnUiResponse>, Status> {
        eprintln!("Agent: Received SpawnUi request.");
        match Launcher::launch_ui(self.state.agent_port, &self.state.agent_token, self.state.test_mode).await {
            Ok(_) => Ok(Response::new(SpawnUiResponse { success: true })),
            Err(e) => Err(Status::internal(format!("Failed to launch UI: {}", e))),
        }
    }
}

async fn try_delegate_to_existing_agent(port: u16, token: String) -> Result<(), Box<dyn std::error::Error>> {
    let uri = format!("http://[::1]:{}", port);
    let channel = Channel::from_shared(uri)?.connect().await?;
    let token_metadata: tonic::metadata::MetadataValue<_> = token.parse()?;

    let mut client = AgentServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token_metadata.clone());
        Ok(req)
    });

    client.spawn_ui(Request::new(SpawnUiRequest {})).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 1. 既存Agentの確認と委譲
    if let Some(info) = load_runtime_info() {
        // ポートファイルがある場合、接続を試みる
        // 接続できれば委譲して終了。できなければ（ゾンビファイルなら）クリーンアップして続行。
        eprintln!("Agent: Found port file (port: {}). Connecting...", info.port);
        match try_delegate_to_existing_agent(info.port, info.token).await {
            Ok(_) => {
                eprintln!("Agent: Delegated to existing agent.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Agent: Connection failed ({}). Cleaning up stale file.", e);
                cleanup_runtime_info();
            }
        }
    }

    // 2. 新規起動 (Server Mode)
    eprintln!("Agent: Starting new session... (Test Mode: {}, Daemon: {})", args.test_mode, args.daemon);

    // Core起動
    let core_token = Uuid::new_v4().to_string();
    let core_handle = Launcher::launch_core(&core_token).await?;
    eprintln!("Agent: Core launched on port {}", core_handle.port.unwrap());

    // Agentサーバーポート確保
    let addr: SocketAddr = "[::1]:0".parse()?;
    let listener = TcpListener::bind(addr).await?;
    let agent_addr = listener.local_addr()?;
    let agent_port = agent_addr.port();
    let agent_token = Uuid::new_v4().to_string();
    
    eprintln!("Agent: Listening on {}", agent_addr);
    save_runtime_info(agent_port, &agent_token)?;

    // UI起動 (初期ウィンドウ)
    let mut ui_handle = Launcher::launch_ui(agent_port, &agent_token, args.test_mode).await?;
    eprintln!("Agent: UI process spawned.");

    // gRPCサーバー構成
    let interceptor = AuthInterceptor::new(agent_token.clone())?;
    let global_state = Arc::new(GlobalState {
        agent_port,
        agent_token,
        test_mode: args.test_mode,
    });

    let editor_service = MyEditorServiceImpl::default();
    let agent_service = MyAgentServiceImpl { state: global_state };
    let incoming = TcpListenerStream::new(listener);

    let server_future = Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)))
        .add_service(EditorServiceServer::with_interceptor(editor_service, interceptor.clone()))
        .add_service(AgentServiceServer::with_interceptor(agent_service, interceptor))
        .serve_with_incoming(incoming);

    eprintln!("Agent: Session active.");

    if args.daemon {
        // Daemonモード: Ctrl+C を待つのみ
        // (UIが終了してもAgentは終了しない)
        tokio::select! {
            res = server_future => eprintln!("Agent: Server error: {:?}", res),
            _ = tokio::signal::ctrl_c() => eprintln!("Agent: Received Ctrl+C."),
        }
    } else {
        // 通常モード: UI終了またはCtrl+Cで終了
        // ※ 複数UIに対応するなら、UIプロセスリストを管理して「0になったら終了」にする必要があるが、
        // 今回は「最初のUI」が閉じたら終了という簡易仕様で進める。
        tokio::select! {
            res = server_future => eprintln!("Agent: Server error: {:?}", res),
            _ = tokio::signal::ctrl_c() => eprintln!("Agent: Received Ctrl+C."),
            status = ui_handle.child.wait() => eprintln!("Agent: Primary UI exited: {:?}", status),
        }
    }

    eprintln!("Agent: Shutting down...");
    cleanup_runtime_info();
    Ok(())
}
