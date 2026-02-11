use sanctum_ai::protocol::{encode, decode};
use sanctum_ai::types::RpcRequest;
use serde_json::json;

#[test]
fn test_length_prefix_encoding() {
    let req = RpcRequest {
        id: 42,
        method: "credential.list".into(),
        params: json!({}),
    };
    let encoded = encode(&req).unwrap();

    // First 4 bytes are big-endian length
    let len = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;
    assert_eq!(len, encoded.len() - 4);

    // Payload is valid JSON
    let payload: serde_json::Value = serde_json::from_slice(&encoded[4..]).unwrap();
    assert_eq!(payload["id"], 42);
    assert_eq!(payload["method"], "credential.list");
}

#[test]
fn test_decode_response() {
    let resp_json = json!({"id": 1, "result": {"path": "db/password", "value": "secret"}});
    let payload = serde_json::to_vec(&resp_json).unwrap();
    let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
    frame.extend_from_slice(&payload);

    let resp = decode(&frame).unwrap();
    assert_eq!(resp.id, 1);
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn test_decode_error_response() {
    let resp_json = json!({"id": 2, "error": {"code": "VAULT_LOCKED", "message": "Vault is locked"}});
    let payload = serde_json::to_vec(&resp_json).unwrap();
    let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
    frame.extend_from_slice(&payload);

    let resp = decode(&frame).unwrap();
    assert_eq!(resp.id, 2);
    assert!(resp.error.is_some());
}

#[test]
fn test_decode_too_short() {
    let result = decode(&[0, 0]);
    assert!(result.is_err());
}
