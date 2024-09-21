use std::{fs::File, io::BufReader};

use types::{
    MpegBitRate, MpegChannelMode, MpegCopyright, MpegLayer, MpegOringinal, MpegProtection,
    MpegSampleRate, MpegSideInfo, MpegVersion, ScaleFactor,
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

    let copyright = MpegCopyright::new((header[3] >> 2) & 0x01);
    dbg_println!(DebugType::Header,
        "Copyright: {}({})",
        copyright.to_string(), copyright.to_value());

    let original = MpegOringinal::new((header[3] >> 1) & 0x01);
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
        side_info.scfsi = [
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
            side_info.scfsi[0], side_info.scfsi[1], side_info.scfsi[2], side_info.scfsi[3]);

        let granule = &mut side_info.granule;
        for gr in 0..2 {
            granule[gr].part2_3_length = bs.read(12).unwrap();
            granule[gr].big_values = bs.read(9).unwrap();
            granule[gr].global_gain = bs.read(8).unwrap();
            granule[gr].scalefac_compress = bs.read(4).unwrap();
            granule[gr].blocksplit_flag = bs.read(1).unwrap();
            
            dbg_println!(DebugType::SideInfo,
                "[\npart2_3_length: {0}({0:012b})", granule[gr].part2_3_length);
            dbg_println!(DebugType::SideInfo,
                "big_values: {0}({0:09b})", granule[gr].big_values);
            dbg_println!(DebugType::SideInfo,
                "global_gain: {0}({0:08b})", granule[gr].global_gain);
            dbg_println!(DebugType::SideInfo,
                "scalefac_compress: {0}({0:04b})", granule[gr].scalefac_compress);
            dbg_println!(DebugType::SideInfo,
                "blocksplit_flag: {0}", granule[gr].blocksplit_flag);

            if granule[gr].blocksplit_flag != 0 {
                granule[gr].block_type = bs.read(2).unwrap();
                granule[gr].switch_point = bs.read(1).unwrap();
                granule[gr].table_select = [bs.read(5).unwrap(), bs.read(5).unwrap(), 0];
                granule[gr].subblock_gain = [
                    bs.read(3).unwrap(),
                    bs.read(3).unwrap(),
                    bs.read(3).unwrap(),
                ];
                granule[gr].region_address1 = if granule[gr].block_type == 2 {8} else {7};
                granule[gr].region_address2 = 20 - granule[gr].region_address1;

                dbg_println!(DebugType::SideInfo,
                    "\tblock_type: {:02b}", granule[gr].block_type);
                dbg_println!(DebugType::SideInfo,
                    "\tswitch_point: {:01b}", granule[gr].switch_point);
                dbg_println!(DebugType::SideInfo,
                    "\ttable_select: [{},{}]",
                    granule[gr].table_select[0],
                    granule[gr].table_select[1]);
                dbg_println!(DebugType::SideInfo,
                    "\tsubblock_gain: [{},{},{}]",
                    granule[gr].subblock_gain[0],
                    granule[gr].subblock_gain[1],
                    granule[gr].subblock_gain[2]);
            } else {
                granule[gr].table_select = [
                    bs.read(5).unwrap(),
                    bs.read(5).unwrap(),
                    bs.read(5).unwrap(),
                ];
                granule[gr].region_address1 = bs.read(4).unwrap();
                granule[gr].region_address2 = bs.read(3).unwrap();

                dbg_println!(DebugType::SideInfo,
                    "\ttable_select: [{},{},{}]",
                    granule[gr].table_select[0],
                    granule[gr].table_select[1],
                    granule[gr].table_select[2]);
                dbg_println!(DebugType::SideInfo,
                    "\tregion_address1: {}", granule[gr].region_address1);
                dbg_println!(DebugType::SideInfo,
                    "\tregion_address2: {}", granule[gr].region_address2);
            }

            granule[gr].preflag = bs.read(1).unwrap();
            granule[gr].scalefac_scale = bs.read(1).unwrap();
            granule[gr].count1table_select = bs.read(1).unwrap();

            dbg_println!(DebugType::SideInfo,
                "\tpreflag: {}", granule[gr].preflag);
            dbg_println!(DebugType::SideInfo,
                "\tscalefac_scale: {}", granule[gr].scalefac_scale);
            dbg_println!(DebugType::SideInfo,
                "\tcount1table_select: {}\n]", granule[gr].count1table_select);
        }
    }
    side_info
}

pub fn parse_scale_factor<R: BitReader>(
    gr: usize,
    bs: &mut BitStream<R>,
    side_info: &MpegSideInfo,
    sf_gr0: ScaleFactor,
) -> ScaleFactor {
    let granule = side_info.granule[gr];

    let slen1 = SLEN[granule.scalefac_compress][0];
    let slen2 = SLEN[granule.scalefac_compress][1];

    let mut sf_l = [0usize; 23];
    let mut sf_s = [[0usize; 3]; 13];

    dbg_println!(DebugType::ScaleFactor, "\nScale factor:");

    if granule.blocksplit_flag == 1 && granule.block_type == 2 {

        // 如果Block的类型是短块
        let (switch_point_l, switch_point_s) = if granule.switch_point == 0 {
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
                if side_info.scfsi[i] == 1 {
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
