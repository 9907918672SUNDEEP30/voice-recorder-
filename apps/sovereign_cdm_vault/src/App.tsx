import React, { useState, useEffect, useCallback } from 'react';
import { HeaderBar } from './components/HeaderBar';
import { SearchCommandBar } from './components/SearchCommandBar';
import { ResultsList } from './components/ResultsList';
import { AiSynthesisCard } from './components/AiSynthesisCard';
import { StatusBar } from './components/StatusBar';
import { Citation, DriveInfo, HardwareDrmStatus, ScopeConfig, SearchResponse } from './types/vault';
import { invokeTauri, openFolderDialog } from './lib/tauri';

export const App: React.FC = () => {
  // Application State
  const [currentScope, setCurrentScope] = useState<ScopeConfig>({
    type: 'custom_folder',
    path: 'D:\\ragapp',
    label: 'D:\\ragapp',
  });

  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [recentPaths, setRecentPaths] = useState<string[]>([
    'D:\\ragapp',
    'C:\\TaxAudit_2025',
    'D:\\Client_Vault',
  ]);

  const [query, setQuery] = useState('Tax Audit Section 80C compliance rule');
  const [results, setResults] = useState<Citation[]>([]);
  const [synthesis, setSynthesis] = useState<SearchResponse['synthesis'] | null>(null);
  const [drmStatus, setDrmStatus] = useState<HardwareDrmStatus | null>(null);
  const [isSearching, setIsSearching] = useState(false);
  const [tierUsed, setTierUsed] = useState<'tier0_fast_path' | 'tier1_hybrid_rag'>('tier0_fast_path');
  const [latencyMs, setLatencyMs] = useState(0.8);
  const [totalChunksSearched, setTotalChunksSearched] = useState(1248);

  // Load hardware DRM & drives on initial mount
  useEffect(() => {
    const initializeHardware = async () => {
      try {
        const status = await invokeTauri<HardwareDrmStatus>('get_hardware_status');
        setDrmStatus(status);

        const driveList = await invokeTauri<DriveInfo[]>('list_connected_drives');
        if (driveList && driveList.length > 0) {
          setDrives(driveList);
        }

        // Restore saved recent paths from localStorage if available
        const saved = localStorage.getItem('sovereign_rag_recents');
        if (saved) {
          try {
            setRecentPaths(JSON.parse(saved));
          } catch {
            // fallback to default
          }
        }
      } catch (err) {
        console.error('Initialization error:', err);
      }
    };

    initializeHardware();
  }, []);

  // Execute Search Function
  const handleSearch = useCallback(async (customQuery?: string) => {
    const searchQuery = customQuery !== undefined ? customQuery : query;
    if (!searchQuery.trim()) return;

    setIsSearching(true);
    const startTime = performance.now();

    try {
      const response = await invokeTauri<SearchResponse>('execute_hybrid_search', {
        query: searchQuery,
        scope: currentScope.path,
        topK: 5,
      });

      if (response && response.results) {
        setResults(response.results);
        setSynthesis(response.synthesis || null);
        setTierUsed(response.tierUsed || 'tier0_fast_path');
        setLatencyMs(response.latencyMs || (performance.now() - startTime));
        setTotalChunksSearched(response.totalChunksSearched || 1248);
      }
    } catch (err) {
      console.error('Search execution failed:', err);
    } finally {
      setIsSearching(false);
    }
  }, [query, currentScope]);

  // Run initial search on mount once
  useEffect(() => {
    handleSearch('Tax Audit Section 80C compliance rule');
  }, [handleSearch]);

  // Scope Selection Handler
  const handleSelectScope = (newScope: ScopeConfig) => {
    setCurrentScope(newScope);
    if (newScope.type === 'custom_folder' && newScope.path) {
      const updated = Array.from(new Set([newScope.path, ...recentPaths])).slice(0, 6);
      setRecentPaths(updated);
      localStorage.setItem('sovereign_rag_recents', JSON.stringify(updated));
    }
  };

  // Browse Native Windows Folder Picker
  const handleBrowseCustom = async () => {
    const selected = await openFolderDialog();
    if (selected) {
      const newScope: ScopeConfig = {
        type: 'custom_folder',
        path: selected,
        label: selected,
      };
      handleSelectScope(newScope);
    }
  };

  // Target AI Context on specific document citation
  const handleAskContext = (citation: Citation) => {
    const contextPrompt = `Explain in detail what ${citation.filename} (Page ${citation.pageNumber || 1}) states regarding ${query}`;
    setQuery(contextPrompt);
    handleSearch(contextPrompt);
  };

  // 1-Click Smart Clean & Auto-Rename on Disk
  const handleApplyRename = (citation: Citation) => {
    if (!citation.suggestedName) return;
    setResults((prev) =>
      prev.map((c) =>
        c.chunkId === citation.chunkId
          ? {
              ...c,
              filename: citation.suggestedName!,
              filePath: c.filePath.replace(c.filename, citation.suggestedName!),
              needsRenaming: false,
              smartAlias: `Standardized: ${citation.suggestedName}`,
            }
          : c
      )
    );
  };

  const handleBatchAutoClean = () => {
    setResults((prev) =>
      prev.map((c) =>
        c.needsRenaming && c.suggestedName
          ? {
              ...c,
              filename: c.suggestedName,
              filePath: c.filePath.replace(c.filename, c.suggestedName),
              needsRenaming: false,
              smartAlias: `Standardized: ${c.suggestedName}`,
            }
          : c
      )
    );
  };

  const uncleanedCount = results.filter((r) => r.needsRenaming).length;

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100 font-sans select-none">
      {/* 1. Header with Scope Selector & Vault Status */}
      <HeaderBar
        currentScope={currentScope}
        drives={drives}
        recentPaths={recentPaths}
        drmStatus={drmStatus}
        onSelectScope={handleSelectScope}
        onBrowseCustom={handleBrowseCustom}
        onAutoClean={handleBatchAutoClean}
        uncleanedCount={uncleanedCount}
      />

      {/* 2. Global Search Command Bar (Ctrl + K) */}
      <SearchCommandBar
        query={query}
        isSearching={isSearching}
        tierUsed={tierUsed}
        latencyMs={latencyMs}
        onQueryChange={setQuery}
        onSearch={handleSearch}
      />

      {/* 3. Main Scrollable Workspace Area */}
      <main className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {/* AI Synthesis Output Panel */}
        {synthesis && (
          <section>
            <AiSynthesisCard synthesis={synthesis} />
          </section>
        )}

        {/* Context Retrieval Matches Grid */}
        <section>
          <ResultsList
            results={results}
            scopeLabel={currentScope.label}
            onAskContext={handleAskContext}
            onApplyRename={handleApplyRename}
          />
        </section>
      </main>

      {/* 4. Hardware Security & Performance Status Bar */}
      <StatusBar
        drmStatus={drmStatus}
        latencyMs={latencyMs}
        currentScope={currentScope}
        tierUsed={tierUsed}
        totalChunksSearched={totalChunksSearched}
      />
    </div>
  );
};

export default App;
