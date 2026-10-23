export type ScopeType = 'all_drives' | 'drive' | 'custom_folder';

export interface DriveInfo {
  letter: string;
  name: string;
  totalSpaceGb?: number;
  freeSpaceGb?: number;
}

export interface ScopeConfig {
  type: ScopeType;
  path: string; // e.g. "ALL", "C:\", "D:\ragapp"
  label: string;
}

export interface Citation {
  chunkId: string;
  docId: string;
  filename: string;
  filePath: string;
  category: string;
  pageNumber?: number;
  score: number;
  content: string;
  smartAlias?: string;
  needsRenaming?: boolean;
  suggestedName?: string;
}

export interface AiSynthesis {
  query: string;
  summaryHindi: string;
  summaryEnglish: string;
  keyPoints: string[];
  citations: Citation[];
  confidence: number;
  model: string;
}

export interface HardwareDrmStatus {
  isBound: boolean;
  machineUuid: string;
  cpuId: string;
  motherboardSerial: string;
  tamperNeutralized: boolean;
  directMlActive: boolean;
  leaseExpiresInHours: number;
}

export interface SearchResponse {
  query: string;
  results: Citation[];
  synthesis?: AiSynthesis;
  latencyMs: number;
  tierUsed: 'tier0_fast_path' | 'tier1_hybrid_rag';
  scopeUsed: string;
  totalChunksSearched: number;
}
