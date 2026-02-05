interface Mover {
  market_id: string;
  title: string;
  source_name: string;
  probability_before: number;
  probability_after: number;
  change_pct: number;
  direction?: string;
}

interface VolumeMarket {
  market_id: string;
  title: string;
  total_volume: number | null;
  source_count: number;
  probability?: number;
}

interface SourceCount {
  source_name: string;
  market_count: number;
}

interface Briefing {
  briefing_date: string;
  total_markets_tracked: number;
  new_markets_24h: number;
  top_movers: Mover[];
  high_volume_markets: VolumeMarket[];
  source_counts?: SourceCount[];
  source_agreement?: Array<{
    market_id: string;
    title: string;
    min_probability: number;
    max_probability: number;
    spread: number;
    source_count: number;
  }>;
  key_stats?: {
    total_active_markets: number;
    total_sources_active: number;
    avg_source_count_per_market: number;
    total_movements_24h: number;
    markets_with_consensus: number;
  };
  summary: string | null;
}

export default async function BriefingPage() {
  let briefing: Briefing | null = null;

  try {
    const res = await fetch("http://localhost:3001/api/briefing/latest", {
      next: { revalidate: 300 },
    });
    if (res.ok) {
      const data = await res.json();
      briefing = data.data;
    }
  } catch {
    // API not available
  }

  const today = new Date().toLocaleDateString("en-US", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 bg-gradient-to-br from-amber-400 to-orange-500 rounded-xl flex items-center justify-center">
            <span className="text-white text-lg">&#9788;</span>
          </div>
          <div>
            <h1 className="text-2xl font-bold text-navy">Morning Briefing</h1>
            <p className="text-sm text-gray-500">{today}</p>
          </div>
        </div>
      </div>

      {briefing ? (
        <div className="space-y-6">
          {/* Summary */}
          {briefing.summary && (
            <div className="bg-gradient-to-r from-navy to-blue-800 rounded-xl p-6 text-white">
              <h2 className="text-sm font-medium text-blue-200 uppercase tracking-wider mb-2">Summary</h2>
              <p className="text-lg leading-relaxed">{briefing.summary}</p>
            </div>
          )}

          {/* Stats Row */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard label="Markets Tracked" value={briefing.total_markets_tracked.toLocaleString()} />
            <StatCard label="New Today" value={`+${briefing.new_markets_24h}`} highlight={briefing.new_markets_24h > 0} />
            <StatCard
              label="Movements (24h)"
              value={(briefing.key_stats?.total_movements_24h ?? briefing.top_movers.length).toString()}
              highlight={true}
            />
            <StatCard
              label="Sources Active"
              value={(briefing.key_stats?.total_sources_active ?? briefing.source_counts?.length ?? 3).toString()}
            />
          </div>

          {/* Source breakdown */}
          {briefing.source_counts && briefing.source_counts.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-100 p-6">
              <h2 className="text-lg font-semibold text-navy mb-4">Markets by Source</h2>
              <div className="space-y-3">
                {briefing.source_counts.map((sc) => {
                  const maxCount = Math.max(...briefing!.source_counts!.map((s) => s.market_count));
                  const pct = maxCount > 0 ? (sc.market_count / maxCount) * 100 : 0;
                  return (
                    <div key={sc.source_name}>
                      <div className="flex justify-between text-sm mb-1">
                        <span className="font-medium text-gray-700">{sc.source_name}</span>
                        <span className="text-gray-500">{sc.market_count.toLocaleString()} markets</span>
                      </div>
                      <div className="w-full bg-gray-100 rounded-full h-2">
                        <div
                          className="bg-brand rounded-full h-2 transition-all"
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Top Movers */}
          {briefing.top_movers.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-100 p-6">
              <h2 className="text-lg font-semibold text-navy mb-4">Top Movers (24h)</h2>
              <div className="space-y-3">
                {briefing.top_movers.slice(0, 10).map((mover, i) => {
                  const before = typeof mover.probability_before === "string" ? parseFloat(mover.probability_before) : mover.probability_before;
                  const after = typeof mover.probability_after === "string" ? parseFloat(mover.probability_after) : mover.probability_after;
                  const change = typeof mover.change_pct === "string" ? parseFloat(mover.change_pct) : mover.change_pct;
                  const isUp = after > before;
                  return (
                    <a
                      key={i}
                      href={`/markets/${mover.market_id}`}
                      className="flex items-start gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-3"
                    >
                      <div className={`mt-0.5 w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold ${
                        isUp ? "bg-emerald-50 text-emerald-600" : "bg-red-50 text-red-600"
                      }`}>
                        {isUp ? "\u2191" : "\u2193"}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 truncate">{mover.title}</p>
                        <p className="text-xs text-gray-500">{mover.source_name}</p>
                      </div>
                      <div className="text-right shrink-0">
                        <span className={`text-sm font-mono font-bold ${isUp ? "text-emerald-600" : "text-red-600"}`}>
                          {isUp ? "+" : ""}{(change * 100).toFixed(1)}%
                        </span>
                        <p className="text-xs text-gray-400">
                          {(before * 100).toFixed(0)}% &rarr; {(after * 100).toFixed(0)}%
                        </p>
                      </div>
                    </a>
                  );
                })}
              </div>
            </div>
          )}

          {/* High Volume Markets */}
          {briefing.high_volume_markets.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-100 p-6">
              <h2 className="text-lg font-semibold text-navy mb-4">Highest Volume Markets</h2>
              <div className="space-y-3">
                {briefing.high_volume_markets.slice(0, 8).map((market, i) => {
                  const vol = typeof market.total_volume === "string" ? parseFloat(market.total_volume) : (market.total_volume ?? 0);
                  return (
                    <a
                      key={i}
                      href={`/markets/${market.market_id}`}
                      className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-3"
                    >
                      <span className="text-sm font-mono text-gray-400 w-6">{i + 1}.</span>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900 truncate">{market.title}</p>
                        <p className="text-xs text-gray-500">{market.source_count} source{market.source_count !== 1 ? "s" : ""}</p>
                      </div>
                      <div className="text-right shrink-0">
                        <span className="text-sm font-mono font-semibold text-navy">
                          ${vol >= 1000000 ? `${(vol / 1000000).toFixed(1)}M` : vol >= 1000 ? `${(vol / 1000).toFixed(0)}K` : vol.toFixed(0)}
                        </span>
                      </div>
                    </a>
                  );
                })}
              </div>
            </div>
          )}

          {/* Source Disagreements */}
          {briefing.source_agreement && briefing.source_agreement.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-100 p-6">
              <h2 className="text-lg font-semibold text-navy mb-1">Where Sources Disagree</h2>
              <p className="text-sm text-gray-500 mb-4">Markets with the largest spread between source forecasts</p>
              <div className="space-y-3">
                {briefing.source_agreement.slice(0, 5).map((item, i) => (
                  <a
                    key={i}
                    href={`/markets/${item.market_id}`}
                    className="block p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-3"
                  >
                    <p className="text-sm font-medium text-gray-900 truncate mb-2">{item.title}</p>
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-mono text-gray-500">{(item.min_probability * 100).toFixed(0)}%</span>
                      <div className="flex-1 bg-gray-100 rounded-full h-2 relative">
                        <div
                          className="absolute bg-amber-400 rounded-full h-2"
                          style={{
                            left: `${item.min_probability * 100}%`,
                            width: `${item.spread * 100}%`,
                          }}
                        />
                      </div>
                      <span className="text-xs font-mono text-gray-500">{(item.max_probability * 100).toFixed(0)}%</span>
                      <span className="text-xs font-medium text-amber-600 ml-1">{item.source_count} sources</span>
                    </div>
                  </a>
                ))}
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="bg-white rounded-xl border border-gray-100 p-12 text-center">
          <div className="text-4xl mb-4">&#9788;</div>
          <h2 className="text-lg font-semibold text-gray-700 mb-2">No Briefing Available Yet</h2>
          <p className="text-gray-500 max-w-md mx-auto">
            The morning briefing is generated automatically from prediction market data.
            Check back soon for your daily intelligence summary.
          </p>
        </div>
      )}
    </div>
  );
}

function StatCard({ label, value, highlight }: { label: string; value: string; highlight?: boolean }) {
  return (
    <div className="bg-white rounded-xl border border-gray-100 p-4">
      <div className="text-xs font-medium text-gray-400 uppercase tracking-wider">{label}</div>
      <div className={`text-2xl font-mono font-bold mt-1 ${highlight ? "text-brand" : "text-navy"}`}>
        {value}
      </div>
    </div>
  );
}
