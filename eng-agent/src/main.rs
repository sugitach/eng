mod launcher;
use launcher::Launcher;
use uuid::Uuid;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use eng_core::auth::AuthInterceptor;
use eng_core::editor::editor_service_server::{EditorService, EditorServiceServer};
use eng_core::editor::{HandshakeRequest, HandshakeResponse};
use tonic::{Request, Response, Status};
use std::pin::Pin;
use std::net::SocketAddr;
use tokio_stream::{Stream, StreamExt};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct Args {
    /// Run in test mode (propagate to UI)
    #[arg(long)]
    test_mode: bool,
}

#[derive(Debug, Default)]
struct AgentService;

#[tonic::async_trait]
impl EditorService for AgentService {
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
                            server_message: format!("Agent (on behalf of Core): {}", req.client_message),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    eprintln!("Agent: Starting session... (Test Mode: {})", args.test_mode);

    // 1. Core用のトークン生成と起動
    let core_token = Uuid::new_v4().to_string();
    let core_handle = Launcher::launch_core(&core_token).await?;
    eprintln!("Agent: Core launched on port {}", core_handle.port.unwrap());

    // 2. Agentサーバーのポート確保
    let addr: SocketAddr = "[::1]:0".parse()?;
    let listener = TcpListener::bind(addr).await?;
    let agent_addr = listener.local_addr()?;
    let agent_port = agent_addr.port();
    let agent_token = Uuid::new_v4().to_string();
    
    eprintln!("Agent: Listening for UI on {}", agent_addr);

    // 3. UIの起動 (Agentのポートとトークンを渡す)
    let mut ui_handle = Launcher::launch_ui(agent_port, &agent_token, args.test_mode).await?;
    eprintln!("Agent: UI process spawned.");

    // 4. AgentのgRPCサーバー起動
    let interceptor = AuthInterceptor::new(agent_token)?;
    let service = AgentService::default();
    let incoming = TcpListenerStream::new(listener);

    eprintln!("Agent: Session active. Running gRPC server...");

    let server_future = Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)))
        .add_service(EditorServiceServer::with_interceptor(service, interceptor))
        .serve_with_incoming(incoming);

    // 5. 終了待ち (UIプロセス終了 or Ctrl+C or Serverエラー)
    tokio::select! {
        res = server_future => {
            eprintln!("Agent: Server stopped: {:?}", res);
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Agent: Received Ctrl+C.");
        }
        status = ui_handle.child.wait() => {
            eprintln!("Agent: UI process exited with status: {:?}", status);
        }
    }

    eprintln!("Agent: Shutting down...");
    Ok(())
}