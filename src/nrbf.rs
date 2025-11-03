use std::error::Error;

use nrbf::RemotingMessage;

#[derive(Debug)]
pub enum ParsedCommunication {
    Unknown,
}

/// Decode the message custom action
pub fn decode_single_nrbf(packet: &[u8]) -> Result<ParsedCommunication, Box<dyn Error>> {
    let _ = packet;

    log_bytes(packet);

    match RemotingMessage::parse(packet) {
        Ok(_something) => Ok(ParsedCommunication::Unknown),
        Err(err) => Err(err.to_string().into()),
    }
}

fn log_bytes(buf: &[u8]) {
    let decoded: String = String::from_utf8_lossy(buf).to_string();

    let hex_repr = buf
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    trace!(
        "({} bytes)\nText: \n{}\nHex: \n{}",
        buf.len(),
        decoded.chars().collect::<String>(),
        hex_repr.chars().collect::<String>(),
    );
}
