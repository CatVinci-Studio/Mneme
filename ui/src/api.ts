import { invoke } from "@tauri-apps/api/core";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(command, args); }
  catch (error) {
    if (error instanceof Error) throw error;
    throw new Error(typeof error === "string" ? error : JSON.stringify(error));
  }
}

export interface SourceMeta {
  id: string; kind: string; title: string; author?: string; url?: string;
  fetched_at: string; content_hash: string; word_count: number; summarized?: boolean;
  doi?: string; authors?: string[]; published?: string;
}
export interface SourceDetail { meta: SourceMeta; content: string; note: string | null }
export interface SearchHit { slug: string; title: string; snippet: string; score: number }
export interface WikifyReport { source_id: string; entities: { slug: string; ops: number; result: string }[] }
export interface EntitySummary { slug: string; title: string; kind: string; summary: string; updated: string }
export interface FactEntry { id: string; text: string; prov: { source_id: string; start: number; end: number } }
export interface EntityPage {
  slug: string; title: string; kind: string; aliases: string[]; sources: string[]; updated: string;
  summary: string; facts: FactEntry[]; history: (FactEntry & { superseded_at: string })[];
  contradictions: { note: string }[]; links: { target_slug: string; relation?: string }[];
}
export interface AppConfigView { provider: string; model: string | null; baseUrl: string | null; hasKey: boolean; providers: string[]; retriever: string; syncRemote: string | null; vault: string }
export interface LintReport { entities: number; sources: number; facts: number; orphans: string[]; emptyFacts: string[]; contradictions: { slug: string; count: number }[] }
export interface GraphData { nodes: { slug: string; title: string; kind: string }[]; edges: { source: string; target: string }[] }

export const api = {
  config: (): Promise<AppConfigView> => call("get_config"),
  setConfig: (input: { provider?: string; apiKey?: string; model?: string; baseUrl?: string; retriever?: string; syncRemote?: string }): Promise<AppConfigView> => call("set_config", { input }),
  graph: (): Promise<GraphData> => call("get_graph"),
  lint: (): Promise<LintReport> => call("lint_vault"),
  sync: (): Promise<{ ok: boolean; message: string }> => call("sync_vault"),
  listSources: (): Promise<SourceMeta[]> => call("list_sources"),
  addSource: (input: { url?: string; text?: string; title?: string }): Promise<{ source_id: string; meta: SourceMeta; report: WikifyReport }> => call("add_source", { input }),
  getSource: (id: string): Promise<SourceDetail> => call("get_source", { id }),
  wikify: (id: string): Promise<WikifyReport> => call("wikify_source", { id }),
  listEntities: (): Promise<EntitySummary[]> => call("list_entities"),
  getEntity: (slug: string): Promise<{ page: EntityPage; backlinks: string[] }> => call("get_entity", { slug }),
  query: (question: string): Promise<{ answer: string; sources: { slug: string; title: string }[] }> => call("research_query", { question }),
  search: (query: string): Promise<SearchHit[]> => call("search_entities", { query }),
};
