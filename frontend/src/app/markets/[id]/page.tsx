import ConsensusCard from "@/components/ConsensusCard";
import OddsChart from "@/components/OddsChart";
import { SourceMarketSummary, ConsensusInfo } from "@/lib/api";

interface MarketDetailData {
  id: string;
  slug: string;
  title: string;
  description: string | null;
  status: string;
  category?: { name: string; slug: string } | null;
  sources: SourceMarketSummary[];
  consensus: ConsensusInfo | null;
}

async function getMarketDetail(id: string): Promise<MarketDetailData | null> {
  try {
    const res = await fetch(`http://localhost:3001/api/markets/${id}`, {
      next: { revalidate: 30 },
    });
    if (res.ok) {
      const data = await res.json();
      return data.data;
    }
  } catch {}
  return null;
}

export default async function MarketDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const market = await getMarketDetail(id);

  if (!market) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-6 text-center">
          <p className="text-amber-800 font-medium">Market not found</p>
          <p className="text-amber-600 text-sm mt-1">
            This market may not exist or the API server is not running.
          </p>
          <a href="/markets" className="text-brand hover:underline text-sm mt-2 inline-block">
            Back to Markets
          </a>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="mb-8">
        <div className="flex items-center gap-2 mb-2">
          {market.category && (
            <span className="px-2 py-0.5 text-xs font-medium bg-gray-100 text-gray-600 rounded-full">
              {market.category.name}
            </span>
          )}
          <span className={`px-2 py-0.5 text-xs font-medium rounded-full ${
            market.status === "active" ? "bg-green-100 text-up" : "bg-gray-100 text-gray-600"
          }`}>
            {market.status.charAt(0).toUpperCase() + market.status.slice(1)}
          </span>
        </div>
        <h1 className="text-2xl font-bold text-navy">{market.title}</h1>
        {market.description && (
          <p className="text-gray-500 mt-2">{market.description}</p>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <OddsChart marketId={id} />

          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <h2 className="text-lg font-semibold text-navy mb-4">Source Comparison</h2>
            {market.sources.length === 0 ? (
              <p className="text-gray-400 text-sm">No source data available.</p>
            ) : (
              <div className="space-y-4">
                {market.sources.map((source) => (
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
            )}
          </div>
        </div>

        <div className="space-y-6">
          <ConsensusCard consensus={market.consensus} sources={market.sources} />

          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <h2 className="text-lg font-semibold text-navy mb-4">Market Info</h2>
            <dl className="space-y-3 text-sm">
              <div className="flex justify-between">
                <dt className="text-gray-400">Status</dt>
                <dd className="font-medium text-navy">{market.status}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Sources</dt>
                <dd className="font-medium text-navy">{market.sources.length}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Market ID</dt>
                <dd className="font-mono text-xs text-gray-500">{market.id.slice(0, 8)}...</dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>
  );
}
