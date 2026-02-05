const API_BASE = "/api";

export interface Market {
  id: string;
  slug: string;
  title: string;
  category_name: string | null;
  category_slug: string | null;
  status: string;
  consensus_probability: number | null;
  source_count: number;
  updated_at: string;
}

export interface MarketDetail {
  id: string;
  slug: string;
  title: string;
  description: string | null;
  category_id: string | null;
  status: string;
  resolution_value: number | null;
  resolution_date: string | null;
  created_at: string;
  updated_at: string;
  category: Category | null;
  sources: SourceMarketSummary[];
  consensus: ConsensusInfo | null;
}

export interface Category {
  id: string;
  slug: string;
  name: string;
  description: string | null;
  icon: string | null;
}

export interface SourceMarketSummary {
  source_name: string;
  source_slug: string;
  probability: number | null;
  volume: number | null;
  accuracy_pct: number | null;
  external_url: string | null;
}

export interface ConsensusInfo {
  probability: number;
  confidence: number | null;
  source_count: number;
  agreement: number | null;
}

export interface OddsPoint {
  time: string;
  source_market_id: string;
  probability: number;
  volume: number | null;
}

export interface AccuracyEntry {
  rank: number;
  source_name: string;
  source_slug: string;
  accuracy_pct: number | null;
  brier_score: number | null;
  total_resolved: number;
}

export interface ApiResponse<T> {
  data: T;
  meta?: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

async function fetchApi<T>(path: string): Promise<ApiResponse<T>> {
  const res = await fetch(`${API_BASE}${path}`, { next: { revalidate: 60 } });
  if (!res.ok) {
    throw new Error(`API error: ${res.status}`);
  }
  return res.json();
}

export async function getMarkets(params?: {
  page?: number;
  per_page?: number;
  category?: string;
  status?: string;
  search?: string;
}): Promise<ApiResponse<Market[]>> {
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set("page", String(params.page));
  if (params?.per_page) searchParams.set("per_page", String(params.per_page));
  if (params?.category) searchParams.set("category", String(params.category));
  if (params?.status) searchParams.set("status", String(params.status));
  if (params?.search) searchParams.set("search", String(params.search));
  const qs = searchParams.toString();
  return fetchApi<Market[]>(`/markets${qs ? `?${qs}` : ""}`);
}

export async function getMarket(id: string): Promise<ApiResponse<MarketDetail>> {
  return fetchApi<MarketDetail>(`/markets/${id}`);
}

export async function getMarketOdds(id: string, timeframe?: string): Promise<ApiResponse<OddsPoint[]>> {
  const qs = timeframe ? `?timeframe=${timeframe}` : "";
  return fetchApi<OddsPoint[]>(`/markets/${id}/odds${qs}`);
}

export async function getLeaderboard(category?: string): Promise<ApiResponse<AccuracyEntry[]>> {
  const qs = category ? `?category=${category}` : "";
  return fetchApi<AccuracyEntry[]>(`/accuracy/leaderboard${qs}`);
}

export async function getConsensus(marketId: string): Promise<ApiResponse<ConsensusInfo>> {
  return fetchApi<ConsensusInfo>(`/consensus/${marketId}`);
}
