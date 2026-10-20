#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError {
    Unavailable,
    Io(String),
    Malformed(String),
    Closed,
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Unavailable => write!(formatter, "o sparky não está rodando"),
            IpcError::Io(message) => write!(formatter, "falha no socket do sparky: {message}"),
            IpcError::Malformed(message) => write!(formatter, "mensagem inválida: {message}"),
            IpcError::Closed => write!(formatter, "o sparky fechou a conexão"),
        }
    }
}

impl std::error::Error for IpcError {}
