use crate::rpc::Request;

pub struct Server {
    stream: std::net::TcpStream,
}

pub trait Router {
    /// Handshake request.
    ///
    /// # Arguments
    /// + `params` - The parameters of the handshake request.
    ///
    /// # Returns
    /// The result of the handshake request.
    fn handshake(&self, params: super::HandeshakeParams)
        -> anyhow::Result<super::HandeshakeResult>;

    /// Update request.
    ///
    /// # Arguments
    /// + `params` - The parameters of the update request.
    ///
    /// # Returns
    /// The result of the update request.
    fn update(&self, params: super::UpdateParams) -> anyhow::Result<super::UpdateResult>;

    /// Get outdated packages.
    ///
    /// # Arguments
    /// + `params` - The parameters of the outdated request.
    ///
    /// # Returns
    /// The result of the outdated request.
    fn outdated(&self, params: super::OutdatedParams) -> anyhow::Result<super::OutdatedResult>;

    /// Upgrade packages.
    ///
    /// # Arguments
    /// + `params` - The parameters of the upgrade request.
    ///
    /// # Returns
    /// The result of the upgrade request.
    fn upgrade(&self, params: super::UpgradeParams) -> anyhow::Result<super::UpgradeResult>;
}

impl Server {
    /// Create a new session server on the given stream.
    ///
    /// # Arguments
    /// + `stream` - The stream of the server.
    ///
    /// # Returns
    /// The session server.
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self { stream }
    }

    /// Serve the request.
    pub fn serve(&mut self, router: &dyn Router) -> anyhow::Result<()> {
        use std::io::{Read, Write};

        // Receive 4 bytes magic header and verify.
        {
            let mut magic = [0; 4];
            self.stream.read_exact(&mut magic)?;
            let magic = std::str::from_utf8(&magic).unwrap();
            if magic != "upm:" {
                return Err(anyhow::anyhow!("invalid magic header."));
            }
        }

        // Receive 4 bytes payload length.
        let payload_len: usize;
        {
            let mut len = [0; 4];
            self.stream.read_exact(&mut len)?;
            payload_len = u32::from_be_bytes(len) as usize;
        }

        // Receive payload.
        let mut payload = vec![0; payload_len];
        self.stream.read_exact(&mut payload)?;
        let payload = std::str::from_utf8(&payload)?;
        let msg: super::RpcRequest = serde_json::from_str(payload)?;

        // Call the request.
        let result = match msg.method.as_str() {
            super::Handshake::METHOD => {
                let params: super::HandeshakeParams = serde_json::from_value(msg.params.unwrap())?;
                let result = router.handshake(params);
                let result = convert_result_to_response::<super::Handshake>(result);
                serde_json::to_value(result)?
            }
            super::Update::METHOD => {
                let params: super::UpdateParams = serde_json::from_value(msg.params.unwrap())?;
                let result = router.update(params);
                let result = convert_result_to_response::<super::Update>(result);
                serde_json::to_value(result)?
            }
            _ => {
                return Err(anyhow::anyhow!("unknown method '{}'.", msg.method));
            }
        };

        // Send 4 bytes protocol magic header.
        {
            let magic = "upm:";
            self.stream.write_all(magic.as_bytes())?;
        }

        // Send payload.
        {
            let data = serde_json::to_string(&result)?;
            let len = data.len() as u32;
            let hdr = len.to_be_bytes();
            self.stream.write_all(&hdr)?;
            self.stream.write_all(data.as_bytes())?;
        }

        Ok(())
    }
}

fn convert_result_to_response<R>(result: anyhow::Result<R::Result>) -> super::RpcResponse
where
    R: super::Request,
{
    match result {
        Ok(result) => super::RpcResponse {
            kind: super::RpcResponseKind::Ok {
                result: serde_json::to_value(result).unwrap(),
            },
        },
        Err(err) => super::RpcResponse {
            kind: super::RpcResponseKind::Err {
                error: super::RpcError {
                    code: 1,
                    message: err.to_string(),
                    data: None,
                },
            },
        },
    }
}
