import MarketCard from "@/components/MarketCard";
import { Market } from "@/lib/api";

// Static demo data for initial UI (before backend is connected)
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
  {
    id: "5",
    slug: "openai-ipo-2026",
    title: "Will OpenAI conduct an IPO before end of 2026?",
    category_name: "Technology",
    category_slug: "technology",
    status: "active",
    consensus_probability: 0.45,
    source_count: 3,
    updated_at: new Date().toISOString(),
  },
  {
    id: "6",
    slug: "eu-ai-act-enforcement",
    title: "Will the EU AI Act see its first major enforcement action in 2026?",
    category_name: "Technology",
    category_slug: "technology",
    status: "active",
    consensus_probability: 0.52,
    source_count: 2,
    updated_at: new Date().toISOString(),
  },
];

const categories = [
  { slug: "all", name: "All Markets", icon: "\ud83d\udcca" },
  { slug: "politics", name: "Politics", icon: "\ud83c\udfdb\ufe0f" },
  { slug: "economics", name: "Economics", icon: "\ud83d\udcc8" },
  { slug: "technology", name: "Technology", icon: "\ud83d\udcbb" },
  { slug: "crypto", name: "Crypto", icon: "\ud83e\ude99" },
  { slug: "climate", name: "Climate", icon: "\ud83c\udf0d" },
];

export default function DashboardPage() {
  // In production, this would use getMarkets() from the API
  const markets = demoMarkets;

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Hero Section */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-navy">Dashboard</h1>
        <p className="text-gray-500 mt-1">
          Aggregated prediction market intelligence, weighted by accuracy.
        </p>
      </div>

      {/* Stats Bar */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <StatCard label="Markets Tracked" value="1,247" />
        <StatCard label="Sources" value="5" />
        <StatCard label="Resolved Questions" value="3,891" />
        <StatCard label="Avg Consensus Accuracy" value="84.2%" highlight />
      </div>

      {/* Category Filter */}
      <div className="flex items-center gap-2 mb-6 overflow-x-auto pb-2">
        {categories.map((cat) => (
          <button
            key={cat.slug}
            className={`px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors ${
              cat.slug === "all"
                ? "bg-navy text-white"
                : "bg-white text-gray-600 border border-gray-200 hover:border-brand hover:text-brand"
            }`}
          >
            <span className="mr-1.5">{cat.icon}</span>
            {cat.name}
          </button>
        ))}
      </div>

      {/* Trending / Hot Markets */}
      <div className="mb-8">
        <h2 className="text-lg font-semibold text-navy mb-4">Trending Markets</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {markets.slice(0, 3).map((market) => (
            <MarketCard key={market.id} market={market} />
          ))}
        </div>
      </div>

      {/* All Markets */}
      <div>
        <h2 className="text-lg font-semibold text-navy mb-4">All Markets</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {markets.map((market) => (
            <MarketCard key={market.id} market={market} />
          ))}
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value, highlight }: { label: string; value: string; highlight?: boolean }) {
  return (
    <div className="bg-white rounded-xl border border-gray-100 p-4">
      <div className="text-xs font-medium text-gray-400 uppercase tracking-wider">{label}</div>
      <div className={`text-2xl font-mono font-bold mt-1 ${highlight ? "text-up" : "text-navy"}`}>
        {value}
      </div>
    </div>
  );
}
