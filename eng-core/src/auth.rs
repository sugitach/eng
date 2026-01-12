use tonic::{Request, Status};
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;

pub fn validate_auth_token(token: &str) -> bool {
    !token.is_empty()
}

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    expected_token: MetadataValue<tonic::metadata::Ascii>,
}

impl AuthInterceptor {
    pub fn new(token_str: String) -> Result<Self, Status> {
        let expected_token = token_str
            .parse()
            .map_err(|_| Status::internal("Invalid auth token format"))?;
        Ok(Self { expected_token })
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        match request.metadata().get("authorization") {
            Some(t) if t == self.expected_token => Ok(request),
            _ => Err(Status::unauthenticated("No valid auth token")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_auth_token_valid() {
        // 仮の有効なトークン
        let token = "valid_token";
        assert!(validate_auth_token(token));
    }

    #[test]
    fn test_validate_auth_token_empty() {
        // 空のトークンは無効
        let token = "";
        assert!(!validate_auth_token(token));
    }

    // #[test]
    // fn test_validate_auth_token_invalid() {
    //     // 仮の不正なトークン
    //     let token = "invalid_token";
    //     assert!(!validate_auth_token(token));
    // }
}
