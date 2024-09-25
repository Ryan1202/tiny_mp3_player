use std::{
    fs::File, io::{BufReader, Read, Seek}, time::Duration, vec
};

use bitstream::BitStream;
use debug::DebugType;
use decode::{
    anti_alias, frequency_inversion, imdct::imdct, reorder, requantize::requantize,
    synthesis::Synthesis,
};
use huffman::decode_huffman;
use mpeg_frame::{
    parse_header, parse_scale_factor, parse_side_info,
    types::{MpegChannelMode, MpegProtection, ScaleFactor},
};
use thiserror;

pub mod bitstream;
pub mod debug;
pub mod decode;
pub mod huffman;
pub mod id3;
pub mod mpeg_frame;

const SQRT2: f32 = 1.41421356;

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
    prev_samples: [[[f32; 18]; 32]; 2],
    synthesis: Synthesis,
    fifo: [[f32; 1024]; 2],

    pub channel_num: usize,
    pub sample_rate: usize,

    pub data_end: u64,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            main_buf: Vec::new(),
            main_data_begin: 0,
            prev_samples: [[[0f32; 18]; 32]; 2],
            synthesis: Synthesis::new(),
            fifo: [[0f32; 1024]; 2],
            channel_num: 0,
            sample_rate: 0,
            data_end: 0,
        }
    }

    pub fn calculate_mp3_duration(
        &mut self,
        reader: &mut BufReader<File>,
    ) -> Result<Duration, DecodeError> {
        let pos = reader.stream_position().unwrap();
        let mut duration: f32 = 0.0;

        let mut buf = [0u8; 4];
        loop {
            match reader.read_exact(&mut buf) {
                Ok(_) => {}
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    return Err(DecodeError::ReadFileError(e));
                }
            };
            if buf[0..3] == [b'T', b'A', b'G'] {
                self.data_end = reader.stream_position().unwrap();
                break;
            }
            let mpeg_header = parse_header(&buf).unwrap();
            self.channel_num = mpeg_header.channel.to_channel_num();
            self.sample_rate = mpeg_header.sample_rate.get_rate();
            let bit_rate = mpeg_header.bit_rate.get_rate();

            let size = 144 * bit_rate * 1000 / mpeg_header.sample_rate.get_rate()
                - 4 + if mpeg_header.padding { 1 } else { 0 };
            // let nslots = size - 4
            // - if mpeg_header.channel == MpegChannelMode::SingleChannel {17} else {32}
            // - if mpeg_header.protection == MpegProtection::Protected {2} else {0}
            // + if mpeg_header.padding {1} else {0};

            duration += 1.0 / self.sample_rate as f32 * 1152.0;

            reader.seek_relative(size as i64).unwrap();
        }

        reader.seek(std::io::SeekFrom::Start(pos)).unwrap();
        Ok(Duration::from_secs_f32(duration))
    }

    pub fn decode_mp3(&mut self, reader: &mut BufReader<File>) -> Result<Vec<f32>, DecodeError> {
        let mut buf = [0u8; 4];
        match reader.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Err(DecodeError::EndOfFile);
                }
                return Err(DecodeError::ReadFileError(e));
            }
        };
        let mpeg_header = parse_header(&buf)?;
        self.channel_num = mpeg_header.channel.to_channel_num();
        self.sample_rate = mpeg_header.sample_rate.get_rate();

        let size =
            144 * mpeg_header.bit_rate.get_rate() * 1000 / mpeg_header.sample_rate.get_rate();

        let side_info = {
            let mut bs = BitStream::new(reader);
            parse_side_info(&mpeg_header, &mut bs)
        };
        let nslots = size
            - 4
            - if mpeg_header.channel == MpegChannelMode::SingleChannel {
                17
            } else {
                32
            }
            - if mpeg_header.protection == MpegProtection::Protected {
                2
            } else {
                0
            }
            + if mpeg_header.padding { 1 } else { 0 };
        dbg_println!(DebugType::Header, "nslots: {}", nslots);

        self.main_buf = self
            .main_buf
            .split_off(self.main_buf.len() - side_info.main_data_end);
        self.main_data_begin = side_info.main_data_end;

        let mut buf = vec![0u8; nslots];
        reader.read_exact(&mut buf).unwrap();
        self.main_buf.extend(buf);

        // reader.seek(std::io::SeekFrom::Current(nslots as i64)).unwrap();

        let mut sf = [[ScaleFactor::new(); 2]; 2];
        let mut samples = [[[0.0; 576]; 2]; 2];
        let mut bs = BitStream::new(&mut self.main_buf);
        let mut pcm = vec![0.0; 2304];

        for gr in 0..2 {
            let granule = &side_info.granule[gr];
            for ch in 0..self.channel_num {
                let channel = &granule.channel[ch];
                let max_bit = bs.get_bit_offset() + channel.part2_3_length;
                sf[gr][ch] =
                    parse_scale_factor(gr, &mut bs, &side_info.scfsi[ch], &channel, sf[0][ch]);
                decode_huffman(
                    &mut bs,
                    &mpeg_header,
                    &channel,
                    &mut samples[gr][ch],
                    max_bit,
                );
            }

            for ch in 0..self.channel_num {
                let channel = &granule.channel[ch];
                requantize(&mut samples[gr][ch], &mpeg_header, sf[gr][ch], channel);
            }

            if mpeg_header.channel == MpegChannelMode::JointStereo
                && mpeg_header.mode_extension.ms_stereo
            {
                for i in 0..576 {
                    let middle = samples[gr][0][i];
                    let side = samples[gr][1][i];
                    samples[gr][0][i] = (middle + side) / SQRT2;
                    samples[gr][1][i] = (middle - side) / SQRT2;
                }
            }

            for ch in 0..self.channel_num {
                let channel = &granule.channel[ch];

                if channel.block_type == 2 || channel.switch_point == 1 {
                    samples[gr][ch] = reorder(samples[gr][ch], &mpeg_header, &channel);
                }
                if channel.block_type != 2 {
                    anti_alias(&mut samples[gr][ch], &channel);
                }
                imdct(&mut samples[gr][ch], &mut self.prev_samples[ch], &channel);
                frequency_inversion(&mut samples[gr][ch]);
                // 合成滤波的同时将两个声道（如果有）数据交替写入pcm
                self.synthesis.synthesis_filter(
                    &samples[gr][ch],
                    &mut pcm[gr * 576 * self.channel_num..],
                    &mut self.fifo[ch],
                    ch,
                    self.channel_num,
                );
            }
        }

        Ok(pcm[0..(1152 * self.channel_num)].to_vec())
    }
}
