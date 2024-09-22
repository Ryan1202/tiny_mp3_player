pub mod tables;

use tables::{BAND_INDEX_L, HUFFMAN_CODE_TABLE, HUFFMAN_CODE_TABLE_QUAD_A, HUFFMAN_CODE_TABLE_QUAD_B, HUFFMAN_CODE_TABLE_SIZE, HUFFMAN_LINBITS};

use crate::{bitstream::{BitReader, BitStream}, mpeg_frame::{types::Channel, MpegHeader}};

pub fn decode_huffman<R: BitReader>(
    bs: &mut BitStream<R>,
    header: &MpegHeader,
    channel: &Channel,
    samples: &mut [f32; 576],
    max_bit: usize,
) {
    let (region0_count, region1_count) = if channel.blocksplit_flag == 1 && channel.block_type == 2 {
        (36, 576)
    } else {
        (
            BAND_INDEX_L[header.sample_rate.get_value()][channel.region_address1 + 1],
            BAND_INDEX_L[header.sample_rate.get_value()][channel.region_address1 + 1 + channel.region_address2 + 1],
        )
    };

    // 解码big value区
    for i in 0..channel.big_values {
        let table_num;
        if i*2 < region0_count {
            table_num = channel.table_select[0];
        } else if i*2 < region1_count {
            table_num = channel.table_select[1];
        } else {
            table_num = channel.table_select[2];
        }
        let table = HUFFMAN_CODE_TABLE[table_num];
        if table_num == 0 {
            continue;
        }

        // 遍历找到哈夫曼编码对应的值对
        'outer: for x in 0..HUFFMAN_CODE_TABLE_SIZE[table_num] {
            for y in 0..HUFFMAN_CODE_TABLE_SIZE[table_num] {
                let linbits = HUFFMAN_LINBITS[table_num];
                let bin = table[x][y];

                let len = bin.bit_length;
                if bs.try_read(len).unwrap() == bin.value {
                    bs.read(len).unwrap();
                    let mut result = [x as f32, y as f32];
                    for value in result.iter_mut() {
                        if linbits != 0 && (*value as usize) == HUFFMAN_CODE_TABLE_SIZE[table_num] - 1 {
                            *value += bs.read(linbits).unwrap() as f32;
                        }
                        if *value > 0.0 {
                            *value *= if bs.read(1).unwrap() == 1 {-1.0} else {1.0};
                        }
                    }
                    samples[i * 2 + 0] = result[0];
                    samples[i * 2 + 1] = result[1];
                    break 'outer;
                }
            }
        }
    }

    // 解码count1区
    let mut bits_cnt = bs.get_bit_offset();
    let mut sample = channel.big_values * 2;
    while sample + 4 < 576 && bits_cnt < max_bit {
        let mut value = if channel.count1table_select == 1 {
            let tmp = HUFFMAN_CODE_TABLE_QUAD_B[bs.read(4).unwrap()].value;
            bits_cnt += 4;
            [
                (tmp >> 3) as f32,
                ((tmp >> 2) & 1) as f32,
                ((tmp >> 1) & 1) as f32,
                (tmp & 1) as f32,
            ]
        } else {
            let mut tmp = 0;
            for (i, bin) in HUFFMAN_CODE_TABLE_QUAD_A.iter().enumerate() {
                let len = bin.bit_length;

                if bs.try_read(len).unwrap() == bin.value {
                    bs.read(len).unwrap();
                    bits_cnt += len;
                    tmp = i as isize;
                    break;
                }
            }
            [
                (tmp >> 3) as f32,
                ((tmp >> 2) & 1) as f32,
                ((tmp >> 1) & 1) as f32,
                (tmp & 1) as f32,
            ]
        };
        for v in value.iter_mut() {
            if *v != 0.0 {
                *v *= if bs.read(1).unwrap() == 1 {-1.0} else {1.0};
                bits_cnt += 1;
            }
        }
        samples[sample + 0] = value[0];
        samples[sample + 1] = value[1];
        samples[sample + 2] = value[2];
        samples[sample + 3] = value[3];
        sample += 4;
    }
}