use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum NookError {
    #[error("Docker connection failed: {0}")]
    DockerConnection(String),

    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    #[error("devcontainer CLI not found")]
    DevcontainerCliNotFound,

    #[error("devcontainer up failed: {0}")]
    DevcontainerUpFailed(String),

    #[error("Host port {0} is already in use")]
    HostPortBusy(u16),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Terminal not found")]
    TerminalNotFound,

    #[error("Pod is busy with another operation")]
    PodBusy(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Docker API error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for NookError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type NookResult<T> = Result<T, NookError>;
