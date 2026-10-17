use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct WordTimestamp {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

pub struct SubtitleEngine;

impl SubtitleEngine {
    /// Generates word-level timestamps from text and total audio duration
    pub fn align_word_timestamps(text: &str, total_duration_secs: f32) -> Vec<WordTimestamp> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Vec::new();
        }

        let total_ms = (total_duration_secs * 1000.0) as u64;
        let ms_per_word = (total_ms / words.len() as u64).max(200);

        let mut timestamps = Vec::with_capacity(words.len());
        let mut cur_ms = 0u64;

        for w in words {
            let end_ms = (cur_ms + ms_per_word).min(total_ms);
            timestamps.push(WordTimestamp {
                word: w.to_string(),
                start_ms: cur_ms,
                end_ms,
            });
            cur_ms = end_ms;
        }

        timestamps
    }

    /// Compiles word timestamps into dynamic glowing ASS Karaoke subtitles (Viral Hormozi Style)
    pub fn export_ass_karaoke(
        timestamps: &[WordTimestamp],
        output_ass_path: &Path,
    ) -> Result<()> {
        let mut ass = String::new();

        // Standard ASS Script Info & Styles Header
        ass.push_str("[Script Info]\nTitle: Viral Karaoke Subtitles\nScriptType: v4.00+\nPlayResX: 1080\nPlayResY: 1920\n\n");
        ass.push_str("[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        // Style: Bold Montserrat/Arial, White text, Glowing Yellow active highlight (&H0000FFFF), Deep Outline (&H00000000)
        ass.push_str("Style: Default,Arial Black,72,&H00FFFFFF,&H0000FFFF,&H00000000,&H80000000,-1,0,0,0,100,100,2,0,1,6,3,2,60,60,350,1\n\n");
        ass.push_str("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

        // Chunk words into groups of 3-4 words for high-retention rapid reading
        for chunk in timestamps.chunks(4) {
            if chunk.is_empty() {
                continue;
            }
            let start_time = format_ass_time(chunk.first().unwrap().start_ms);
            let end_time = format_ass_time(chunk.last().unwrap().end_ms);

            // Construct karaoke text with animated highlight
            let mut line_text = String::new();
            for word_item in chunk {
                let duration_cs = ((word_item.end_ms - word_item.start_ms) / 10).max(1);
                line_text.push_str(&format!("{{\\k{}}}{} ", duration_cs, word_item.word));
            }

            ass.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                start_time,
                end_time,
                line_text.trim()
            ));
        }

        std::fs::write(output_ass_path, ass)?;
        println!("  ✅ Glowing ASS Karaoke subtitles exported to: {:?}", output_ass_path);
        Ok(())
    }

    /// Compiles word timestamps into standard SRT format
    pub fn export_srt(timestamps: &[WordTimestamp], output_srt_path: &Path) -> Result<()> {
        let mut srt = String::new();
        let mut index = 1;

        for chunk in timestamps.chunks(5) {
            if chunk.is_empty() {
                continue;
            }
            let start = format_srt_time(chunk.first().unwrap().start_ms);
            let end = format_srt_time(chunk.last().unwrap().end_ms);
            let text = chunk
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            srt.push_str(&format!("{}\n{} --> {}\n{}\n\n", index, start, end, text));
            index += 1;
        }

        std::fs::write(output_srt_path, srt)?;
        Ok(())
    }
}

fn format_ass_time(ms: u64) -> String {
    let cs = (ms % 1000) / 10;
    let s = (ms / 1000) % 60;
    let m = (ms / 60000) % 60;
    let h = ms / 3600000;
    format!("{}:{:02}:{:02}.{:02}", h, m, s, cs)
}

fn format_srt_time(ms: u64) -> String {
    let millis = ms % 1000;
    let s = (ms / 1000) % 60;
    let m = (ms / 60000) % 60;
    let h = ms / 3600000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}
