pub mod client;
pub mod server;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// The request trait.
pub trait Request {
    type Params: DeserializeOwned + Serialize + Send + Sync + 'static;
    type Result: DeserializeOwned + Serialize + Send + Sync + 'static;
    const METHOD: &'static str;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcRequest {
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RpcResponseKind {
    Ok { result: serde_json::Value },
    Err { error: RpcError },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcResponse {
    #[serde(flatten)]
    pub kind: RpcResponseKind,
}

/// The handshake request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Handshake {}

impl Request for Handshake {
    type Params = HandeshakeParams;
    type Result = HandeshakeResult;
    const METHOD: &'static str = "handshake";
}

/// Parameters for the handshake request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandeshakeParams {
    pub pid: u32,
}

/// Result for the handshake request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandeshakeResult {
    /// Running in root privilege.
    pub privilige: bool,
}

/// The update request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {}

impl Request for Update {
    type Params = UpdateParams;
    type Result = UpdateResult;
    const METHOD: &'static str = "update";
}

/// Parameters for the update request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParams {
    /// The name of backend.
    pub backend_name: String,
}

/// Result for the update request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {}

/// The outdated request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outdated {}

impl Request for Outdated {
    type Params = OutdatedParams;
    type Result = OutdatedResult;
    const METHOD: &'static str = "outdated";
}

/// Parameters for the outdated request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedParams {
    /// The name of backend.
    pub backend_name: String,
}

/// Result for the outdated request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedResult {
    /// The outdated packages.
    pub pkgs: Vec<OutdateItem>,
}

/// Outdated item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdateItem {
    /// The name of package.
    pub name: String,
    /// The vendor of package.
    pub vendor: String,
    /// The current version of package.
    pub current_version: String,
    /// The target version of package.
    pub target_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Upgrade {}

impl Request for Upgrade {
    type Params = UpgradeParams;
    type Result = UpgradeResult;
    const METHOD: &'static str = "upgrade";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeParams {
    pub backend_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {}
