use crate::error::{SanctumError, Result};
use crate::types::{RpcRequest, RpcResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Encode a JSON-RPC request into length-prefixed wire format.
pub fn encode(request: &RpcRequest) -> Result<Vec<u8>> {
    let payload = serde_json::to_vec(request)?;
    let len = payload.len() as u32;
    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

/// Decode a length-prefixed JSON-RPC response from raw bytes.
pub fn decode(data: &[u8]) -> Result<RpcResponse> {
    if data.len() < 4 {
        return Err(SanctumError::Protocol("frame too short".into()));
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len {
        return Err(SanctumError::Protocol("incomplete frame".into()));
    }
    let response: RpcResponse = serde_json::from_slice(&data[4..4 + len])?;
    Ok(response)
}

/// Write a length-prefixed frame to an async writer.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, request: &RpcRequest) -> Result<()> {
    let buf = encode(request)?;
    writer.write_all(&buf).await?;
    writer.flush().await?;
    Ok(())
}

/// Read a length-prefixed frame from an async reader.
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<RpcResponse> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 16 * 1024 * 1024 {
        return Err(SanctumError::Protocol(format!("frame too large: {len} bytes")));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    let response: RpcResponse = serde_json::from_slice(&payload)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encode_decode_roundtrip() {
        let req = RpcRequest {
            id: 1,
            method: "test".into(),
            params: json!({"key": "value"}),
        };
        let encoded = encode(&req).unwrap();
        assert_eq!(&encoded[..4], &(encoded.len() as u32 - 4).to_be_bytes());

        // Decode as response (same JSON structure test)
        let resp_json = serde_json::json!({"id": 1, "result": {"ok": true}});
        let resp_bytes = serde_json::to_vec(&resp_json).unwrap();
        let mut frame = (resp_bytes.len() as u32).to_be_bytes().to_vec();
        frame.extend_from_slice(&resp_bytes);
        let decoded = decode(&frame).unwrap();
        assert_eq!(decoded.id, 1);
        assert!(decoded.result.is_some());
    }
}
