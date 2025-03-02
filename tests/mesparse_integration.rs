use rust_async_tuyapi::{
    mesparse::{CommandType, Message, MessageParser},
    Payload,
};
use serde_json::json;
use std::collections::HashMap;

fn create_test_payload() -> Payload {
    let mut dps = HashMap::new();
    dps.insert("1".to_string(), json!(true));
    Payload::new(
        "002004265ccf7fb1b659".to_string(),
        None,
        None,
        None,
        None,
        Some(dps),
    )
}

#[test]
fn encode_and_decode_message() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.1", None).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, Some(2));
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    let decoded = parser.parse(&encoded).unwrap();

    assert_eq!(message_to_encode, decoded[0]);
}

#[test]
fn encode_and_decode_get_message_version_three_three() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.3", Some("bbe88b3f4106d354")).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, Some(2));
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    let decoded = parser.parse(&encoded).unwrap();

    assert_eq!(message_to_encode, decoded[0]);
}

#[test]
fn encode_and_decode_set_message_version_three_three() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.3", Some("bbe88b3f4106d354")).unwrap();
    let message_to_encode = Message::new(payload, CommandType::Control, Some(0));
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    let decoded = parser.parse(&encoded).unwrap();

    assert_eq!(message_to_encode, decoded[0]);
}

#[test]
fn decode_empty_message() {
    let payload = Payload::String("".to_string());

    let parser = MessageParser::create("3.1", None).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, Some(0));
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    let decoded = parser.parse(&encoded).unwrap();

    assert_eq!(message_to_encode, decoded[0]);
}

#[test]
fn decode_corrupt_shortened_message() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.1", None).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, None);
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    assert!(parser.parse(&encoded[40..]).is_err());
}

#[test]
fn decode_corrupt_shorter_than_possible_message() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.1", None).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, None);
    let encoded = parser.encode(&message_to_encode, false).unwrap();

    assert!(parser.parse(&encoded[0..23]).is_err());
}

#[test]
fn decode_corrupt_crc_mismatch_message() {
    let payload = create_test_payload();
    let parser = MessageParser::create("3.1", None).unwrap();
    let message_to_encode = Message::new(payload, CommandType::DpQuery, None);
    let encoded = parser.encode(&message_to_encode, false).unwrap();
    // mess up the crc code
    let mut messedup_encoded: Vec<u8> = vec![];
    messedup_encoded.extend(encoded[0..encoded.len() - 8].iter());
    messedup_encoded.extend(hex::decode("DEADBEEF").unwrap());
    messedup_encoded.extend(hex::decode("0000AA55").unwrap());
    assert!(parser.parse(&messedup_encoded).is_err());
}
