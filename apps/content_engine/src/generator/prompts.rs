use crate::recon::TrendDossier;

pub struct PromptEngine;

impl PromptEngine {
    pub fn build_short_script_prompt(dossier: &TrendDossier) -> String {
        let hook_sample = dossier
            .detected_viral_hooks
            .first()
            .cloned()
            .unwrap_or_else(|| "Stop scrolling right now...".to_string());

        let tools = dossier.reverse_engineered_tools.join(", ");

        format!(
            "You are a master viral short-form video creator (YouTube Shorts, Instagram Reels, TikTok).\n\
            Topic: {topic}\n\
            Recommended Hook: {hook_sample}\n\
            Key Tools/Mechanics: {tools}\n\
            Target Audience Pain Point: {pain_point}\n\n\
            Task: Write a 60-second high-retention script with exact visual cues.\n\
            Structure:\n\
            00:00-00:03: Disruptive Pattern-Interrupt Hook\n\
            00:03-00:15: The Problem & Curiosity Gap\n\
            00:15-00:45: The Secret Method / Practical Tutorial\n\
            00:45-00:55: Visual Proof or Side-by-Side Comparison\n\
            00:55-01:00: High-Converting Call to Action\n\
            Tone: Confident, fast-paced, high value, zero fluff.\n",
            topic = dossier.topic,
            hook_sample = hook_sample,
            tools = tools,
            pain_point = dossier.target_audience_pain_point
        )
    }

    pub fn build_long_script_prompt(dossier: &TrendDossier) -> String {
        format!(
            "You are a world-class YouTube tech & trend documentary creator.\n\
            Topic: {topic}\n\
            Angle: {angle}\n\
            Key Facts: {facts:?}\n\n\
            Task: Write an in-depth 8-10 minute YouTube video script.\n\
            Include:\n\
            1. Chapter Timestamps\n\
            2. High-retention intro hook (First 30 seconds)\n\
            3. Deep technical/market breakdown\n\
            4. Step-by-step tutorial or analysis\n\
            5. Potential risks & future outlook\n\
            6. Outro & engagement question\n",
            topic = dossier.topic,
            angle = dossier.recommended_angle,
            facts = dossier.breaking_news.iter().map(|n| &n.title).take(3).collect::<Vec<_>>()
        )
    }

    pub fn build_blog_article_prompt(dossier: &TrendDossier) -> String {
        format!(
            "You are a top-tier SEO technical writer.\n\
            Topic: {topic}\n\
            Tools identified: {tools:?}\n\n\
            Task: Write a 1,200-word comprehensive, SEO-optimized blog article in Markdown.\n\
            Requirements:\n\
            - Single H1 Title Tag (High CTR)\n\
            - Meta Description (150 chars)\n\
            - H2 and H3 structured headings\n\
            - Step-by-step tutorial walkthrough\n\
            - Tool Comparison Table\n\
            - FAQ Section (3 questions for Google rich snippets)\n",
            topic = dossier.topic,
            tools = dossier.reverse_engineered_tools
        )
    }

    pub fn build_twitter_thread_prompt(dossier: &TrendDossier) -> String {
        format!(
            "You are a viral Twitter (X) growth strategist.\n\
            Topic: {topic}\n\
            Hook: {hook}\n\n\
            Task: Write a 6-tweet viral thread designed for bookmarking and retweets.\n\
            Tweet 1: Strong curiosity hook (e.g., 'Everything you were told about {topic} is wrong. Here's what's actually happening (and how to profit/benefit): 🧵')\n\
            Tweets 2-4: Actionable steps and tool breakdowns\n\
            Tweet 5: Key mistakes to avoid\n\
            Tweet 6: Summary infographic prompt and CTA to follow.\n",
            topic = dossier.topic,
            hook = dossier.detected_viral_hooks.first().unwrap_or(&dossier.topic)
        )
    }
}
