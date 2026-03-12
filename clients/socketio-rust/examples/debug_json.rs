use socketio_rust::Packet;

fn main() {
    let packet = Packet::connect("/");
    let json = serde_json::to_string_pretty(&packet).unwrap();
    println!("Serialized CONNECT packet:\n{}", json);

    let event_packet = Packet::event("test", Some(serde_json::json!({"key":"value"})), None);
    let event_json = serde_json::to_string_pretty(&event_packet).unwrap();
    println!("\nSerialized EVENT packet:\n{}", event_json);
}
