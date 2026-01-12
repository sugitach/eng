pub mod auth;

// 自動生成されたコードをインポート
pub mod editor {
    tonic::include_proto!("editor.v1");
}
use editor::{
    editor_service_server::EditorService,
    HandshakeRequest, HandshakeResponse,
};
use tokio_stream::{Stream, StreamExt};
use tonic::Status;

#[derive(Debug, Default)]
pub struct MyEditorService {}

// ロジックを分離してテスト可能にする
fn handle_handshake_logic<S>(mut in_stream: S) -> impl Stream<Item = Result<HandshakeResponse, Status>>
where
    S: Stream<Item = Result<HandshakeRequest, Status>> + Unpin + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);

    tokio::spawn(async move {
        while let Some(result) = in_stream.next().await {
            match result {
                Ok(req) => {
                    let response = HandshakeResponse {
                        server_message: format!("Echo: {}", req.client_message),
                    };
                    if tx.send(Ok(response)).await.is_err() {
                        break;
                    }
                }
                Err(status) => {
                    eprintln!("Handshake stream error: {:?}", status);
                    break;
                }
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tonic::async_trait]
impl EditorService for MyEditorService {
    type HandshakeStream = std::pin::Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send + Sync + 'static>>;

    async fn handshake(
        &self,
        request: tonic::Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<tonic::Response<Self::HandshakeStream>, Status> {
        let in_stream = request.into_inner();
        let out_stream = handle_handshake_logic(in_stream);
        Ok(tonic::Response::new(Box::pin(out_stream)))
    }
}


#[cfg(test)]
mod tests {
    use super::auth::*;
    use tokio::net::TcpListener;
    use tonic::Status;
    use tokio_stream::{Stream, StreamExt};
    use tokio::sync::mpsc;
    use crate::editor::HandshakeRequest;
    use crate::handle_handshake_logic;

    // モックのストリームを作成するためのヘルパー関数
    fn create_mock_request_stream(messages: Vec<String>) -> impl Stream<Item = Result<HandshakeRequest, Status>> {
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            for msg in messages {
                tx.send(HandshakeRequest { client_message: msg }).await.unwrap();
            }
        });
        tokio_stream::wrappers::ReceiverStream::new(rx).map(Ok)
    }

    #[test]
    fn test_validate_auth_token_valid() {
        let token = "valid_token";
        assert!(validate_auth_token(token));
    }

    #[test]
    fn test_validate_auth_token_empty() {
        let token = "";
        assert!(!validate_auth_token(token));
    }

    #[tokio::test]
    async fn test_dynamic_port_assignment() -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("[::1]:0").await?;
        let port = listener.local_addr()?.port();
        assert_ne!(port, 0, "割り当てられたポートは0であってはならない");
        println!("動的に割り当てられたポート: {}", port);
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_dynamic_port_assignments_are_unique() -> Result<(), Box<dyn std::error::Error>> {
        let listener1 = TcpListener::bind("[::1]:0").await?;
        let port1 = listener1.local_addr()?.port();

        let listener2 = TcpListener::bind("[::1]:0").await?;
        let port2 = listener2.local_addr()?.port();

        assert_ne!(port1, 0, "最初の割り当てられたポートは0であってはならない");
        assert_ne!(port2, 0, "2番目の割り当てられたポートは0であってはならない");
        assert_ne!(port1, port2, "動的に割り当てられたポートはユニークであるべき");

        println!("動的に割り当てられたポート1: {}", port1);
        println!("動的に割り当てられたポート2: {}", port2);
        Ok(())
    }

    #[tokio::test]
    async fn test_handshake_server_logic() {
        // サービス経由ではなくロジック関数を直接テストする
        let client_messages = vec!["Hello".to_string(), "World".to_string()];
        let request_stream = create_mock_request_stream(client_messages.clone());

        let mut response_stream = handle_handshake_logic(request_stream);

        // 最初のメッセージを検証
        let first_response = response_stream.next().await.unwrap().unwrap();
        assert_eq!(first_response.server_message, "Echo: Hello".to_string());
        
        // 2番目のメッセージを検証
        let second_response = response_stream.next().await.unwrap().unwrap();
        assert_eq!(second_response.server_message, "Echo: World".to_string());

        // ストリームが終了したことを確認
        assert!(response_stream.next().await.is_none());
    }
}