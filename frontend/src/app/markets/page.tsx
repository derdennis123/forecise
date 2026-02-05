import MarketCard from "@/components/MarketCard";
import { Market } from "@/lib/api";

// This would use getMarkets() in production
const demoMarkets: Market[] = [
  {
    id: "1",
    slug: "trump-eu-tariffs-2026",
    title: "Will Trump impose 25%+ tariffs on EU goods before July 2026?",
    category_name: "Politics",
    category_slug: "politics",
    status: "active",
    consensus_probability: 0.71,
    source_count: 4,
    updated_at: new Date().toISOString(),
  },
  {
    id: "2",
    slug: "ecb-rate-cut-march-2026",
    title: "Will the ECB cut interest rates in March 2026?",
    category_name: "Economics",
    category_slug: "economics",
    status: "active",
    consensus_probability: 0.684,
    source_count: 3,
    updated_at: new Date().toISOString(),
  },
  {
    id: "3",
    slug: "bitcoin-100k-june-2026",
    title: "Will Bitcoin exceed $100,000 by June 2026?",
    category_name: "Crypto",
    category_slug: "crypto",
    status: "active",
    consensus_probability: 0.38,
    source_count: 5,
    updated_at: new Date().toISOString(),
  },
  {
    id: "4",
    slug: "us-recession-2026",
    title: "Will the US enter a recession in 2026?",
    category_name: "Economics",
    category_slug: "economics",
    status: "active",
    consensus_probability: 0.23,
    source_count: 4,
    updated_at: new Date().toISOString(),
  },
];

export default function MarketsPage() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-navy">Markets</h1>
          <p className="text-gray-500 mt-1">Browse all tracked prediction markets.</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {demoMarkets.map((market) => (
          <MarketCard key={market.id} market={market} />
        ))}
      </div>
    </div>
  );
}
