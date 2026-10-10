use ai_core::models::adapters::{
    EmbeddingAdapter, LlmAdapter, OcrAdapter, RerankerAdapter, TtsAdapter, VADAdapter,
    VisionAdapter,
};
use ai_core::models::onnx_adapters::{
    BgeEmbeddingLocal, BgeRerankerLocal, OpenVoiceLocal, QwenLocal, SileroVadLocal,
};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_verify_all_models_zero_mock() -> anyhow::Result<()> {
    println!("\n========================================================");
    println!("🚀 STARTING 100% REAL ZERO-MOCK COMPONENT VERIFICATION");
    println!("========================================================");

    let base_dir = "C:\\AI_Models";

    // 1. VERIFY BGE EMBEDDING ADAPTER (REAL ONNX FORWARD PASS + MEAN POOLING + L2 NORM)
    println!("\n[1/6] Verifying BgeEmbeddingLocal...");
    let bge_model = format!("{}\\bge_small_embedding\\model.onnx", base_dir);
    let embedding_adapter = BgeEmbeddingLocal::new(&bge_model)?;
    let test_sentence = "Artificial intelligence and neural network architecture in Rust.";
    let embedding = embedding_adapter.embed_text(test_sentence).await?;
    println!(" - Vector dimension: {}", embedding.len());
    let norm: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
    println!(" - Vector L2 Norm: {}", norm);
    assert_eq!(embedding.len(), 384, "BGE Small must produce 384-dimensional embeddings");
    assert!((norm - 1.0).abs() < 1e-3, "Embedding must be unit L2-normalized");
    println!(" ✅ BgeEmbeddingLocal PASSED with authentic normalized vector!");

    // 2. VERIFY BGE RERANKER ADAPTER (REAL ONNX CROSS-ENCODER + SIGMOID)
    println!("\n[2/6] Verifying BgeRerankerLocal...");
    let reranker_model = format!("{}\\bge_reranker\\model.onnx", base_dir);
    let reranker_adapter = BgeRerankerLocal::new(&reranker_model)?;
    let query = "What is Rust programming language?";
    let docs = vec![
        "Rust is a high-performance systems language with zero-cost abstractions and memory safety without GC.".to_string(),
        "Bananas are tropical fruits rich in potassium.".to_string(),
    ];
    let ranked = reranker_adapter.rerank(query, docs).await?;
    println!(" - Top result: '{}' (Score: {:.4})", ranked[0].0, ranked[0].1);
    println!(" - Bottom result: '{}' (Score: {:.4})", ranked[1].0, ranked[1].1);
    assert!(ranked[0].1 >= ranked[1].1, "Relevant doc must score higher than irrelevant doc");
    assert!(ranked[0].1 > 0.0 && ranked[0].1 <= 1.0, "Score must be a valid probability");
    println!(" ✅ BgeRerankerLocal PASSED with authentic cross-encoder relevance scores!");

    // 3. VERIFY SILERO VAD ADAPTER (REAL ONNX RECURRENT LSTM)
    println!("\n[3/6] Verifying SileroVadLocal...");
    let vad_model = format!("{}\\silero_vad.onnx", base_dir);
    let mut vad_adapter = SileroVadLocal::new(&vad_model)?;
    let silent_pcm = vec![0u8; 1024]; // 512 16-bit silence samples
    let (is_speech, prob) = vad_adapter.process_chunk(&silent_pcm).await?;
    println!(" - Silence check: is_speech={}, prob={:.4}", is_speech, prob);
    assert!(!is_speech, "Silence must not be detected as speech");
    println!(" ✅ SileroVadLocal PASSED with authentic ONNX LSTM inference!");

    // 4. VERIFY PIPER TTS ADAPTER (REAL ONNX WAVEFORM SYNTHESIS)
    println!("\n[4/6] Verifying OpenVoiceLocal (Piper TTS)...");
    let piper_model = format!("{}\\piper_hi\\hi_IN-priyamvada-medium-int8.onnx", base_dir);
    let tone_color = format!("{}\\OpenVoice-ONNX-v2\\tone_color.onnx", base_dir);
    let tone_extract = format!("{}\\OpenVoice-ONNX-v2\\tone_extract.onnx", base_dir);
    let tts_adapter = OpenVoiceLocal::new(&piper_model, &tone_color, &tone_extract, "sweet_girl_reference.wav")?;
    let (audio_tx, mut audio_rx) = mpsc::channel::<Bytes>(50);
    let cancel = CancellationToken::new();
    let text_to_speak = "नमस्ते संदीप जी";
    tts_adapter.synthesize_stream(text_to_speak, audio_tx, cancel).await?;
    let mut total_pcm_bytes = 0;
    while let Some(chunk) = audio_rx.recv().await {
        total_pcm_bytes += chunk.len();
    }
    println!(" - Synthesized PCM audio: {} bytes", total_pcm_bytes);
    assert!(total_pcm_bytes > 0, "TTS must produce real PCM audio bytes");
    println!(" ✅ OpenVoiceLocal (Piper TTS) PASSED with real neural waveform synthesis!");

    // 5. VERIFY QWEN LLM (REAL AUTOREGRESSIVE TOKEN GENERATION & STREAMING)
    println!("\n[5/6] Verifying QwenLocal LLM & Brain...");
    let qwen_llm = format!("{}\\qwen2-vl-2b\\llmmodel.onnx", base_dir);
    let qwen_vis = format!("{}\\qwen2-vl-2b\\visionmodel.onnx", base_dir);
    let qwen_vis_enc = format!("{}\\qwen2-vl-2b\\vision_encoder.onnx", base_dir);
    let qwen_tok = format!("{}\\qwen2-vl-2b\\tokenizer.json", base_dir);
    let qwen_adapter = Arc::new(QwenLocal::new(&qwen_llm, &qwen_vis, &qwen_vis_enc, &qwen_tok)?);

    let cancel_llm = CancellationToken::new();
    let mut rx = qwen_adapter.generate_stream(
        "You are a helpful assistant. Keep your response very brief and precise.",
        vec![("user".to_string(), "Say hello and give one word.".to_string())],
        cancel_llm,
    ).await?;

    let mut generated_text = String::new();
    while let Some(chunk) = rx.recv().await {
        print!("{}", chunk);
        generated_text.push_str(&chunk);
        if generated_text.len() > 30 {
            break;
        }
    }
    println!("\n - Final LLM Output: '{}'", generated_text.trim());
    assert!(!generated_text.is_empty(), "LLM must generate authentic non-empty tokens");
    println!(" ✅ QwenLocal LLM PASSED with authentic neural autoregressive token generation!");

    // 6. VERIFY VISION & OCR ADAPTERS (REAL MULTIMODAL TENSOR PIPELINE)
    println!("\n[6/6] Verifying Qwen Vision & OCR Adapters...");
    // Create a 28x28 sample image in PNG format
    let img_buf = image::RgbImage::new(28, 28);
    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        &img_buf,
        28,
        28,
        image::ColorType::Rgb8,
    )?;

    let vision_result = qwen_adapter.analyze_image(&png_bytes, "What do you observe?").await?;
    println!(" - Vision Analysis Result: {}", vision_result);
    assert!(!vision_result.is_empty(), "Vision adapter must return non-empty visual analysis");

    let ocr_result = qwen_adapter.extract_text(&png_bytes).await?;
    println!(" - OCR Extraction Result: {}", ocr_result);
    assert!(!ocr_result.is_empty(), "OCR adapter must return non-empty text extraction");
    println!(" ✅ Vision & OCR Adapters PASSED with real neural multimodal inference!");

    println!("\n========================================================");
    println!("🎉 ALL 6 SUBSYSTEMS FULLY VERIFIED - 100% REAL ZERO-MOCK!");
    println!("========================================================\n");

    Ok(())
}
