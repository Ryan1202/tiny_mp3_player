use std::{fs::File, io::{self, BufReader, Seek, Write}};
use rodio::{OutputStream, Sink};
use tiny_mp3_player::{id3::Id3v2, DecodeError, Decoder};
use debug::{DebugType, DebugConfig};
use clap::{Parser, ArgAction};
mod debug;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "debug", value_name = "TYPE", action = ArgAction::Append)]
    debug: Vec<DebugType>,

    #[arg(required = true)]
    input_file: String,
}

fn pcm_f32_to_int16(sample: f32) -> i16
{
   if sample < -0.999999f32
   {
      return i16::MIN;
   }
   else if sample > 0.999999f32
   {
     return i16::MAX;
   }
   else
   {
     return (sample * 32767.0f32) as i16;
   }
}

fn write_wav_header<W: Write>(
    writer: &mut W,
    num_samples: u32,
    sample_rate: u32,
    num_channels: u16,
    bits_per_sample: u16,
) -> io::Result<()> {
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample / 8) as u32;
    let block_align = num_channels * (bits_per_sample / 8);

    // 写入 RIFF 头
    writer.write_all(b"RIFF")?;
    writer.write_all(&(36 + num_samples * (bits_per_sample / 8) as u32).to_le_bytes())?; // 文件大小
    writer.write_all(b"WAVE")?;

    // 写入 fmt 头
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?; // Subchunk1Size (PCM = 16)
    writer.write_all(&1u16.to_le_bytes())?;  // AudioFormat (PCM = 1)
    writer.write_all(&num_channels.to_le_bytes())?; // NumChannels
    writer.write_all(&sample_rate.to_le_bytes())?;  // SampleRate
    writer.write_all(&byte_rate.to_le_bytes())?;    // ByteRate
    writer.write_all(&block_align.to_le_bytes())?;  // BlockAlign
    writer.write_all(&bits_per_sample.to_le_bytes())?; // BitsPerSample

    // 写入 data 头
    writer.write_all(b"data")?;
    writer.write_all(&(num_samples * (bits_per_sample / 8) as u32).to_le_bytes())?; // Subchunk2Size

    Ok(())
}

#[test]
fn test() {
    let filename = "test.mp3";
    let file = File::open(filename).expect("打开文件失败！");
    let mut reader = BufReader::new(file);
    
    let id3 = Id3v2::new(&mut reader);
    if let None = id3 {
        reader.seek_relative(-10).unwrap();
    }

    let mut decoder = Decoder::new();

    decoder.calculate_mp3_duration(&mut reader).unwrap();

    let mut file = File::create("out.wav").unwrap();
    write_wav_header(&mut file, 230 * 1152, 44100, 2, 16).unwrap();

    let mut _i = 0;
    'outer: loop {
        let pcm_data = match decoder.decode_mp3(&mut reader) {
            Ok(pcm_data) => pcm_data,
            Err(e) => {
                match e {
                    DecodeError::EndOfFile => {
                        dbg_println!(DebugType::Decoder, "到达文件末尾，解码完成");
                        break 'outer;
                    },
                    _ => {
                        dbg_println!(DebugType::Decoder, "解码错误: {:?}", e);
                        break 'outer;
                    }
                }
            },
        };
        let mut _j = 0;
        for a in pcm_data {
            let x = pcm_f32_to_int16(a);
            file.write_all(&x.to_le_bytes()).unwrap();
            _j += 1;
        }
        _i += 1;
    }
}

fn main() {
    let args = Args::parse();
    
    // 初始化 DEBUG_CONFIG
    DebugConfig::init(&args.debug);

    let file = File::open(&args.input_file).expect("打开文件失败！");
    let mut reader = BufReader::new(file);

    // 读取描述信息
    let id3 = Id3v2::new(&mut reader);
    if let Some(id3) = id3 {
        let title = match id3.title {
            Some(title) => {title},
            None => {args.input_file.clone()},
        };
        println!("Title: {}", title);
        if let Some(artist) = id3.artist {
            println!("Artist: {}", artist);
        }
        if let Some(album) = id3.album {
            println!("Album: {}", album);
        }
        if let Some(year) = id3.year {
            println!("Year: {}", year);
        }
        if let Some(comment) = id3.comment {
            println!("Comment: {}", comment);
        }
        if let Some(genre) = id3.genre {
            println!("Genre: {}", genre);
        }
        if let Some(track_number) = id3.track_number {
            println!("Track Number: {}", track_number);
        }
    } else {
        println!("Title: {}", args.input_file);
        reader.seek_relative(-10).unwrap();
    }

    let mut decoder = Decoder::new();

    // 计算时长
    let duration = decoder.calculate_mp3_duration(&mut reader).unwrap();
    let second = duration.as_secs();
    let minute = second / 60;
    let hour = minute / 60;
    let minute = minute % 60;
    let second = second % 60;
    print!("Duration: ");
    for i in [hour, minute] {
        if i != 0 {
            print!("{:02}:", i)
        }
    }
    println!("{:02}", second);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // 解码
    'outer: loop {
        if reader.stream_position().unwrap() >= decoder.data_end && decoder.data_end != 0 {
            break;
        }
        let pcm_data = match decoder.decode_mp3(&mut reader) {
            Ok(pcm_data) => pcm_data,
            Err(e) => {
                match e {
                    DecodeError::EndOfFile => {
                        dbg_println!(DebugType::Decoder, "到达文件末尾，解码完成");
                        break 'outer;
                    },
                    _ => {
                        dbg_println!(DebugType::Decoder, "解码错误: {:?}", e);
                        break 'outer;
                    }
                }
            },
        };

        let samples: Vec<i16> = pcm_data.iter()
            .map(|&sample| pcm_f32_to_int16(sample))
            .collect();
        let source = rodio::buffer::SamplesBuffer::new(
            decoder.channel_num as u16, 
            decoder.sample_rate as u32, 
            samples
        );
        sink.append(source);
    }
    dbg_println!(DebugType::Decoder, "音频解码完成");
    sink.sleep_until_end();
}
