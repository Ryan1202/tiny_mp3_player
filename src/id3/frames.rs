pub enum ID3v2Frame {
    Title(String),        // TIT2
    Artist(String),       // TPE1
    Album(String),        // TALB
    Year(String),         // TYER
    Comment(String),      // COMM
    Genre(String),        // TCON
    TrackNumber(String),  // TRCK
    // 其他ID3v2标签类型可以在这里继续添加
    Unknown,
}

fn text_parse(data: &[u8], revision: u8) -> String {
    let encode = data[0];
    if encode == 0x01 {
        // UCS-2(相当于UTF16的子集)
        if (data[1] == 0xff && data[2] == 0xfe) || revision == 4 {
            // 小段字节序
            let mut result = String::new();
            for i in (3..data.len()).step_by(2) {
                let c = u16::from_le_bytes([data[i], data[i + 1]]);
                result.push(char::from_u32(c as u32).unwrap());
            }
            return result;
        } else if data[1] == 0xfe && data[2] == 0xff {
            // 大端字节序
            let mut result = String::new();
            for i in (3..data.len()).step_by(2) {
                let c = u16::from_be_bytes([data[i], data[i + 1]]);
                result.push(char::from_u32(c as u32).unwrap());
            }
            return result;
        }
    } else if encode == 3 {
        return String::from_utf8_lossy(&data[1..]).to_string();
    }
    return String::new();
}

impl ID3v2Frame {
    pub fn new(frame_id: &[u8; 4], data: &[u8], revision: u8) -> Self {
        match frame_id {
            [b'T', b'I', b'T', b'2'] => {
                Self::Title(text_parse(data, revision))
            },
            [b'T', b'P', b'E', b'1'] => {
                Self::Artist(text_parse(data, revision))
            },
            [b'T', b'A', b'L', b'B'] => {
                Self::Album(text_parse(data, revision))
            },
            [b'T', b'Y', b'E', b'R'] => {
                Self::Year(text_parse(data, revision))
            },
            [b'C', b'O', b'M', b'M'] => {
                Self::Comment(text_parse(data, revision))
            },
            [b'T', b'C', b'O', b'N'] => {
                Self::Genre(text_parse(data, revision))
            },
            [b'T', b'R', b'C', b'K'] => {
                Self::TrackNumber(text_parse(data, revision))
            },
            _ => Self::Unknown,
        }
    }
}