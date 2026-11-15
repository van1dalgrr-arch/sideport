const MAGIC: &[u8; 8] = b"CFA6LPAA";
const HEADER_LEN: usize = 40;

#[derive(Debug)]
struct Header {
    entire_length: u64,
    this_length: u64,
    packet_num: u64,
    operation: u64,
}

impl Header {
    fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0u8; HEADER_LEN];

        bytes[0..8].copy_from_slice(MAGIC);
        bytes[8..16].copy_from_slice(&self.entire_length.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.this_length.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.packet_num.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.operation.to_le_bytes());

        bytes
    }
}
