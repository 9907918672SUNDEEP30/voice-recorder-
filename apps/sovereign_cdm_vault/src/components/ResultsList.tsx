import React from 'react';
import { Layers, FolderCheck } from 'lucide-react';
import { Citation } from '../types/vault';
import { ResultCard } from './ResultCard';

interface ResultsListProps {
  results: Citation[];
  scopeLabel: string;
  onAskContext: (citation: Citation) => void;
  onApplyRename?: (citation: Citation) => void;
}

export const ResultsList: React.FC<ResultsListProps> = ({
  results,
  scopeLabel,
  onAskContext,
  onApplyRename,
}) => {
  if (results.length === 0) {
    return (
      <div className="py-16 text-center text-zinc-500">
        <Layers className="w-10 h-10 mx-auto text-zinc-600 mb-3 opacity-60" />
        <div className="text-sm font-medium text-zinc-400">No matching document chunks found in scope</div>
        <div className="text-xs text-zinc-600 mt-1 max-w-sm mx-auto">
          Try expanding scope to "This PC (All Connected Drives)" or searching for names, sections, or tax keywords.
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* Section Header */}
      <div className="flex items-center justify-between pb-2 border-b border-zinc-800/80">
        <div className="flex items-center gap-2">
          <FolderCheck className="w-4 h-4 text-amber-400" />
          <h2 className="text-xs font-bold tracking-wider uppercase text-zinc-300">
            CONTEXT RETRIEVAL MATCHES <span className="text-zinc-500 font-mono font-normal">(Scope: {scopeLabel})</span>
          </h2>
        </div>
        <div className="text-xs font-mono text-zinc-400">
          Found <span className="font-semibold text-emerald-400">{results.length}</span> primary evidence chunks
        </div>
      </div>

      {/* Results Cards List */}
      <div className="space-y-2.5">
        {results.map((citation) => (
          <ResultCard
            key={citation.chunkId}
            citation={citation}
            onAskContext={onAskContext}
            onApplyRename={onApplyRename}
          />
        ))}
      </div>
    </div>
  );
};
