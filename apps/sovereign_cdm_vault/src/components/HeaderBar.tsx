import React from 'react';
import { ShieldCheck, Cpu, Wand2 } from 'lucide-react';
import { ScopeSelector } from './ScopeSelector';
import { DriveInfo, HardwareDrmStatus, ScopeConfig } from '../types/vault';

interface HeaderBarProps {
  currentScope: ScopeConfig;
  drives: DriveInfo[];
  recentPaths: string[];
  drmStatus: HardwareDrmStatus | null;
  onSelectScope: (scope: ScopeConfig) => void;
  onBrowseCustom: () => void;
  onAutoClean: () => void;
  uncleanedCount: number;
}

export const HeaderBar: React.FC<HeaderBarProps> = ({
  currentScope,
  drives,
  recentPaths,
  drmStatus,
  onSelectScope,
  onBrowseCustom,
  onAutoClean,
  uncleanedCount,
}) => {
  return (
    <header className="h-14 border-b border-zinc-800/90 bg-zinc-950/80 backdrop-blur-md px-4 flex items-center justify-between shrink-0 select-none">
      {/* Brand & Security Seal */}
      <div className="flex items-center space-x-3">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-600 via-indigo-500 to-amber-500 flex items-center justify-center shadow-lg shadow-indigo-500/20 border border-indigo-400/30">
            <ShieldCheck className="w-4 h-4 text-white" />
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <span className="font-bold text-sm tracking-tight text-zinc-100">
                Shambhavi Sovereign RAG Vault
              </span>
              <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-mono font-medium">
                v2.6 Enterprise
              </span>
            </div>
            <div className="text-[10px] text-zinc-500 font-mono">
              Pure Native Rust • DirectML • Hardware-Locked
            </div>
          </div>
        </div>
      </div>

      {/* Scope Selector Bar */}
      <div className="flex items-center space-x-2">
        <ScopeSelector
          currentScope={currentScope}
          drives={drives}
          recentPaths={recentPaths}
          onSelectScope={onSelectScope}
          onBrowseCustom={onBrowseCustom}
        />

        {/* 1-Click Smart Clean & Rename Files Trigger */}
        {uncleanedCount > 0 && (
          <button
            type="button"
            onClick={onAutoClean}
            className="flex items-center gap-1.5 px-2.5 py-1.5 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 text-amber-400 hover:text-amber-300 rounded-lg text-xs font-medium transition shadow-sm"
            title="Auto-extract client & doc type from scans (e.g. SCAN_00492.pdf) into clean canonical filenames"
          >
            <Wand2 className="w-3.5 h-3.5" />
            <span>Smart Rename ({uncleanedCount})</span>
          </button>
        )}
      </div>

      {/* Vault Status Indicator */}
      <div className="flex items-center space-x-2">
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-zinc-900/90 border border-zinc-800 text-xs">
          <Cpu className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
          <span className="text-zinc-400">Vault:</span>
          <span className="font-semibold text-emerald-400 font-mono">
            {drmStatus?.isBound ? 'Bound (DirectML GPU)' : 'Verifying...'}
          </span>
        </div>
      </div>
    </header>
  );
};
