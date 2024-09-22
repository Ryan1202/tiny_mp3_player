use crate::{
    huffman::tables::{BAND_INDEX_L, BAND_WIDTH_S},
    mpeg_frame::{
        types::{Channel, ScaleFactor},
        MpegHeader,
    },
};

const PRETAB: [usize; 23] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 3, 2,
    0, 0,
];

pub fn requantize(
    samples: &mut [f32; 576],
    header: &MpegHeader,
    sf: ScaleFactor,
    channel: &Channel,
) {
    let sample_rate = header.sample_rate.get_value();

    let mut sfb = 0;
    let mut window = 0;
    let mut i = 0;
    for sample in 0..576 {
        let a;
        let b;
        if channel.block_type == 2 || (channel.switch_point == 1 && sfb >= 8) {
            if i == BAND_WIDTH_S[sample_rate][sfb] {
                i = 0;
                if window == 2 {
                    window = 0;
                    sfb += 1;
                } else {
                    window += 1;
                }
            }

            a = channel.global_gain as f32 - 210.0 - 8.0 * channel.subblock_gain[window] as f32;
            b = (channel.scalefac_scale as f32 + 1.0) / 2.0 * sf.sf_s[sfb][window] as f32;
        } else {
            if sample == BAND_INDEX_L[sample_rate][sfb + 1] {
                sfb += 1;
            }

            a = channel.global_gain as f32 - 210.0;
            b = (channel.scalefac_scale as f32 + 1.0) / 2.0
                * (sf.sf_l[sfb] as f32 + channel.preflag as f32 * PRETAB[sfb] as f32);
        }
        let sign = if samples[sample] >= 0.0 { 1.0 } else { -1.0 };
        let c = (samples[sample] as f32).abs().powf(4.0 / 3.0);
        let d = 2.0f32.powf(a / 4.0);
        let e = 2.0f32.powf(-b);

        samples[sample] = sign * c * d * e;
        i += 1;
    }
}
