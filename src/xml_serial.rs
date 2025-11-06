use crate::{instructions::InstructionFromCameraProgram, nrbf::hex_log_bytes};

pub struct BufferedParserXML {
    state: Vec<u8>,
}
impl BufferedParserXML {
    pub fn new() -> Self {
        Self { state: Vec::new() }
    }

    pub fn feed_bytes(
        &mut self,
        packet: &[u8],
    ) -> Option<Result<InstructionFromCameraProgram, String>> {
        let termination_res = check_xml_termination_bytes(packet);

        if self.state.is_empty() {
            if termination_res {
                // self contained packet (never copy)
                return Some(decode_single_xml(&packet[..packet.len() - 1]));
            } else {
                self.state.extend_from_slice(&packet);
            }
        } else {
            if termination_res {
                self.state.extend_from_slice(&packet[..packet.len() - 1]);

                // self contained packet (never copy)
                let res = decode_single_xml(&self.state);

                // clear internal buffer
                self.state.clear();

                return Some(res);
            } else {
                // append full packet
                self.state.extend_from_slice(packet);
            }
        }

        // Missing data
        None
    }
}

// 0D seems to be end of a message, always
fn check_xml_termination_bytes(possible_packet: &[u8]) -> bool {
    possible_packet.ends_with(b"\x0D")
}

fn decode_single_xml(packet: &[u8]) -> Result<InstructionFromCameraProgram, String> {
    let decoded: String = String::from_utf8_lossy(packet).to_string();

    debug!("XML message:\n{}", decoded);

    Err("Could not decode".into())
}

pub struct BufferedParserSerial {
    state: Vec<u8>,
}
impl BufferedParserSerial {
    pub fn new() -> Self {
        Self { state: Vec::new() }
    }

    pub fn feed_bytes(
        &mut self,
        packet: &[u8],
    ) -> Option<Result<InstructionFromCameraProgram, String>> {
        let termination_res = check_serial_termination_bytes(packet);

        if self.state.is_empty() {
            if termination_res {
                // self contained packet (never copy)
                return Some(decode_single_serial(&packet[..packet.len() - 1]));
            } else {
                self.state.extend_from_slice(&packet);
            }
        } else {
            if termination_res {
                self.state.extend_from_slice(&packet[..packet.len() - 1]);

                // self contained packet (never copy)
                let res = decode_single_serial(&self.state);

                // clear internal buffer
                self.state.clear();

                return Some(res);
            } else {
                // append full packet
                self.state.extend_from_slice(packet);
            }
        }

        // Missing data
        None
    }
}

// 0D seems to be end of a message, always
fn check_serial_termination_bytes(possible_packet: &[u8]) -> bool {
    possible_packet.ends_with(b"\x0D")
}

fn decode_single_serial(packet: &[u8]) -> Result<InstructionFromCameraProgram, String> {
    debug!("Serial message:");
    hex_log_bytes(packet);

    Err("Could not decode".into())
}
