import ConsensusCard from "@/components/ConsensusCard";
import { SourceMarketSummary, ConsensusInfo } from "@/lib/api";

// Demo data - would use getMarket(id) in production
const demoSources: SourceMarketSummary[] = [
  { source_name: "Polymarket", source_slug: "polymarket", probability: 0.67, volume: 5200000, accuracy_pct: 89.2, external_url: "https://polymarket.com" },
  { source_name: "Kalshi", source_slug: "kalshi", probability: 0.61, volume: 1800000, accuracy_pct: 81.3, external_url: "https://kalshi.com" },
  { source_name: "Metaculus", source_slug: "metaculus", probability: 0.72, volume: null, accuracy_pct: 84.7, external_url: "https://metaculus.com" },
];

const demoConsensus: ConsensusInfo = {
  probability: 0.684,
  confidence: 0.78,
  source_count: 3,
  agreement: 0.85,
};

export default function MarketDetailPage() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center gap-2 mb-2">
          <span className="px-2 py-0.5 text-xs font-medium bg-gray-100 text-gray-600 rounded-full">Economics</span>
          <span className="px-2 py-0.5 text-xs font-medium bg-green-100 text-up rounded-full">Active</span>
        </div>
        <h1 className="text-2xl font-bold text-navy">
          Will the ECB cut interest rates in March 2026?
        </h1>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main content */}
        <div className="lg:col-span-2 space-y-6">
          {/* Chart placeholder */}
          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-navy">Price History</h2>
              <div className="flex gap-1">
                {["1D", "1W", "1M", "3M", "1Y", "ALL"].map((tf) => (
                  <button
                    key={tf}
                    className={`px-3 py-1 text-xs font-medium rounded-md transition-colors ${
                      tf === "1M"
                        ? "bg-navy text-white"
                        : "text-gray-500 hover:bg-gray-100"
                    }`}
                  >
                    {tf}
                  </button>
                ))}
              </div>
            </div>
            <div className="h-64 bg-gradient-to-br from-gray-50 to-gray-100 rounded-lg flex items-center justify-center text-gray-400 text-sm">
              Chart will render here with TradingView Lightweight Charts
            </div>
          </div>

          {/* Source comparison */}
          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <h2 className="text-lg font-semibold text-navy mb-4">Source Comparison</h2>
            <div className="space-y-4">
              {demoSources.map((source) => (
                <div key={source.source_slug} className="flex items-center gap-4">
                  <div className="w-28 text-sm font-medium text-navy">{source.source_name}</div>
                  <div className="flex-1">
                    <div className="h-8 bg-gray-100 rounded-full overflow-hidden relative">
                      <div
                        className="h-full bg-brand/20 rounded-full transition-all duration-500"
                        style={{ width: `${(Number(source.probability) || 0) * 100}%` }}
                      />
                      <span className="absolute inset-0 flex items-center justify-center text-xs font-mono font-semibold">
                        {source.probability !== null ? `${(Number(source.probability) * 100).toFixed(1)}%` : "\u2014"}
                      </span>
                    </div>
                  </div>
                  <div className="w-20 text-right">
                    {source.accuracy_pct && (
                      <span className="text-xs text-gray-400">
                        {Number(source.accuracy_pct).toFixed(0)}% acc.
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          <ConsensusCard consensus={demoConsensus} sources={demoSources} />

          {/* Market Info */}
          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <h2 className="text-lg font-semibold text-navy mb-4">Market Info</h2>
            <dl className="space-y-3 text-sm">
              <div className="flex justify-between">
                <dt className="text-gray-400">Status</dt>
                <dd className="font-medium text-navy">Active</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Sources</dt>
                <dd className="font-medium text-navy">3</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Total Volume</dt>
                <dd className="font-mono font-medium text-navy">$7.0M</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Created</dt>
                <dd className="font-medium text-navy">Jan 15, 2026</dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>
  );
}
