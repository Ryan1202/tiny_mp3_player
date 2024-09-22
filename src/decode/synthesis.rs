use std::f64::consts::PI;

use super::synth_window::SYNTH_WINDOW;

pub struct Synthesis {
    n: [[f32; 32]; 64],
}
impl Synthesis {
    pub fn new() -> Self {
        let mut n = [[0f32; 32]; 64];
        for i in 0..64 {
            for j in 0..32 {
                let f = (16.0 + i as f64) * (2.0 * j as f64 + 1.0) * (PI / 64.0);
                n[i][j] = f64::cos(f) as f32;
            }
        }
        Self { n: n }
    }

    pub fn synthesis_filter(
        &self,
        samples: &[f32; 576],
        pcm: &mut [f32],
        fifo: &mut [f32],
        ch: usize,
        ch_num: usize,
    ) {
        let mut s = [0f32; 32];
        let mut u = [0f32; 512];
        let mut w = [0f32; 512];

        for sb in 0..18 {
            for i in 0..32 {
                s[i] = samples[i * 18 + sb];
            }

            for i in (64..1024).rev() {
                fifo[i] = fifo[i - 64];
            }
            for i in 0..64 {
                fifo[i] = 0.0;
                for j in 0..32 {
                    fifo[i] += s[j] * self.n[i][j];
                }
            }

            for i in 0..8 {
                for j in 0..32 {
                    u[i * 64 + j] = fifo[i * 128 + j];
                    u[i * 64 + j + 32] = fifo[i * 128 + j + 96];
                }
            }
            for i in 0..512 {
                w[i] = u[i] * SYNTH_WINDOW[i];
            }

            for i in 0..32 {
                let mut sum = 0.0;
                for j in 0..16 {
                    sum += w[j * 32 + i];
                }
                // pcm[32 * sb + i] = sum;
                
                /* 将左右声道数据交替写入 */
                pcm[(32 * sb + i) * ch_num + ch] = sum;
            }
        }
    }
}