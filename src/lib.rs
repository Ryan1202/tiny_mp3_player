use std::{fs::File, io::{BufReader, Read}};

use bitstream::BitStream;
use decode::{anti_alias, frequency_inversion, imdct::imdct, reorder, requantize::requantize, synthesis::Synthesis};
use huffman::decode_huffman;
use mpeg_frame::{parse_header, parse_scale_factor, parse_side_info, types::{MpegChannelMode, MpegProtection, ScaleFactor}};
use thiserror;

pub mod id3;
pub mod mpeg_frame;
pub mod bitstream;
pub mod huffman;
pub mod decode;
pub mod debug;


#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error("到达文件末尾")]
    EndOfFile,
    #[error("无法找到帧同步")]
    CanNotFindFrameSync,
    #[error("不支持的MPEG版本: {0}")]
    UnsupportedMpegVersion(u8),
    #[error("读取文件失败: {0}")]
    ReadFileError(std::io::Error),
}


pub struct Decoder {
    pub main_buf: Vec<u8>,
    main_data_begin: usize,
    prev_samples: [[f32; 18]; 32],
    synthesis: Synthesis,
    fifo: [f32; 1024],

    pub channel_num: usize,
    pub sample_rate: usize,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            main_buf: Vec::new(),
            main_data_begin: 0,
            prev_samples: [[0f32; 18]; 32],
            synthesis: Synthesis::new(),
            fifo: [0f32; 1024],
            channel_num: 0,
            sample_rate: 0,
        }
    }

    pub fn decode_mp3(&mut self, reader: &mut BufReader<File>) -> Result<[[f32;576]; 2], DecodeError> {
    
        let mut buf = [0u8;4];
        match reader.read_exact(&mut buf) {
            Ok(_) => {},
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Err(DecodeError::EndOfFile);
                }
                return Err(DecodeError::ReadFileError(e));
            },

        };
        let mpeg_header = parse_header(&buf)?;
        self.channel_num = mpeg_header.channel.to_channel_num();
        self.sample_rate = mpeg_header.sample_rate.get_rate();

        
        let size = 144 * mpeg_header.bit_rate.get_rate() * 1000 / mpeg_header.sample_rate.get_rate();
    
        let side_info = {
            let mut bs = BitStream::new(reader);
            parse_side_info(&mpeg_header, &mut bs)
        };
        let nslots = size - 4
            - if mpeg_header.channel == MpegChannelMode::SingleChannel {17} else {32}
            - if mpeg_header.protection == MpegProtection::Protected {2} else {0}
            + if mpeg_header.padding {1} else {0};
        // println!("nslots: {}", nslots);
        
        self.main_buf = self.main_buf.split_off(self.main_buf.len() - side_info.main_data_end);
        self.main_data_begin = side_info.main_data_end;

        let mut buf = vec![0u8; nslots];
        reader.read_exact(&mut buf).unwrap();
        self.main_buf.extend(buf);
    
        // reader.seek(std::io::SeekFrom::Current(nslots as i64)).unwrap();
    
        let mut sf = [ScaleFactor::new(); 2];
        let mut samples = [[0isize; 576]; 2];
        let mut bs = BitStream::new(&mut self.main_buf);
        for gr in 0..2 {
            let max_bit = bs.get_bit_offset() + side_info.granule[gr].part2_3_length;
            sf[gr] = parse_scale_factor(gr, &mut bs, &side_info, sf[0]);
            samples[gr] = decode_huffman(gr, &mut bs, &mpeg_header, &side_info, max_bit);
        }

        let mut result = [[0f32;576]; 2];
        for gr in 0..2 {
            let granule = side_info.granule[gr];
            result[gr] = requantize(samples[gr], &mpeg_header, sf[gr], &granule);
            
            if granule.block_type == 2 || granule.switch_point == 1 {
                result[gr] = reorder(result[gr], &mpeg_header, &granule);
            }
            if granule.block_type != 2 {
                anti_alias(&mut result[gr], &granule);
            }
            
            imdct(&mut result[gr], &mut self.prev_samples, &granule);
            frequency_inversion(&mut result[gr]);
            result[gr] = self.synthesis.synthesis_filter(&result[gr], &mut self.fifo);
        }
        Ok(result)
    }
}