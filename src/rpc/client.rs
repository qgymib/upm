pub struct Client {
    stream: std::net::TcpStream,
}

impl Client {
    /// Create a new session client on the given stream.
    ///
    /// # Arguments
    /// + `stream` - The stream of the client.
    ///
    /// # Returns
    /// The session client.
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self { stream }
    }

    /// Call the request.
    ///
    /// # Arguments
    /// + `req` - The request.
    ///
    /// # Returns
    /// The result of the request.
    pub fn call<R>(&mut self, req: &R::Params) -> anyhow::Result<R::Result>
    where
        R: super::Request,
    {
        use std::io::{Read, Write};

        // Send 4 bytes protocol magic header.
        {
            let magic = "upm:";
            self.stream.write_all(magic.as_bytes())?;
        }

        // Send payload.
        {
            let msg = super::RpcRequest {
                method: R::METHOD.into(),
                params: Some(serde_json::to_value(req)?),
            };

            let data = serde_json::to_string(&msg)?;
            let len = data.len() as u32;
            let hdr = len.to_be_bytes();
            self.stream.write_all(&hdr)?;
            self.stream.write_all(data.as_bytes())?;
        }

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
        let mut data = vec![0u8; payload_len];
        self.stream.read_exact(&mut data)?;

        let rsp: R::Result = serde_json::from_slice(&data)?;
        Ok(rsp)
    }

    /// Shutdown the client.
    ///
    /// # Returns
    /// `Ok(())` if the shutdown is successful, otherwise `Err(std::io::Error)`.
    pub fn shutdown(&self) -> anyhow::Result<()> {
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }
}
