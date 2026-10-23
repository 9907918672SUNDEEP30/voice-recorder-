import React, { useState } from 'react';
import { FileText, FolderOpen, Copy, Check, Sparkles, Tag, ArrowUpRight } from 'lucide-react';
import { Citation } from '../types/vault';
import { copyToClipboard, openInExplorer } from '../lib/tauri';

interface ResultCardProps {
  citation: Citation;
  onAskContext: (citation: Citation) => void;
  onApplyRename?: (citation: Citation) => void;
}

export const ResultCard: React.FC<ResultCardProps> = ({
  citation,
  onAskContext,
  onApplyRename,
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopyPath = async () => {
    const success = await copyToClipboard(citation.filePath);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleOpenFolder = () => {
    openInExplorer(citation.filePath);
  };

  const scorePercentage = Math.round(citation.score * 100);

  return (
    <div className="glass-panel glass-panel-hover rounded-xl p-4 transition duration-200">
      {/* Top Header Row */}
      <div className="flex items-start justify-between gap-3 mb-2.5">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="p-2 rounded-lg bg-zinc-800/80 border border-zinc-700/60 text-indigo-400 shrink-0">
            <FileText className="w-4 h-4" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-sm text-zinc-100 truncate font-mono">
                {citation.filename}
              </span>
              {citation.pageNumber !== undefined && (
                <span className="px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300 text-[11px] font-mono border border-zinc-700/60 shrink-0">
                  Page {citation.pageNumber}
                </span>
              )}
            </div>
            <div className="text-[11px] text-zinc-400 truncate flex items-center gap-1.5 mt-0.5">
              <span className="text-zinc-500">{citation.category}</span>
              {citation.smartAlias && (
                <>
                  <span className="text-zinc-600">•</span>
                  <span className="text-indigo-400/90 font-mono text-[10px] flex items-center gap-1">
                    <Tag className="w-2.5 h-2.5" />
                    Alias: {citation.smartAlias}
                  </span>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Score Pill */}
        <div className="shrink-0 flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 font-mono text-xs font-semibold">
          <span>Score: {citation.score.toFixed(2)}</span>
          <span className="text-[10px] opacity-75">({scorePercentage}%)</span>
        </div>
      </div>

      {/* Suggested Smart Rename Banner (for messy scanned files) */}
      {citation.needsRenaming && citation.suggestedName && (
        <div className="mb-3 px-3 py-2 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-between gap-2 text-xs">
          <div className="flex items-center gap-2 text-amber-300 min-w-0">
            <Sparkles className="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span className="truncate">
              Auto-Detected: <span className="font-mono font-medium">{citation.suggestedName}</span>
            </span>
          </div>
          {onApplyRename && (
            <button
              type="button"
              onClick={() => onApplyRename(citation)}
              className="px-2 py-0.5 bg-amber-500 hover:bg-amber-400 text-zinc-950 font-semibold rounded text-[11px] transition shrink-0"
            >
              Rename on Disk
            </button>
          )}
        </div>
      )}

      {/* Excerpt / Chunk Content */}
      <div className="bg-zinc-950/70 border border-zinc-800/80 rounded-lg p-3 text-xs text-zinc-300 leading-relaxed font-sans mb-3 select-text">
        <p className="line-clamp-3">"{citation.content}"</p>
      </div>

      {/* Action Bar */}
      <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-zinc-800/60 text-xs">
        <div className="flex items-center space-x-1.5">
          <button
            type="button"
            onClick={handleOpenFolder}
            className="flex items-center gap-1.5 px-2.5 py-1 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 hover:text-white rounded-md transition"
            title="Open containing folder in Windows Explorer"
          >
            <FolderOpen className="w-3.5 h-3.5 text-amber-400" />
            <span>Open File Folder</span>
          </button>

          <button
            type="button"
            onClick={handleCopyPath}
            className="flex items-center gap-1.5 px-2.5 py-1 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 hover:text-white rounded-md transition font-mono"
            title="Copy absolute file path"
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-emerald-400" />
                <span className="text-emerald-400">Copied!</span>
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5 text-zinc-400" />
                <span>Copy Path</span>
              </>
            )}
          </button>
        </div>

        <button
          type="button"
          onClick={() => onAskContext(citation)}
          className="flex items-center gap-1.5 px-3 py-1 bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/30 text-indigo-300 hover:text-indigo-200 rounded-md font-medium transition"
        >
          <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
          <span>Ask AI Context on this Document</span>
          <ArrowUpRight className="w-3 h-3 text-indigo-400" />
        </button>
      </div>
    </div>
  );
};
