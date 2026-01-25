//! Authentication provider hooks for MCP servers.
//!
//! Auth providers are transport-agnostic and operate on the JSON-RPC
//! request payload. They may populate [`AuthContext`] to be stored in
//! session state for downstream handlers.

use std::collections::HashMap;
use std::sync::Arc;

use fastmcp_core::{AccessToken, AuthContext, McpContext, McpError, McpErrorCode, McpResult};

/// Authentication request view used by providers.
#[derive(Debug, Clone, Copy)]
pub struct AuthRequest<'a> {
    /// JSON-RPC method name.
    pub method: &'a str,
    /// Raw params payload (if present).
    pub params: Option<&'a serde_json::Value>,
    /// Internal request ID (u64) used for tracing.
    pub request_id: u64,
}

impl AuthRequest<'_> {
    /// Attempts to extract an access token from the raw request params.
    #[must_use]
    pub fn access_token(&self) -> Option<AccessToken> {
        extract_access_token(self.params)
    }
}

/// Extracts an access token from request params using common field names.
fn extract_access_token(params: Option<&serde_json::Value>) -> Option<AccessToken> {
    let params = params?;
    match params {
        serde_json::Value::String(value) => AccessToken::parse(value),
        serde_json::Value::Object(map) => {
            if let Some(token) = extract_from_map(map) {
                return Some(token);
            }
            if let Some(meta) = map.get("_meta").and_then(serde_json::Value::as_object) {
                if let Some(token) = extract_from_map(meta) {
                    return Some(token);
                }
            }
            if let Some(headers) = map.get("headers").and_then(serde_json::Value::as_object) {
                if let Some(token) = extract_from_map(headers) {
                    return Some(token);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_from_map(map: &serde_json::Map<String, serde_json::Value>) -> Option<AccessToken> {
    for key in [
        "authorization",
        "Authorization",
        "auth",
        "token",
        "access_token",
        "accessToken",
    ] {
        if let Some(value) = map.get(key) {
            if let Some(token) = extract_from_value(value) {
                return Some(token);
            }
        }
    }
    None
}

fn extract_from_value(value: &serde_json::Value) -> Option<AccessToken> {
    match value {
        serde_json::Value::String(value) => AccessToken::parse(value),
        serde_json::Value::Object(map) => {
            if let (Some(scheme), Some(token)) = (
                map.get("scheme").and_then(serde_json::Value::as_str),
                map.get("token").and_then(serde_json::Value::as_str),
            ) {
                if !scheme.trim().is_empty() && !token.trim().is_empty() {
                    return Some(AccessToken {
                        scheme: scheme.trim().to_string(),
                        token: token.trim().to_string(),
                    });
                }
            }
            for key in ["authorization", "token", "access_token", "accessToken"] {
                if let Some(value) = map.get(key).and_then(serde_json::Value::as_str) {
                    if let Some(token) = AccessToken::parse(value) {
                        return Some(token);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Authentication provider interface.
///
/// Implementations decide whether a request is allowed and may return
/// an [`AuthContext`] describing the authenticated subject.
pub trait AuthProvider: Send + Sync {
    /// Authenticate an incoming request.
    ///
    /// Return `Ok(AuthContext)` to allow, or an `Err(McpError)` to deny.
    fn authenticate(&self, ctx: &McpContext, request: AuthRequest<'_>) -> McpResult<AuthContext>;
}

/// Token verifier interface used by token-based auth providers.
pub trait TokenVerifier: Send + Sync {
    /// Verify an access token and return an auth context if valid.
    fn verify(
        &self,
        ctx: &McpContext,
        request: AuthRequest<'_>,
        token: &AccessToken,
    ) -> McpResult<AuthContext>;
}

/// Token-based authentication provider.
#[derive(Clone)]
pub struct TokenAuthProvider {
    verifier: Arc<dyn TokenVerifier>,
    missing_token_error: McpError,
}

impl TokenAuthProvider {
    /// Creates a new token auth provider with the given verifier.
    #[must_use]
    pub fn new<V: TokenVerifier + 'static>(verifier: V) -> Self {
        Self {
            verifier: Arc::new(verifier),
            missing_token_error: auth_error("Missing access token"),
        }
    }

    /// Overrides the error returned when a token is missing.
    #[must_use]
    pub fn with_missing_token_error(mut self, error: McpError) -> Self {
        self.missing_token_error = error;
        self
    }
}

impl AuthProvider for TokenAuthProvider {
    fn authenticate(&self, ctx: &McpContext, request: AuthRequest<'_>) -> McpResult<AuthContext> {
        let access = request
            .access_token()
            .ok_or_else(|| self.missing_token_error.clone())?;
        self.verifier.verify(ctx, request, &access)
    }
}

/// Static token verifier backed by an in-memory token map.
#[derive(Debug, Clone)]
pub struct StaticTokenVerifier {
    tokens: HashMap<String, AuthContext>,
    allowed_schemes: Option<Vec<String>>,
}

impl StaticTokenVerifier {
    /// Creates a new static verifier from a token → context map.
    pub fn new<I, K>(tokens: I) -> Self
    where
        I: IntoIterator<Item = (K, AuthContext)>,
        K: Into<String>,
    {
        let tokens = tokens
            .into_iter()
            .map(|(token, ctx)| (token.into(), ctx))
            .collect();
        Self {
            tokens,
            allowed_schemes: None,
        }
    }

    /// Restricts accepted token schemes (case-insensitive).
    #[must_use]
    pub fn with_allowed_schemes<I, S>(mut self, schemes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_schemes = Some(schemes.into_iter().map(Into::into).collect());
        self
    }
}

impl TokenVerifier for StaticTokenVerifier {
    fn verify(
        &self,
        _ctx: &McpContext,
        _request: AuthRequest<'_>,
        token: &AccessToken,
    ) -> McpResult<AuthContext> {
        if let Some(allowed) = &self.allowed_schemes {
            if !allowed
                .iter()
                .any(|scheme| scheme.eq_ignore_ascii_case(&token.scheme))
            {
                return Err(auth_error("Unsupported auth scheme"));
            }
        }

        let Some(auth) = self.tokens.get(&token.token) else {
            return Err(auth_error("Invalid access token"));
        };

        let mut ctx = auth.clone();
        ctx.token.get_or_insert_with(|| token.clone());
        Ok(ctx)
    }
}

fn auth_error(message: impl Into<String>) -> McpError {
    McpError::new(McpErrorCode::ResourceForbidden, message)
}

#[cfg(feature = "jwt")]
mod jwt {
    use super::{AuthContext, AuthRequest, TokenVerifier, auth_error};
    use fastmcp_core::{AccessToken, McpContext, McpResult};
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    /// JWT verifier for HMAC-SHA tokens.
    #[derive(Debug, Clone)]
    pub struct JwtTokenVerifier {
        decoding_key: DecodingKey,
        validation: Validation,
    }

    impl JwtTokenVerifier {
        /// Creates an HS256 verifier from a shared secret.
        #[must_use]
        pub fn hs256(secret: impl AsRef<[u8]>) -> Self {
            Self {
                decoding_key: DecodingKey::from_secret(secret.as_ref()),
                validation: Validation::new(Algorithm::HS256),
            }
        }

        /// Overrides the JWT validation settings.
        #[must_use]
        pub fn with_validation(mut self, validation: Validation) -> Self {
            self.validation = validation;
            self
        }
    }

    impl TokenVerifier for JwtTokenVerifier {
        fn verify(
            &self,
            _ctx: &McpContext,
            _request: AuthRequest<'_>,
            token: &AccessToken,
        ) -> McpResult<AuthContext> {
            if !token.scheme.eq_ignore_ascii_case("Bearer") {
                return Err(auth_error("Unsupported auth scheme"));
            }

            let data =
                decode::<serde_json::Value>(&token.token, &self.decoding_key, &self.validation)
                    .map_err(|err| auth_error(format!("Invalid token: {err}")))?;

            let claims = data.claims;
            let subject = claims
                .get("sub")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            let scopes = extract_scopes(&claims);

            Ok(AuthContext {
                subject,
                scopes,
                token: Some(token.clone()),
                claims: Some(claims),
            })
        }
    }

    fn extract_scopes(claims: &serde_json::Value) -> Vec<String> {
        let mut scopes = Vec::new();
        if let Some(scope) = claims.get("scope").and_then(serde_json::Value::as_str) {
            scopes.extend(scope.split_whitespace().map(str::to_string));
        }
        if let Some(list) = claims.get("scopes").and_then(serde_json::Value::as_array) {
            scopes.extend(
                list.iter()
                    .filter_map(|value| value.as_str().map(str::to_string)),
            );
        }
        scopes
    }
}

#[cfg(feature = "jwt")]
pub use jwt::JwtTokenVerifier;

/// Default allow-all provider (returns anonymous auth context).
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllAuthProvider;

impl AuthProvider for AllowAllAuthProvider {
    fn authenticate(&self, _ctx: &McpContext, _request: AuthRequest<'_>) -> McpResult<AuthContext> {
        Ok(AuthContext::anonymous())
    }
}
