import React, { useState } from 'react';
import { HardDrive, Folder, History, ChevronDown, Check, Globe } from 'lucide-react';
import { DriveInfo, ScopeConfig } from '../types/vault';

interface ScopeSelectorProps {
  currentScope: ScopeConfig;
  drives: DriveInfo[];
  recentPaths: string[];
  onSelectScope: (scope: ScopeConfig) => void;
  onBrowseCustom: () => void;
}

export const ScopeSelector: React.FC<ScopeSelectorProps> = ({
  currentScope,
  drives,
  recentPaths,
  onSelectScope,
  onBrowseCustom,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative inline-block text-left">
      <div className="flex items-center space-x-1.5 bg-zinc-900/90 border border-zinc-800 rounded-lg p-1 text-xs">
        <span className="text-zinc-400 font-medium px-2 flex items-center gap-1.5">
          <Folder className="w-3.5 h-3.5 text-amber-400" />
          <span>Scope:</span>
        </span>

        {/* Scope Pill Button */}
        <button
          type="button"
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 px-2.5 py-1 bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-200 rounded font-mono font-medium text-xs transition border border-zinc-700/50 shadow-inner"
        >
          <span className="truncate max-w-[160px]">{currentScope.label}</span>
          <ChevronDown className="w-3 h-3 text-zinc-400 ml-0.5" />
        </button>

        {/* Browse Button */}
        <button
          type="button"
          onClick={onBrowseCustom}
          className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white rounded text-xs font-medium transition border border-zinc-700/40"
        >
          Browse...
        </button>
      </div>

      {/* Dropdown Menu */}
      {isOpen && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute left-0 mt-2 w-72 bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl z-50 py-1.5 backdrop-blur-xl divide-y divide-zinc-800/70 text-xs">
            {/* Option 1: All Connected Drives */}
            <div className="p-1">
              <button
                type="button"
                onClick={() => {
                  onSelectScope({ type: 'all_drives', path: 'ALL', label: 'This PC (All Connected Drives)' });
                  setIsOpen(false);
                }}
                className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-left transition ${
                  currentScope.type === 'all_drives'
                    ? 'bg-amber-500/10 text-amber-400 font-semibold'
                    : 'text-zinc-300 hover:bg-zinc-800/70'
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <Globe className="w-4 h-4 text-amber-400" />
                  <div>
                    <div className="text-xs">This PC (All Connected Drives)</div>
                    <div className="text-[10px] text-zinc-500">Fast In-Memory Multi-Drive Scan</div>
                  </div>
                </div>
                {currentScope.type === 'all_drives' && <Check className="w-4 h-4 text-amber-400" />}
              </button>
            </div>

            {/* Option 2: Connected Physical Drives */}
            <div className="p-1">
              <div className="px-3 py-1 text-[10px] font-semibold text-zinc-500 uppercase tracking-wider">
                Logical Drives
              </div>
              {drives.map((drive) => (
                <button
                  key={drive.letter}
                  type="button"
                  onClick={() => {
                    onSelectScope({ type: 'drive', path: drive.letter, label: `${drive.letter} (${drive.name})` });
                    setIsOpen(false);
                  }}
                  className={`w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-left transition ${
                    currentScope.path === drive.letter
                      ? 'bg-indigo-500/10 text-indigo-400 font-semibold'
                      : 'text-zinc-300 hover:bg-zinc-800/70'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <HardDrive className="w-3.5 h-3.5 text-zinc-400" />
                    <span className="font-mono">{drive.letter}</span>
                    <span className="text-zinc-500 text-[11px] truncate max-w-[120px]">{drive.name}</span>
                  </div>
                  {currentScope.path === drive.letter && <Check className="w-3.5 h-3.5 text-indigo-400" />}
                </button>
              ))}
            </div>

            {/* Option 3: Recent Paths Quick-List */}
            {recentPaths.length > 0 && (
              <div className="p-1">
                <div className="px-3 py-1 text-[10px] font-semibold text-zinc-500 uppercase tracking-wider flex items-center gap-1">
                  <History className="w-3 h-3" />
                  <span>Recent Paths Quick-List</span>
                </div>
                {recentPaths.map((path) => (
                  <button
                    key={path}
                    type="button"
                    onClick={() => {
                      onSelectScope({ type: 'custom_folder', path, label: path });
                      setIsOpen(false);
                    }}
                    className={`w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-left transition font-mono ${
                      currentScope.path === path
                        ? 'bg-emerald-500/10 text-emerald-400 font-semibold'
                        : 'text-zinc-300 hover:bg-zinc-800/70'
                    }`}
                  >
                    <div className="flex items-center gap-2 truncate">
                      <Folder className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                      <span className="truncate text-xs">{path}</span>
                    </div>
                    {currentScope.path === path && <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0 ml-1" />}
                  </button>
                ))}
              </div>
            )}

            {/* Option 4: Custom Folder Picker Action */}
            <div className="p-1">
              <button
                type="button"
                onClick={() => {
                  setIsOpen(false);
                  onBrowseCustom();
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-zinc-300 hover:text-white hover:bg-zinc-800/80 rounded-lg text-left transition font-medium"
              >
                <Folder className="w-4 h-4 text-amber-400" />
                <span>Custom Folder... (Windows Picker)</span>
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
};
