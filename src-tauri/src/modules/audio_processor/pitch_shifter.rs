//! 相位声码器实现 - 用于高质量升降调
//!
//! 使用 STFT + 相位声码器算法实现 pitch shifting

use rustfft::{FftPlanner, FftDirection};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::f64::consts::PI;

/// 相位声码器配置
pub struct PhaseVocoderConfig {
    /// 窗口大小 (FFT size)
    pub window_size: usize,
    /// 跳跃大小 (hop size)
    pub hop_size: usize,
    /// 采样率
    pub sample_rate: u32,
}

impl Default for PhaseVocoderConfig {
    fn default() -> Self {
        Self {
            window_size: 2048,
            hop_size: 512,
            sample_rate: 44100,
        }
    }
}

/// 相位声码器
pub struct PhaseVocoder {
    config: PhaseVocoderConfig,
    /// 汉宁窗
    window: Vec<f64>,
    /// 上一帧的相位（用于相位累积）
    prev_phase: Vec<f64>,
    /// 输出缓冲
    output_buffer: Vec<f64>,
    /// FFT planner
    fft_planner: FftPlanner<f64>,
}

impl PhaseVocoder {
    pub fn new(config: PhaseVocoderConfig) -> Self {
        let window_size = config.window_size;

        // 创建汉宁窗
        let window: Vec<f64> = (0..window_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (window_size - 1) as f64).cos()))
            .collect();

        Self {
            config,
            window,
            prev_phase: vec![0.0; window_size / 2 + 1],
            output_buffer: Vec::new(),
            fft_planner: FftPlanner::new(),
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.prev_phase.fill(0.0);
        self.output_buffer.clear();
    }

    /// 计算半音对应的频率比例
    pub fn semitones_to_ratio(semitones: i32) -> f64 {
        2.0_f64.powf(semitones as f64 / 12.0)
    }

    /// 对单个音频块进行 pitch shifting
    /// input: 输入样本
    /// pitch_ratio: 音调变化比例 (>1 升调, <1 降调)
    pub fn process(&mut self, input: &[f64], pitch_ratio: f64) -> Vec<f64> {
        let window_size = self.config.window_size;
        let hop_size = self.config.hop_size;

        if input.len() < window_size {
            return input.to_vec();
        }

        // 分析阶段：STFT
        let num_frames = (input.len() - window_size) / hop_size + 1;
        let mut frames: Vec<Vec<Complex<f64>>> = Vec::with_capacity(num_frames);
        let mut magnitudes: Vec<Vec<f64>> = Vec::with_capacity(num_frames);
        let mut phases: Vec<Vec<f64>> = Vec::with_capacity(num_frames);

        let fft = self.fft_planner.plan_fft_forward(window_size);

        for frame_idx in 0..num_frames {
            let start = frame_idx * hop_size;
            let mut frame: Vec<Complex<f64>> = input[start..start + window_size]
                .iter()
                .zip(self.window.iter())
                .map(|(&s, &w)| Complex::new(s * w, 0.0))
                .collect();

            fft.process(&mut frame);
            frames.push(frame.clone());

            // 提取幅度和相位
            let mut mag: Vec<f64> = Vec::with_capacity(window_size / 2 + 1);
            let mut phase: Vec<f64> = Vec::with_capacity(window_size / 2 + 1);

            for i in 0..=window_size / 2 {
                mag.push(frames[frame_idx][i].norm());
                phase.push(frames[frame_idx][i].arg());
            }

            magnitudes.push(mag);
            phases.push(phase);
        }

        // 相位声码器处理
        let synthesis_hop = (hop_size as f64 * pitch_ratio) as usize;
        let num_output_frames = num_frames;

        // 合成阶段
        let ifft = self.fft_planner.plan_fft_inverse(window_size);
        let mut output = vec![0.0; num_output_frames * synthesis_hop + window_size];

        for frame_idx in 0..num_output_frames {
            // 重建频谱
            let mut frame: Vec<Complex<f64>> = vec![Complex::zero(); window_size];

            // 频率缩放
            for i in 0..=window_size / 2 {
                let new_bin = (i as f64 / pitch_ratio) as usize;
                if new_bin <= window_size / 2 {
                    let mag = magnitudes[frame_idx][new_bin.min(magnitudes[frame_idx].len() - 1)];
                    let phase = phases[frame_idx][new_bin.min(phases[frame_idx].len() - 1)];
                    frame[i] = Complex::from_polar(mag, phase);

                    // 镜像
                    if i > 0 && i < window_size / 2 {
                        frame[window_size - i] = frame[i].conj();
                    }
                }
            }

            // IFFT
            ifft.process(&mut frame);

            // 重叠加窗
            let start = frame_idx * synthesis_hop;
            for i in 0..window_size {
                if start + i < output.len() {
                    output[start + i] += frame[i].re * self.window[i] / window_size as f64;
                }
            }
        }

        output
    }
}

/// WSOLA 时间拉伸
pub struct WsolaTimeStretcher {
    window_size: usize,
    hop_size: usize,
    /// 搜索范围
    search_range: usize,
}

impl WsolaTimeStretcher {
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        Self {
            window_size,
            hop_size,
            search_range: hop_size / 2,
        }
    }

    /// 时间拉伸
    /// input: 输入样本
    /// stretch_ratio: 拉伸比例 (>1 变慢, <1 变快)
    pub fn stretch(&self, input: &[f64], stretch_ratio: f64) -> Vec<f64> {
        if input.len() < self.window_size * 2 {
            return input.to_vec();
        }

        let analysis_hop = self.hop_size;
        let synthesis_hop = (self.hop_size as f64 / stretch_ratio) as usize;

        let num_frames = (input.len() - self.window_size) / analysis_hop;
        let output_len = num_frames * synthesis_hop + self.window_size;
        let mut output = vec![0.0; output_len];
        let mut ola_norm = vec![0.0; output_len];

        // 汉宁窗
        let window: Vec<f64> = (0..self.window_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (self.window_size - 1) as f64).cos()))
            .collect();

        let mut prev_offset = 0;

        for frame_idx in 0..num_frames {
            // 找最佳匹配位置
            let nominal_pos = frame_idx * analysis_hop;
            let search_start = if frame_idx == 0 {
                0
            } else {
                (prev_offset + analysis_hop).saturating_sub(self.search_range)
            };
            let search_end = (nominal_pos + self.search_range).min(input.len() - self.window_size);

            let best_offset = if frame_idx == 0 {
                0
            } else {
                self.find_best_match(input, prev_offset + analysis_hop, search_start, search_end)
            };

            prev_offset = best_offset;

            // 重叠加窗
            let out_start = frame_idx * synthesis_hop;
            for i in 0..self.window_size {
                if best_offset + i < input.len() && out_start + i < output.len() {
                    output[out_start + i] += input[best_offset + i] * window[i];
                    ola_norm[out_start + i] += window[i] * window[i];
                }
            }
        }

        // 归一化
        for i in 0..output.len() {
            if ola_norm[i] > 1e-10 {
                output[i] /= ola_norm[i];
            }
        }

        output
    }

    fn find_best_match(&self, input: &[f64], ref_pos: usize, search_start: usize, search_end: usize) -> usize {
        let mut best_pos = search_start;
        let mut best_corr = f64::NEG_INFINITY;

        for pos in search_start..=search_end {
            let corr = self.cross_correlation(input, ref_pos, pos);
            if corr > best_corr {
                best_corr = corr;
                best_pos = pos;
            }
        }

        best_pos
    }

    fn cross_correlation(&self, input: &[f64], pos1: usize, pos2: usize) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.window_size {
            if pos1 + i < input.len() && pos2 + i < input.len() {
                sum += input[pos1 + i] * input[pos2 + i];
            }
        }
        sum
    }
}

/// 音调变换器（组合 WSOLA 和相位声码器）
pub struct PitchShifter {
    phase_vocoder: PhaseVocoder,
    sample_rate: u32,
}

impl PitchShifter {
    pub fn new(sample_rate: u32) -> Self {
        let config = PhaseVocoderConfig {
            window_size: 2048,
            hop_size: 512,
            sample_rate,
        };

        Self {
            phase_vocoder: PhaseVocoder::new(config),
            sample_rate,
        }
    }

    /// 改变音调（半音）
    /// 正数升调，负数降调
    pub fn shift_pitch(&mut self, input: &[f64], semitones: i32) -> Vec<f64> {
        if semitones == 0 {
            return input.to_vec();
        }

        let pitch_ratio = PhaseVocoder::semitones_to_ratio(semitones);

        // 使用相位声码器进行 pitch shifting
        self.phase_vocoder.process(input, pitch_ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semitones_to_ratio() {
        // 12个半音 = 一个八度 = 频率翻倍
        let ratio = PhaseVocoder::semitones_to_ratio(12);
        assert!((ratio - 2.0).abs() < 0.001);

        // 0个半音 = 不变
        let ratio = PhaseVocoder::semitones_to_ratio(0);
        assert!((ratio - 1.0).abs() < 0.001);
    }
}
