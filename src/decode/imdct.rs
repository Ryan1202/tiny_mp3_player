use std::f32::consts::PI;

use crate::mpeg_frame::types::{Channel, Granule};

const SINE_BLOCK: [[f32; 36]; 4] = [
	[
		0.043619387365336, 0.13052619222005157, 0.21643961393810288, 0.3007057995042731, 0.3826834323650898, 0.4617486132350339, 0.5372996083468238, 0.6087614290087207, 0.6755902076156601, 0.737277336810124, 0.7933533402912352, 0.8433914458128856, 0.8870108331782216, 0.9238795325112867, 0.9537169507482268, 0.9762960071199334, 0.9914448613738104, 0.9990482215818578, 0.9990482215818578, 0.9914448613738104, 0.9762960071199334, 0.9537169507482269, 0.9238795325112867, 0.8870108331782218, 0.8433914458128858, 0.7933533402912352, 0.7372773368101241, 0.6755902076156604, 0.6087614290087209, 0.5372996083468241, 0.4617486132350339, 0.3826834323650899, 0.30070579950427334, 0.21643961393810318, 0.13052619222005157, 0.04361938736533607, 
	],[
		0.043619387365336, 0.13052619222005157, 0.21643961393810288, 0.3007057995042731, 0.3826834323650898, 0.4617486132350339, 0.5372996083468238, 0.6087614290087207, 0.6755902076156601, 0.737277336810124, 0.7933533402912352, 0.8433914458128856, 0.8870108331782216, 0.9238795325112867, 0.9537169507482268, 0.9762960071199334, 0.9914448613738104, 0.9990482215818578, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9914448613738105, 0.9238795325112868, 0.7933533402912354, 0.6087614290087209, 0.3826834323650899, 0.130526192220052, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 
	],[
		0.13052619222005157, 0.3826834323650898, 0.6087614290087205, 0.7933533402912352, 0.9238795325112867, 0.9914448613738104, 0.9914448613738105, 0.9238795325112868, 0.7933533402912354, 0.6087614290087209, 0.3826834323650899, 0.130526192220052, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 
	],[
		0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.13052619222005157, 0.3826834323650898, 0.6087614290087205, 0.7933533402912352, 0.9238795325112867, 0.9914448613738104, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9990482215818578, 0.9914448613738104, 0.9762960071199334, 0.9537169507482269, 0.9238795325112867, 0.8870108331782218, 0.8433914458128858, 0.7933533402912352, 0.7372773368101241, 0.6755902076156604, 0.6087614290087209, 0.5372996083468241, 0.4617486132350339, 0.3826834323650899, 0.30070579950427334, 0.21643961393810318, 0.13052619222005157, 0.04361938736533607, 
	]
];

pub fn imdct(
    samples: &mut [f32; 576],
    prev_samples: &mut [[f32;18]; 32],
    channel: &Channel,
) {
    let n = if channel.block_type == 2 {12} else {36};
    let half_n = n / 2;
    let win_cnt = if channel.block_type == 2 {3} else {1};
    let mut sample_block = [0f32; 36];

    for block in 0..32 {
        for win in 0..win_cnt {
            for i in 0..n {
                let mut xi = 0.0f32;
                for k in 0..half_n {
                    xi += samples[block * 18 + half_n * win + k] *
                        (PI / n as f32 / 2.0 * (2.0 * i as f32 + 1.0 + half_n as f32) * (2.0 * k as f32 + 1.0)).cos();
                }

                sample_block[win * n + i] = xi * SINE_BLOCK[channel.block_type][i];
            }
        }

        if channel.block_type == 2 {
            let mut tmp_block = [0f32; 36];
            for i in 0..6 {tmp_block[i] = 0.0;}
            for i in 6..12 {tmp_block[i] = sample_block[i - 6];}
            for i in 12..18 {tmp_block[i] = sample_block[i - 6] + sample_block[12 + i - 12];}
            for i in 18..24 {tmp_block[i] = sample_block[12 + i - 12] + sample_block[24 + i - 18];}
            for i in 24..30 {tmp_block[i] = sample_block[24 + i - 18];}
            for i in 30..36 {tmp_block[i] = 0.0;}
            sample_block = tmp_block;
        }

        for i in 0..18 {
            samples[block * 18 + i] = sample_block[i] + prev_samples[block][i];
            prev_samples[block][i] = sample_block[i + 18];
        }
    }
}
