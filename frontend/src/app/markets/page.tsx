import MarketCard from "@/components/MarketCard";
import { Market } from "@/lib/api";

export default async function MarketsPage() {
  let markets: Market[] = [];

  try {
    const res = await fetch("http://localhost:3001/api/markets?per_page=50", {
      next: { revalidate: 30 },
    });
    if (res.ok) {
      const data = await res.json();
      markets = data.data ?? [];
    }
  } catch {
    // API not available
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-navy">Markets</h1>
          <p className="text-gray-500 mt-1">
            {markets.length > 0
              ? `${markets.length} tracked prediction markets.`
              : "No markets loaded yet. Start the API server and workers."}
          </p>
        </div>
      </div>

      {markets.length === 0 && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-6 text-center">
          <p className="text-amber-800 font-medium">No market data yet</p>
          <p className="text-amber-600 text-sm mt-1">
            Start the backend with <code className="bg-amber-100 px-1 rounded">cargo run --bin forecise-workers</code> to begin ingesting data.
          </p>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {markets.map((market) => (
          <MarketCard key={market.id} market={market} />
        ))}
      </div>
    </div>
  );
}
