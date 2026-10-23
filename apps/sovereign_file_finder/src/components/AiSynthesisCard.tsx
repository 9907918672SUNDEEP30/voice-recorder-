import React, { useState } from 'react';
import { Bot, Copy, Check, Sparkles, Layers, ShieldCheck } from 'lucide-react';
import { AiSynthesis } from '../types/vault';
import { copyToClipboard } from '../lib/tauri';

interface AiSynthesisCardProps {
  synthesis: AiSynthesis;
}

export const AiSynthesisCard: React.FC<AiSynthesisCardProps> = ({ synthesis }) => {
  const [lang, setLang] = useState<'hindi' | 'english'>('hindi');
  const [copied, setCopied] = useState(false);

  const activeText = lang === 'hindi' ? synthesis.summaryHindi : synthesis.summaryEnglish;

  const handleCopy = async () => {
    const success = await copyToClipboard(activeText);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="rounded-xl border border-indigo-500/30 bg-gradient-to-b from-indigo-950/20 via-zinc-900/90 to-zinc-950/90 p-4 shadow-xl backdrop-blur-xl">
      {/* Header */}
      <div className="flex items-center justify-between pb-3 border-b border-zinc-800/80 mb-3">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded-lg bg-indigo-500/20 border border-indigo-500/30 text-indigo-400">
            <Bot className="w-4 h-4" />
          </div>
          <span className="font-bold text-xs uppercase tracking-wider text-indigo-300">
            AI SYNTHESIS OUTPUT
          </span>
          <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-mono">
            {Math.round(synthesis.confidence * 100)}% Confidence
          </span>
        </div>

        {/* Language Tabs & Copy */}
        <div className="flex items-center space-x-2">
          <div className="flex items-center bg-zinc-900 p-0.5 rounded-lg border border-zinc-800 text-xs">
            <button
              type="button"
              onClick={() => setLang('hindi')}
              className={`px-2.5 py-1 rounded-md text-xs font-medium transition ${
                lang === 'hindi'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              हिन्दी
            </button>
            <button
              type="button"
              onClick={() => setLang('english')}
              className={`px-2.5 py-1 rounded-md text-xs font-medium transition ${
                lang === 'english'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              English
            </button>
          </div>

          <button
            type="button"
            onClick={handleCopy}
            className="p-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-lg transition"
            title="Copy Synthesis"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Synthesis Text Body */}
      <div className="p-3.5 rounded-lg bg-zinc-950/80 border border-zinc-800/80 text-sm leading-relaxed text-zinc-100 select-text font-sans">
        <p className="whitespace-pre-line">{activeText}</p>
      </div>

      {/* Verification & DirectML Footer */}
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-[11px] text-zinc-500">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1 text-emerald-400/90 font-mono">
            <ShieldCheck className="w-3 h-3 text-emerald-400" />
            Verified against {synthesis.citations.length} primary source citations
          </span>
          <span className="flex items-center gap-1 font-mono text-zinc-400">
            <Layers className="w-3 h-3" />
            {synthesis.model}
          </span>
        </div>
        <div className="flex items-center gap-1 text-indigo-400/80 text-[10px]">
          <Sparkles className="w-2.5 h-2.5" />
          <span>Zero external SaaS • 100% On-Device Sovereign Privacy</span>
        </div>
      </div>
    </div>
  );
};
