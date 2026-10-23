import React, { useRef, useEffect } from 'react';
import { Search, Sparkles, X, ArrowRight, Zap } from 'lucide-react';

interface SearchCommandBarProps {
  query: string;
  isSearching: boolean;
  tierUsed?: 'tier0_fast_path' | 'tier1_hybrid_rag';
  latencyMs?: number;
  onQueryChange: (query: string) => void;
  onSearch: (customQuery?: string) => void;
}

export const SearchCommandBar: React.FC<SearchCommandBarProps> = ({
  query,
  isSearching,
  tierUsed,
  latencyMs,
  onQueryChange,
  onSearch,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  // Ctrl + K global shortcut to focus input
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      onSearch();
    }
  };

  const sampleQueries = [
    'Sandeep 2019-20 plot purchase and capital gain',
    'Tax Audit Section 80C compliance rule',
    'GST Input Tax Credit Section 16(2) Rule 42',
  ];

  return (
    <div className="p-4 border-b border-zinc-800/80 bg-zinc-950/50">
      {/* Main Search Command Bar */}
      <div className="relative flex items-center">
        <div className="absolute left-3.5 flex items-center pointer-events-none text-zinc-500">
          <Search className="w-5 h-5 text-indigo-400" />
        </div>

        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder='Search or ask AI inside PDFs... (e.g. "Tax Audit Section 80C" or "Sandeep 2019-20 plot purchase")'
          className="w-full bg-zinc-900/90 border border-zinc-700/60 focus:border-indigo-500/80 focus:ring-2 focus:ring-indigo-500/20 text-zinc-100 placeholder-zinc-500 pl-11 pr-32 py-3 rounded-xl text-sm font-medium transition shadow-lg outline-none"
        />

        {/* Clear & Hotkey & Action Buttons */}
        <div className="absolute right-2.5 flex items-center space-x-1.5">
          {query && (
            <button
              type="button"
              onClick={() => onQueryChange('')}
              className="p-1 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-md transition"
              title="Clear Search"
            >
              <X className="w-4 h-4" />
            </button>
          )}

          <div className="hidden sm:flex items-center px-1.5 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-[10px] font-mono text-zinc-400">
            Ctrl + K
          </div>

          <button
            type="button"
            onClick={() => onSearch()}
            disabled={isSearching || !query.trim()}
            className="flex items-center gap-1.5 px-3.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg text-xs font-semibold transition shadow-md shadow-indigo-600/30"
          >
            {isSearching ? (
              <span className="flex items-center gap-1">
                <span className="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                <span>Searching...</span>
              </span>
            ) : (
              <span className="flex items-center gap-1">
                <span>Ask AI</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </span>
            )}
          </button>
        </div>
      </div>

      {/* Query Suggestions & Engine Mode Badge */}
      <div className="mt-2.5 flex flex-wrap items-center justify-between gap-2 text-xs">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-zinc-500 text-[11px] font-medium flex items-center gap-1">
            <Sparkles className="w-3 h-3 text-amber-400" />
            <span>Try asking:</span>
          </span>
          {sampleQueries.map((sample, idx) => (
            <button
              key={idx}
              type="button"
              onClick={() => {
                onQueryChange(sample);
                onSearch(sample);
              }}
              className="px-2 py-0.5 rounded-full bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-400 hover:text-zinc-200 text-[11px] transition"
            >
              "{sample}"
            </button>
          ))}
        </div>

        {/* Engine Pipeline Badge */}
        {tierUsed && (
          <div className="flex items-center gap-2">
            <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono font-medium border ${
              tierUsed === 'tier0_fast_path'
                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                : 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20'
            }`}>
              <Zap className="w-3 h-3" />
              {tierUsed === 'tier0_fast_path'
                ? '⚡ Tier-0 Fast-Path (Rust Metascan)'
                : '🧠 Tier-1 Contextual Hybrid (BM25 + HNSW + RRF)'}
            </span>
            {latencyMs !== undefined && (
              <span className="text-zinc-500 font-mono text-[10px]">
                {latencyMs.toFixed(1)}ms
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
