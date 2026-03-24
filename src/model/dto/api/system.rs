use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SetLogLevelDto {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct SetShutdownTimeoutDto {
    pub timeout: u64,
}
