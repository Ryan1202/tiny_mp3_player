use std::{fs::File, io::BufReader};

use types::{
    Channel, MpegBitRate, MpegChannelMode, MpegCopyright, MpegLayer, MpegModeExtension, MpegOringinal, MpegProtection, MpegSampleRate, MpegSideInfo, MpegVersion, ScaleFactor
};

use crate::{bitstream::{BitReader, BitStream}, DecodeError};
use crate::dbg_println;
use crate::debug::DebugType;

pub mod types;

const SLEN: [[usize; 2]; 16] = [
    [0, 0],
    [0, 1],
    [0, 2],
    [0, 3],
    [3, 0],
    [1, 1],
    [1, 2],
    [1, 3],
    [2, 1],
    [2, 2],
    [2, 3],
    [3, 1],
    [3, 2],
    [3, 3],
    [4, 2],
    [4, 3],
];

pub struct MpegHeader {
    pub version: MpegVersion,
    pub layer: MpegLayer,
    pub protection: MpegProtection,
    pub bit_rate: MpegBitRate,
    pub sample_rate: MpegSampleRate,
    pub padding: bool,
    pub channel: MpegChannelMode,
    pub mode_extension: MpegModeExtension,
    pub copyright: MpegCopyright,
    pub original: MpegOringinal,
}

pub fn parse_header(header: &[u8; 4]) -> Result<MpegHeader, DecodeError> {
    if header[0] != 0xff || (header[1] & 0xe0) != 0xe0 {
        return Err(DecodeError::CanNotFindFrameSync);
    }

    dbg_println!(DebugType::Header, "MPEG Frame:");
    dbg_println!(DebugType::Header, "Header:");
    dbg_println!(DebugType::Header, "Frame sync: 1111 1111 111");


    let version = MpegVersion::new((header[1] >> 3) & 0x03);
    dbg_println!(DebugType::Header,
        "MPEG Version:{}({:02b})",
        version.to_string(),
        version.to_value()
    );
    if version == MpegVersion::Reserved {
        return Err(DecodeError::UnsupportedMpegVersion(version.to_value()));
    }

    let layer = MpegLayer::new((header[1] >> 1) & 0x03);
    dbg_println!(DebugType::Header, "Layer: {}({:02b})", layer.to_string(), layer.to_value());

    let protection = MpegProtection::new(header[1] & 0x01);

    dbg_println!(DebugType::Header,
        "Protection: {}({:01b})",
        protection.to_string(), protection.to_value());

    let bit_rate = MpegBitRate::new(header[2] >> 4, version, layer);
    dbg_println!(DebugType::Header,
        "Bit Rate: {}({:04b})",
        bit_rate.to_string(), bit_rate.get_value());

    let sample_rate = MpegSampleRate::new((header[2] >> 2) & 0x03, version);
    dbg_println!(DebugType::Header,
        "Sample Rate: {}({:02b})",
        sample_rate.to_string(), sample_rate.get_value());

    let padding_bit = (header[2] >> 1) & 0x01 == 1;

    let channel_mode = MpegChannelMode::new(header[3] >> 6);
    dbg_println!(DebugType::Header,
        "Channel Mode: {}({:02b})",
        channel_mode.to_string(), channel_mode.to_value());

    let mode_extension = MpegModeExtension::new((header[3] >> 4) & 0x03);
    dbg_println!(DebugType::Header,
        "Mode Extension: {}({:02b})",
        mode_extension.to_string(), mode_extension.to_value());

    let copyright = MpegCopyright::new((header[3] >> 3) & 0x01);
    dbg_println!(DebugType::Header,
        "Copyright: {}({})",
        copyright.to_string(), copyright.to_value());

    let original = MpegOringinal::new((header[3] >> 2) & 0x01);
    dbg_println!(DebugType::Header,
        "Original: {}({})",
        original.to_string(), original.to_value());

    dbg_println!(DebugType::Header, "\n");

    Ok(MpegHeader {
        version: version,
        layer: layer,
        protection: protection,
        bit_rate: bit_rate,
        sample_rate: sample_rate,
        padding: padding_bit,
        channel: channel_mode,
        mode_extension: mode_extension,
        copyright: copyright,
        original: original,
    })
}

pub fn parse_side_info(header: &MpegHeader, bs: &mut BitStream<BufReader<File>>) -> MpegSideInfo {
    let mut side_info: MpegSideInfo = MpegSideInfo::new();

    side_info.main_data_end = bs.read(9).unwrap();
    dbg_println!(DebugType::SideInfo,
        "Side Info:\nmain data end: {0}({0:09b})",
        side_info.main_data_end
    );

    if header.channel == MpegChannelMode::SingleChannel {
        side_info.private_bits = bs.read(5).unwrap();
        let scfsi = bs.read(4).unwrap();
        side_info.scfsi[0] = [
            scfsi >> 3,
            (scfsi >> 2) & 0x01,
            (scfsi >> 1) & 0x01,
            scfsi & 0x01,
        ];

        dbg_println!(DebugType::SideInfo,
            "private bits: {0}({0:09b})",
            side_info.private_bits);
        dbg_println!(DebugType::SideInfo,
            "scfsi: [{:01b}, {:01b}, {:01b}, {:01b}]",
            side_info.scfsi[0][0], side_info.scfsi[0][1], side_info.scfsi[0][2], side_info.scfsi[0][3]);

        for gr in 0..2 {
            let channel = &mut side_info.granule[gr].channel[0];
            channel.part2_3_length = bs.read(12).unwrap();
            channel.big_values = bs.read(9).unwrap();
            channel.global_gain = bs.read(8).unwrap();
            channel.scalefac_compress = bs.read(4).unwrap();
            channel.blocksplit_flag = bs.read(1).unwrap();
            
            dbg_println!(DebugType::SideInfo, "Granule {}:", gr);
            dbg_println!(DebugType::SideInfo, "[\npart2_3_length: {0}({0:012b})", channel.part2_3_length);
            dbg_println!(DebugType::SideInfo, "big_values: {0}({0:09b})", channel.big_values);
            dbg_println!(DebugType::SideInfo, "global_gain: {0}({0:08b})", channel.global_gain);
            dbg_println!(DebugType::SideInfo, "scalefac_compress: {0}({0:04b})", channel.scalefac_compress);
            dbg_println!(DebugType::SideInfo, "blocksplit_flag: {0}", channel.blocksplit_flag);

            if channel.blocksplit_flag != 0 {
                channel.block_type = bs.read(2).unwrap();
                channel.switch_point = bs.read(1).unwrap();
                channel.table_select = [bs.read(5).unwrap(), bs.read(5).unwrap(), 0];
                channel.subblock_gain = [
                    bs.read(3).unwrap(),
                    bs.read(3).unwrap(),
                    bs.read(3).unwrap(),
                ];
                channel.region_address1 = if channel.block_type == 2 {8} else {7};
                channel.region_address2 = 20 - channel.region_address1;

                dbg_println!(DebugType::SideInfo, "\tblock_type: {:02b}", channel.block_type);
                dbg_println!(DebugType::SideInfo, "\tswitch_point: {:01b}", channel.switch_point);
                dbg_println!(DebugType::SideInfo, "\ttable_select: [{},{}]", channel.table_select[0], channel.table_select[1]);
                dbg_println!(DebugType::SideInfo, "\tsubblock_gain: [{},{},{}]", channel.subblock_gain[0], channel.subblock_gain[1], channel.subblock_gain[2]);
            } else {
                channel.table_select = [
                    bs.read(5).unwrap(),
                    bs.read(5).unwrap(),
                    bs.read(5).unwrap(),
                ];
                channel.region_address1 = bs.read(4).unwrap();
                channel.region_address2 = bs.read(3).unwrap();

                dbg_println!(DebugType::SideInfo, "\ttable_select: [{},{},{}]", channel.table_select[0], channel.table_select[1], channel.table_select[2]);
                dbg_println!(DebugType::SideInfo, "\tregion_address1: {}", channel.region_address1);
                dbg_println!(DebugType::SideInfo, "\tregion_address2: {}", channel.region_address2);
            }

            channel.preflag = bs.read(1).unwrap();
            channel.scalefac_scale = bs.read(1).unwrap();
            channel.count1table_select = bs.read(1).unwrap();

            dbg_println!(DebugType::SideInfo, "\tpreflag: {}", channel.preflag);
            dbg_println!(DebugType::SideInfo, "\tscalefac_scale: {}", channel.scalefac_scale);
            dbg_println!(DebugType::SideInfo, "\tcount1table_select: {}\n]", channel.count1table_select);
        }
    } else {
        side_info.private_bits = bs.read(3).unwrap();
        for ch in 0..2 {
            let scfsi = &mut side_info.scfsi[ch];
            for i in 0..4 {
                scfsi[i] = bs.read(1).unwrap();
            }
        }
        for gr in 0..2 {
            for ch in 0..2 {
                let granule = &mut side_info.granule[gr];
                let channel = &mut granule.channel[ch];

                channel.part2_3_length = bs.read(12).unwrap();
                channel.big_values = bs.read(9).unwrap();
                channel.global_gain = bs.read(8).unwrap();
                channel.scalefac_compress = bs.read(4).unwrap();
                channel.blocksplit_flag = bs.read(1).unwrap();
                
                dbg_println!(DebugType::SideInfo, "Channel{} Granule{}:", ch, gr);
                dbg_println!(DebugType::SideInfo, "[\npart2_3_length: {0}({0:012b})", channel.part2_3_length);
                dbg_println!(DebugType::SideInfo, "big_values: {0}({0:09b})", channel.big_values);
                dbg_println!(DebugType::SideInfo, "global_gain: {0}({0:08b})", channel.global_gain);
                dbg_println!(DebugType::SideInfo, "scalefac_compress: {0}({0:04b})", channel.scalefac_compress);
                dbg_println!(DebugType::SideInfo, "blocksplit_flag: {0}", channel.blocksplit_flag);
                if channel.blocksplit_flag != 0 {
                    channel.block_type = bs.read(2).unwrap();
                    channel.switch_point = bs.read(1).unwrap();
                    channel.table_select = [
                        bs.read(5).unwrap(),
                        bs.read(5).unwrap(),
                        0
                    ];
                    channel.subblock_gain = [
                        bs.read(3).unwrap(),
                        bs.read(3).unwrap(),
                        bs.read(3).unwrap(),
                    ];

                    channel.region_address1 = if channel.block_type == 2 {8} else {7};
                    channel.region_address2 = 20 - channel.region_address1;dbg_println!(DebugType::SideInfo, "\tblock_type: {:02b}", channel.block_type);
                    dbg_println!(DebugType::SideInfo, "\tswitch_point: {:01b}", channel.switch_point);
                    dbg_println!(DebugType::SideInfo, "\ttable_select: [{},{}]", channel.table_select[0], channel.table_select[1]);
                    dbg_println!(DebugType::SideInfo, "\tsubblock_gain: [{},{},{}]", channel.subblock_gain[0], channel.subblock_gain[1], channel.subblock_gain[2]);
                } else {
                    channel.table_select = [
                        bs.read(5).unwrap(),
                        bs.read(5).unwrap(),
                        bs.read(5).unwrap(),
                    ];
                    channel.region_address1 = bs.read(4).unwrap();
                    channel.region_address2 = bs.read(3).unwrap();

                    dbg_println!(DebugType::SideInfo, "\ttable_select: [{},{},{}]", channel.table_select[0], channel.table_select[1], channel.table_select[2]);
                    dbg_println!(DebugType::SideInfo, "\tregion_address1: {}", channel.region_address1);
                    dbg_println!(DebugType::SideInfo, "\tregion_address2: {}", channel.region_address2);
                }

                channel.preflag = bs.read(1).unwrap();
                channel.scalefac_scale = bs.read(1).unwrap();
                channel.count1table_select = bs.read(1).unwrap();
            }
        }
    }
    side_info
}

pub fn parse_scale_factor<R: BitReader>(
    gr: usize,
    bs: &mut BitStream<R>,
    scfsi: &[usize; 4],
    channel: &Channel,
    sf_gr0: ScaleFactor,
) -> ScaleFactor {

    let slen1 = SLEN[channel.scalefac_compress][0];
    let slen2 = SLEN[channel.scalefac_compress][1];

    let mut sf_l = [0usize; 23];
    let mut sf_s = [[0usize; 3]; 13];

    dbg_println!(DebugType::ScaleFactor, "\nScale factor:");

    if channel.blocksplit_flag == 1 && channel.block_type == 2 {

        // 如果Block的类型是短块
        let (switch_point_l, switch_point_s) = if channel.switch_point == 0 {
            (0, 0)
        } else {
            (8, 3)
        };

        for sfb in 0..switch_point_l {
            sf_l[sfb] = bs.read(slen1).unwrap();
        }
        for sfb in switch_point_s..6 {
            for window in 0..3 {
                sf_s[sfb][window] = bs.read(slen1).unwrap();
            }
        }
        for sfb in 6..12 {
            for window in 0..3 {
                sf_s[sfb][window] = bs.read(slen2).unwrap();
            }
        }

        sf_s[12] = [0, 0, 0];
        dbg_println!(DebugType::ScaleFactor, "sf_s:{:?}", sf_s);
    } else {
        if gr == 0 {

            // granule0为第一个granule，只能直接读取
            for sfb in 0..11 {
                sf_l[sfb] = bs.read(slen1).unwrap();
            }

            for sfb in 11..21 {
                sf_l[sfb] = bs.read(slen2).unwrap();
            }
        } else {
            // 如果scfsi为1表示复制前一个granule的内容，为0则需要读取
            let mut sfb = 0;
            for i in 0..4 {
                let slen = if i < 2 {slen1} else {slen2};
                let sb_len = if i == 0 {6} else {5};
                if scfsi[i] == 1 {
                    for _ in 0..sb_len {
                        sf_l[sfb] = sf_gr0.sf_l[sfb];
                        sfb += 1;
                    }
                } else {
                    for _ in 0..sb_len {
                        sf_l[sfb] = bs.read(slen).unwrap();
                        sfb += 1;
                    }
                }
            }
        }
        dbg_println!(DebugType::ScaleFactor, "sf_l:{:?}", sf_l);
    }


    ScaleFactor {
        sf_l: sf_l,
        sf_s: sf_s,
    }
}
