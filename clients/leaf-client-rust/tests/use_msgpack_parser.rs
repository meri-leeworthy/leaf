//! Test if we can use socketioxide-parser-msgpack to encode/decode packets

use socketioxide_parser_msgpack::MsgPackParser;
use socketioxide_core::parser::Parse;
use socketioxide_core::Value;

#[test]
fn test_msgpack_parser_exists() {
    let parser = MsgPackParser;
    println!("✅ MsgPackParser created: {:?}", parser);
}

#[test]
fn test_encode_value() {
    let parser = MsgPackParser;

    // Try to encode a simple value
    let data = vec![1, 2, 3, 4, 5];

    match parser.encode_value(&data, Some("test")) {
        Ok(Value::Bytes(bytes)) => {
            println!("✅ Encoded {} bytes as msgpack", bytes.len());
            println!("   First 10 bytes: {:?}", &bytes[..bytes.len().min(10)]);
        }
        Ok(other) => {
            println!("⚠️  Got non-bytes value: {:?}", other);
        }
        Err(e) => {
            println!("❌ Encode error: {:?}", e);
        }
    }
}

#[test]
fn test_encode_cbor_as_msgpack() {
    let parser = MsgPackParser;

    // Encode some CBOR data as msgpack (simulating what we'd send to server)
    let cbor_data = vec![0x82, 0x01, 0x02]; // Example CBOR

    match parser.encode_value(&cbor_data, Some("module/upload")) {
        Ok(Value::Bytes(bytes)) => {
            println!("✅ CBOR data wrapped in msgpack: {} bytes", bytes.len());
            println!("   First 10 bytes: {:?}", &bytes[..bytes.len().min(10)]);
        }
        Ok(other) => {
            println!("⚠️  Got non-bytes value: {:?}", other);
        }
        Err(e) => {
            println!("❌ Encode error: {:?}", e);
        }
    }
}
