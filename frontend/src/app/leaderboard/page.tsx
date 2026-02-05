import LeaderboardTable from "@/components/LeaderboardTable";
import { AccuracyEntry } from "@/lib/api";

// Demo data - would use getLeaderboard() in production
const demoEntries: AccuracyEntry[] = [
  { rank: 1, source_name: "Polymarket", source_slug: "polymarket", accuracy_pct: 89.2, brier_score: 0.1080, total_resolved: 134 },
  { rank: 2, source_name: "Metaculus", source_slug: "metaculus", accuracy_pct: 84.7, brier_score: 0.1530, total_resolved: 89 },
  { rank: 3, source_name: "Kalshi", source_slug: "kalshi", accuracy_pct: 81.3, brier_score: 0.1870, total_resolved: 67 },
  { rank: 4, source_name: "Manifold Markets", source_slug: "manifold", accuracy_pct: 76.5, brier_score: 0.2350, total_resolved: 203 },
  { rank: 5, source_name: "PredictIt", source_slug: "predictit", accuracy_pct: 72.1, brier_score: 0.2790, total_resolved: 48 },
];

const categories = [
  { slug: "all", name: "All Categories" },
  { slug: "politics", name: "Politics" },
  { slug: "economics", name: "Economics" },
  { slug: "technology", name: "Technology" },
  { slug: "crypto", name: "Crypto" },
];

export default function LeaderboardPage() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-navy">Accuracy Leaderboard</h1>
        <p className="text-gray-500 mt-1">
          Who has been historically most accurate? Tracked across all resolved questions.
        </p>
      </div>

      {/* Category tabs */}
      <div className="flex items-center gap-2 mb-6">
        {categories.map((cat) => (
          <button
            key={cat.slug}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              cat.slug === "all"
                ? "bg-navy text-white"
                : "bg-white text-gray-600 border border-gray-200 hover:border-brand hover:text-brand"
            }`}
          >
            {cat.name}
          </button>
        ))}
      </div>

      {/* Info banner */}
      <div className="bg-blue-50 border border-blue-100 rounded-xl p-4 mb-6">
        <p className="text-sm text-blue-800">
          <span className="font-semibold">How it works:</span> We track every resolved prediction from each source and calculate their Brier Score — a mathematical measure of forecast accuracy. Sources need at least 30 resolved questions to be ranked.
        </p>
      </div>

      <LeaderboardTable entries={demoEntries} />
    </div>
  );
}
