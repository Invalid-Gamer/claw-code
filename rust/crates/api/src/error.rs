use std::env::VarError;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug)]
pub enum ApiError {
    MissingApiKey,
    ExpiredOAuthToken,
    Auth(String),
    InvalidApiKeyEnv(VarError),
    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Api {
        status: reqwest::StatusCode,
        error_type: Option<String>,
        message: Option<String>,
        body: String,
        retryable: bool,
    },
    RetriesExhausted {
        attempts: u32,
        last_error: Box<ApiError>,
    },
    InvalidSseFrame(&'static str),
    BackoffOverflow {
        attempt: u32,
        base_delay: Duration,
    },
}

impl ApiError {
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(error) => error.is_connect() || error.is_timeout() || error.is_request(),
            Self::Api { retryable, .. } => *retryable,
            Self::RetriesExhausted { last_error, .. } => last_error.is_retryable(),
            Self::MissingApiKey
            | Self::ExpiredOAuthToken
            | Self::Auth(_)
            | Self::InvalidApiKeyEnv(_)
            | Self::Io(_)
            | Self::Json(_)
            | Self::InvalidSseFrame(_)
            | Self::BackoffOverflow { .. } => false,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingApiKey => {
                // Intentionally explicit about what this branch does and
                // does not support so users who exported OPENAI_API_KEY,
                // XAI_API_KEY, DASHSCOPE_API_KEY, AWS_*, or Google
                // service-account credentials get an immediate "aha"
                // instead of assuming a misconfiguration bug. See
                // rust/README.md § "Providers & Auth Support Matrix"
                // for the full matrix and the branch differences.
                write!(
                    f,
                    "ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is not set; export one before calling the Anthropic API. \
                     On this branch (`dev/rust`) only Anthropic is wired up \
                     — OPENAI_API_KEY, XAI_API_KEY, DASHSCOPE_API_KEY, and \
                     AWS/Google credentials are ignored. Multi-provider \
                     routing (OpenAI, xAI, DashScope) lives on `main`; AWS \
                     Bedrock, Google Vertex AI, and Azure OpenAI are not \
                     supported on any branch yet. See rust/README.md \
                     § 'Providers & Auth Support Matrix' for details."
                )
            }
            Self::ExpiredOAuthToken => {
                write!(
                    f,
                    "saved OAuth token is expired and no refresh token is available"
                )
            }
            Self::Auth(message) => write!(f, "auth error: {message}"),
            Self::InvalidApiKeyEnv(error) => {
                write!(
                    f,
                    "failed to read ANTHROPIC_AUTH_TOKEN / ANTHROPIC_API_KEY: {error}"
                )
            }
            Self::Http(error) => write!(f, "http error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Api {
                status,
                error_type,
                message,
                body,
                ..
            } => match (error_type, message) {
                (Some(error_type), Some(message)) => {
                    write!(
                        f,
                        "anthropic api returned {status} ({error_type}): {message}"
                    )
                }
                _ => write!(f, "anthropic api returned {status}: {body}"),
            },
            Self::RetriesExhausted {
                attempts,
                last_error,
            } => write!(
                f,
                "anthropic api failed after {attempts} attempts: {last_error}"
            ),
            Self::InvalidSseFrame(message) => write!(f, "invalid sse frame: {message}"),
            Self::BackoffOverflow {
                attempt,
                base_delay,
            } => write!(
                f,
                "retry backoff overflowed on attempt {attempt} with base delay {base_delay:?}"
            ),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<VarError> for ApiError {
    fn from(value: VarError) -> Self {
        Self::InvalidApiKeyEnv(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_api_key_display_lists_supported_and_unsupported_providers_and_points_at_readme() {
        // given
        let error = ApiError::MissingApiKey;

        // when
        let rendered = format!("{error}");

        // then — the message must keep the grep-stable core so CI
        // parsers and docs that quote the exact substring continue to
        // resolve, AND it must tell the user which env vars are
        // ignored on this branch and where to find the full matrix.
        assert!(
            rendered.contains("ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is not set"),
            "grep-stable prefix must remain intact, got: {rendered}"
        );
        assert!(
            rendered.contains("OPENAI_API_KEY"),
            "should explicitly call out that OPENAI_API_KEY is ignored on dev/rust, got: {rendered}"
        );
        assert!(
            rendered.contains("XAI_API_KEY"),
            "should explicitly call out that XAI_API_KEY is ignored on dev/rust, got: {rendered}"
        );
        assert!(
            rendered.contains("DASHSCOPE_API_KEY"),
            "should explicitly call out that DASHSCOPE_API_KEY is ignored on dev/rust, got: {rendered}"
        );
        assert!(
            rendered.contains("Bedrock") && rendered.contains("Vertex") && rendered.contains("Azure"),
            "should tell users Bedrock/Vertex/Azure are not supported on any branch, got: {rendered}"
        );
        assert!(
            rendered.contains("rust/README.md"),
            "should point users at the README matrix for the full story, got: {rendered}"
        );
    }
}
