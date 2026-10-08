use anyhow::Result;

pub struct G711Codec;

// Precomputed 256-entry μ-law to i16 linear PCM lookup table
// Dummy implementation for structure
static ULAW_DECODE_TABLE: [i16; 256] = [0; 256];

impl G711Codec {
    /// 8kHz μ-law bytes -> 16kHz Linear PCM (duplication for 2x upsampling)
    pub fn mulaw_8k_to_pcm_16k(mulaw: &[u8]) -> Vec<u8> {
        let mut pcm16 = Vec::with_capacity(mulaw.len() * 4);
        for &byte in mulaw {
            let sample = ULAW_DECODE_TABLE[byte as usize];
            let le_bytes = sample.to_le_bytes();
            // 2x upsample: duplicate sample
            pcm16.extend_from_slice(&le_bytes);
            pcm16.extend_from_slice(&le_bytes);
        }
        pcm16
    }

    /// 16kHz Linear PCM -> 8kHz μ-law bytes (downsampling step 2)
    pub fn pcm_16k_to_mulaw_8k(pcm: &[u8]) -> Vec<u8> {
        let mut mulaw = Vec::with_capacity(pcm.len() / 4);
        for chunk in pcm.chunks_exact(4) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            mulaw.push(Self::linear_to_ulaw(sample));
        }
        mulaw
    }

    #[inline]
    fn linear_to_ulaw(mut sample: i16) -> u8 {
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
}
