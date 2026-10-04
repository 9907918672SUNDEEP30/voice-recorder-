use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use std::path::Path;

/// ऑडियो को मैनेज करने के लिए ऑडियो ब्रिज (cpal और hound का उपयोग)
pub struct AudioBridge;

impl AudioBridge {
    /// सिस्टम के डिफ़ॉल्ट माइक्रोफ़ोन को डिटेक्ट करने का फंक्शन
    pub fn check_microphone() -> Result<()> {
        let host = cpal::default_host();
        
        // डिफ़ॉल्ट इनपुट डिवाइस (माइक्रोफ़ोन) खोजें
        let device = host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("❌ सिस्टम पर कोई डिफ़ॉल्ट माइक्रोफ़ोन नहीं मिला!"))?;

        println!("🎤 माइक्रोफ़ोन सफलतापूर्वक मिल गया है!");

        // ऑडियो कॉन्फ़िगरेशन की जाँच
        let config = device.default_input_config()?;
        println!("📊 ऑडियो फॉर्मेट: {:?} चैनल, {} Hz", config.channels(), config.sample_rate());

        Ok(())
    }

    /// WAV फाइल में ऑडियो सेव करने के लिए hound का सेटअप
    pub fn save_to_wav<P: AsRef<Path>>(file_path: P, samples: &[f32], sample_rate: u32) -> Result<()> {
        let path = file_path.as_ref();
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;
        for &sample in samples {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }
        writer.finalize()?;

        println!("💾 ऑडियो सफलतापूर्वक WAV फाइल में सेव हो गया: {:?}", path);
        Ok(())
    }
}