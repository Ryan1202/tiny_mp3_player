
pub fn get_id3_size(header: &[u8;10]) -> usize
{
    if header[0..3] == [b'I', b'D', b'3'] {
        (header[6] as usize & 0x7f) << 21 |
        (header[7] as usize & 0x7f) << 14 |
        (header[8] as usize & 0x7f) << 7 |
        (header[9] as usize & 0x7f) + 10
    } else {
        0
    }
}