use anyhow::Result;
use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};

pub struct AudioResampler {
    resampler: SincFixedIn<f32>,
}

impl AudioResampler {
    pub fn new(in_freq: usize, out_freq: usize, chunk_size: usize) -> Result<Self> {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        
        let resampler = SincFixedIn::<f32>::new(
            out_freq as f64 / in_freq as f64,
            2.0,
            params,
            chunk_size,
            1, // Mono channel
        ).map_err(|e| anyhow::anyhow!("DSP Error: {:?}", e))?;
        
        Ok(Self { resampler })
    }

    /// DSP Math: Resample PCM f32 from one frequency to another
    pub fn resample(&mut self, input_audio: &[f32]) -> Result<Vec<f32>> {
        let input_buffers = vec![input_audio.to_vec()];
        let output_buffers = self.resampler.process(&input_buffers, None)
            .map_err(|e| anyhow::anyhow!("DSP Error: {:?}", e))?;
            
        Ok(output_buffers[0].clone())
    }
}
