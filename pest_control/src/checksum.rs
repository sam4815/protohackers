pub fn calculate_checksum(bytes: &[u8]) -> u8 {
    0xFF - bytes.iter().fold(0x00, |a, b| (a + b) % 0xFF)
}
