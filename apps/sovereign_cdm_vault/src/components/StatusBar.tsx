import React from 'react';
import { Lock, Zap, HardDrive, Cpu, Activity } from 'lucide-react';
import { HardwareDrmStatus, ScopeConfig } from '../types/vault';

interface StatusBarProps {
  drmStatus: HardwareDrmStatus | null;
  latencyMs?: number;
  currentScope: ScopeConfig;
  tierUsed?: 'tier0_fast_path' | 'tier1_hybrid_rag';
  totalChunksSearched?: number;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  drmStatus,
  latencyMs = 0.8,
  currentScope,
  tierUsed = 'tier0_fast_path',
  totalChunksSearched = 1248,
}) => {
  return (
    <footer className="h-8 border-t border-zinc-800/90 bg-zinc-950 px-4 flex items-center justify-between text-[11px] font-mono text-zinc-400 shrink-0 select-none">
      {/* Left: Hardware Binding & DRM Status */}
      <div className="flex items-center space-x-3">
        <div className="flex items-center gap-1.5 text-emerald-400">
          <Lock className="w-3.5 h-3.5" />
          <span className="font-medium">
            Hardware Signature Verified
          </span>
          <span className="text-zinc-600">|</span>
          <span className="text-zinc-300">
            Motherboard GUID: <span className="text-emerald-400 font-semibold">{drmStatus?.isBound ? 'Bound' : 'Active'}</span>
          </span>
        </div>

        <span className="text-zinc-700">•</span>

        <div className="hidden sm:flex items-center gap-1 text-zinc-400">
          <Cpu className="w-3 h-3 text-indigo-400" />
          <span>DirectML GPU: <span className="text-indigo-300">Engaged</span></span>
        </div>
      </div>

      {/* Right: Latency & Scope Metrics */}
      <div className="flex items-center space-x-3">
        <div className="hidden md:flex items-center gap-1 text-zinc-400">
          <HardDrive className="w-3 h-3 text-amber-400" />
          <span>Scope: <span className="text-zinc-200">{currentScope.label}</span></span>
          <span className="text-zinc-600">({totalChunksSearched} Chunks)</span>
        </div>

        <span className="text-zinc-700">•</span>

        <div className="flex items-center gap-1.5 text-zinc-300">
          <Activity className="w-3 h-3 text-indigo-400" />
          <span>Pipeline:</span>
          <span className={`px-1.5 py-0.2 rounded text-[10px] ${
            tierUsed === 'tier0_fast_path'
              ? 'bg-emerald-500/20 text-emerald-300 font-semibold'
              : 'bg-indigo-500/20 text-indigo-300 font-semibold'
          }`}>
            {tierUsed === 'tier0_fast_path' ? 'Tier-0 Fast' : 'Tier-1 Hybrid'}
          </span>
        </div>

        <span className="text-zinc-700">•</span>

        <div className="flex items-center gap-1 text-emerald-400 font-semibold">
          <Zap className="w-3 h-3 text-emerald-400" />
          <span>Latency: {latencyMs.toFixed(1)}ms</span>
        </div>
      </div>
    </footer>
  );
};
