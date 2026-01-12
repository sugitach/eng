use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use eng_core::auth::AuthInterceptor;
use eng_core::editor::editor_service_server::EditorServiceServer;
use eng_core::MyEditorService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 認証トークンを標準入力から読み込む
    let mut auth_token = String::new();
    std::io::stdin().read_line(&mut auth_token)?;
    let auth_token = auth_token.trim().to_string();

    if auth_token.is_empty() {
        eprintln!("Error: Auth token not provided via stdin.");
        return Err("Auth token required".into());
    }

    // AuthInterceptorの初期化
    let interceptor = AuthInterceptor::new(auth_token)?;

    let addr: SocketAddr = "[::1]:0".parse()?; // 動的にポートを割り当てる
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    
    // ポート番号のみを標準出力に書き出す（UIプロセスがこれを読み取る）
    println!("{}", local_addr.port());
    
    eprintln!("Core server listening on {}", local_addr);

    let incoming = TcpListenerStream::new(listener);
    let service = MyEditorService::default();

    Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)))
        .add_service(EditorServiceServer::with_interceptor(service, interceptor))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}