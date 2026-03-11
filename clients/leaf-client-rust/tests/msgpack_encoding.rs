//! Test MessagePack encoding to match server expectations

use rmp_serde::encode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestBin {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

#[test]
fn test_rmp_binary_encoding() {
    let test_data = vec![1, 2, 3, 4, 5];
    let test_obj = TestBin { data: test_data.clone() };

    // Encode with rmp_serde
    let encoded = encode::to_vec(&test_obj).unwrap();
    println!("Encoded: {:?}", encoded);

    // Try encoding just bytes
    let bytes_encoded = rmp_serde::to_vec(&test_data).unwrap();
    println!("Bytes encoded: {:?}", bytes_encoded);
}

#[test]
fn test_manual_bin32() {
    let payload = vec![1, 2, 3, 4, 5];
    let len = payload.len() as u32;

    let mut msgpack = Vec::new();
    msgpack.push(0xC6); // bin32 marker
    msgpack.extend_from_slice(&len.to_be_bytes());
    msgpack.extend_from_slice(&payload);

    println!("Manual bin32: {:?}", msgpack);
}

#[test]
fn test_compare_formats() {
    let payload = vec![1, 2, 3, 4, 5];

    // Format 1: Manual bin32
    let len = payload.len() as u32;
    let mut manual = Vec::new();
    manual.push(0xC6);
    manual.extend_from_slice(&len.to_be_bytes());
    manual.extend_from_slice(&payload);

    // Format 2: rmp_serde
    let rmp_encoded = rmp_serde::to_vec(&payload).unwrap();

    println!("Manual: {:?}", manual);
    println!("RMP:    {:?}", rmp_encoded);
    println!("Manual len: {}, RMP len: {}", manual.len(), rmp_encoded.len());

    // Try to decode the rmp version
    use rmp_serde::decode;
    let decoded: Vec<u8> = decode::from_slice(&rmp_encoded).unwrap();
    println!("Decoded: {:?}", decoded);
    assert_eq!(decoded, payload);
}
