use crate::{huffman::tables::BAND_WIDTH_S, mpeg_frame::{types::{Channel, Granule}, MpegHeader}};

pub mod requantize;
pub mod imdct;
pub mod synthesis;
pub mod synth_window;

const CS: [f32; 8] = [
    0.8574929257, 0.8817419973, 0.9496286491, 0.9833145925,
    0.9955178161, 0.9991605582, 0.9998991952, 0.9999931551
];
const CA: [f32; 8] = [
    -0.5144957554, -0.4717319686, -0.3133774542, -0.1819131996,
    -0.0945741925, -0.0409655829, -0.0141985686, -0.0036999747
];

const SUBBAND_SIZE_L: usize = 18;
const SUBBAND_SIZE_S: usize = 6;

pub fn reorder(
    samples: [f32; 576],
    header: &MpegHeader,
    channel: &Channel,
) -> [f32; 576] {
    let mut result = [0f32; 576];

    let mut base1 = 0;
    let mut base2 = 0;
    let mut block = 0;

    let start;
    if channel.switch_point == 1 {
        // 如果是混合块仅对短块部分处理
        start = 3;
        base1 = 36;
        base2 = base1;
    } else {
        start = 0;
    }

    let table = BAND_WIDTH_S[header.sample_rate.get_value()];
    for i in start..12 {
        let band_width = table[i];
        // 以子带为单元重排序
        for j in 0..band_width {
            result[base1 + SUBBAND_SIZE_S * 0 + block] = samples[base2 + band_width * 0 + j];
            result[base1 + SUBBAND_SIZE_S * 1 + block] = samples[base2 + band_width * 1 + j];
            result[base1 + SUBBAND_SIZE_S * 2 + block] = samples[base2 + band_width * 2 + j];
            if block >= SUBBAND_SIZE_S - 1 {
                block = 0;
                base1 += SUBBAND_SIZE_S * 3;
            } else {
                block += 1;
            }
        }

        base2 += band_width * 3;
    }
    result
}

pub fn anti_alias(
    samples: &mut [f32; 576],
    channel: &Channel,
) {
    let end;
    if channel.switch_point == 1 {
        end = 2;
    } else {
        end = 32;
    }

    for i in 1..end {
        for j in 0..8 {
            let pos1 = i * SUBBAND_SIZE_L - j - 1;
            let pos2 = i * SUBBAND_SIZE_L + j;
            let s1 = samples[pos1];
            let s2 = samples[pos2];
            samples[pos1] = s1 * CS[j] - s2 * CA[j];
            samples[pos2] = s2 * CS[j] + s1 * CA[j];
        }
    }
}

pub fn frequency_inversion(
    samples: &mut [f32; 576],
) {
    for sb in (1..18).step_by(2) {
        for i in (1..32).step_by(2) {
            samples[sb + i * 18] *= -1.0;
        }
    }
}
