//! G.711 μ-law Codec and Audio Resampler
//!
//! Provides 100% mathematically correct ITU-T G.711 μ-law encoding and decoding,
//! along with linear-interpolated sample rate conversion between 8kHz (Telephony standard),
//! 16kHz (Silero VAD / Whisper STT standard), and 22.05kHz (Piper TTS standard).

pub struct G711Codec;

/// Precomputed 256-entry μ-law to i16 linear PCM lookup table (ITU-T G.711 standard)
pub const ULAW_DECODE_TABLE: [i16; 256] = {
    let mut table = [0i16; 256];
    let mut i = 0usize;
    while i < 256 {
        let byte = !(i as u8);
        let sign = (byte & 0x80) != 0;
        let exponent = ((byte >> 4) & 0x07) as usize;
        let mantissa = (byte & 0x0F) as i32;
        let sample = (((mantissa << 3) + 0x84) << exponent) - 0x84;
        table[i] = if sign { -sample as i16 } else { sample as i16 };
        i += 1;
    }
    table
};

impl G711Codec {
    /// Decodes a single G.711 μ-law byte to 16-bit linear PCM.
    #[inline]
    #[allow(dead_code)]
    pub fn ulaw_to_linear(byte: u8) -> i16 {
        ULAW_DECODE_TABLE[byte as usize]
    }

    /// Encodes a single 16-bit linear PCM sample to G.711 μ-law byte.
    #[inline]
    pub fn linear_to_ulaw(mut sample: i16) -> u8 {
        const BIAS: i16 = 0x84;
        const CLIP: i16 = 32635;
        let mut sign = 0u8;
        if sample < 0 {
            sample = -sample;
            sign = 0x80;
        }
        if sample > CLIP {
            sample = CLIP;
        }
        sample += BIAS;

        let mut exponent = 7u8;
        let masks = [0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080];
        for (i, &mask) in masks.iter().enumerate() {
            if (sample & mask) != 0 {
                exponent = 7 - (i as u8);
                break;
            }
        }
        let mantissa = ((sample >> (exponent + 3)) & 0x0F) as u8;
        !(sign | (exponent << 4) | mantissa)
    }

    /// 8kHz μ-law bytes -> 16kHz Linear PCM (upsampled with linear interpolation)
    pub fn mulaw_8k_to_pcm_16k(mulaw: &[u8]) -> Vec<u8> {
        if mulaw.is_empty() {
            return Vec::new();
        }
        let mut pcm16 = Vec::with_capacity(mulaw.len() * 4);
        let mut prev_sample = ULAW_DECODE_TABLE[mulaw[0] as usize];
        for &byte in mulaw {
            let curr_sample = ULAW_DECODE_TABLE[byte as usize];
            // Linear interpolation between consecutive samples:
            let interp = ((prev_sample as i32 + curr_sample as i32) / 2) as i16;
            pcm16.extend_from_slice(&interp.to_le_bytes());
            pcm16.extend_from_slice(&curr_sample.to_le_bytes());
            prev_sample = curr_sample;
        }
        pcm16
    }

    /// Linear PCM at arbitrary sample rate (e.g. 16000 or 22050) -> 8kHz μ-law bytes.
    pub fn pcm_to_mulaw_8k(pcm: &[u8], in_sample_rate: u32) -> Vec<u8> {
        let num_samples = pcm.len() / 2;
        if num_samples == 0 {
            return Vec::new();
        }

        let ratio = in_sample_rate as f64 / 8000.0;
        let num_out = ((num_samples as f64) / ratio).round() as usize;
        let mut mulaw = Vec::with_capacity(num_out);

        for j in 0..num_out {
            let pos = (j as f64) * ratio;
            let idx = pos.floor() as usize;
            let frac = (pos - idx as f64) as f32;

            if idx >= num_samples {
                break;
            }

            let s0 = i16::from_le_bytes([pcm[idx * 2], pcm[idx * 2 + 1]]) as f32;
            let s1 = if idx + 1 < num_samples {
                i16::from_le_bytes([pcm[(idx + 1) * 2], pcm[(idx + 1) * 2 + 1]]) as f32
            } else {
                s0
            };

            let interp = (1.0 - frac) * s0 + frac * s1;
            let clamped = interp.clamp(-32768.0, 32767.0) as i16;
            mulaw.push(Self::linear_to_ulaw(clamped));
        }

        mulaw
    }

    /// 16kHz Linear PCM -> 8kHz μ-law bytes.
    #[inline]
    #[allow(dead_code)]
    pub fn pcm_16k_to_mulaw_8k(pcm: &[u8]) -> Vec<u8> {
        Self::pcm_to_mulaw_8k(pcm, 16000)
    }

    /// 22.05kHz Linear PCM (Piper TTS native) -> 8kHz μ-law bytes.
    #[inline]
    pub fn pcm_22k_to_mulaw_8k(pcm: &[u8]) -> Vec<u8> {
        Self::pcm_to_mulaw_8k(pcm, 22050)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ulaw_encode_decode_roundtrip() {
        // Zero
        let enc_zero = G711Codec::linear_to_ulaw(0);
        let dec_zero = G711Codec::ulaw_to_linear(enc_zero);
        assert_eq!(dec_zero, 0);

        // Sine wave test across dynamic range
        for sample in [-25000, -10000, -1000, 0, 1000, 10000, 25000] {
            let enc = G711Codec::linear_to_ulaw(sample);
            let dec = G711Codec::ulaw_to_linear(enc);
            // Mu-law 8-bit dynamic range has small quantization error
            let diff = (sample as i32 - dec as i32).abs();
            assert!(diff < 1500, "Sample {} diff too large: {}", sample, diff);
        }
    }

    #[test]
    fn test_mulaw_resampling() {
        let input_mulaw = vec![0xFF; 80]; // 80 bytes of silence @ 8kHz (10ms)
        let pcm16 = G711Codec::mulaw_8k_to_pcm_16k(&input_mulaw);
        assert_eq!(pcm16.len(), 80 * 4); // 160 samples * 2 bytes = 320 bytes

        let back_to_mulaw = G711Codec::pcm_16k_to_mulaw_8k(&pcm16);
        assert_eq!(back_to_mulaw.len(), 80);
    }
}
