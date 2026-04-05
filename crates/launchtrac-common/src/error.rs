use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaunchTracError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Camera error: {0}")]
    Camera(String),

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Vision pipeline error: {0}")]
    Vision(String),

    #[error("Simulator connection error: {0}")]
    Simulator(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
