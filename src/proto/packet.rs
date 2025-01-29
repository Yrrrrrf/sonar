pub struct Packet {
    packet_type: PacketType,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    Control,
    Data,
    Error,
}
