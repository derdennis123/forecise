import LeaderboardTable from "@/components/LeaderboardTable";
import { AccuracyEntry } from "@/lib/api";

const categories = [
  { slug: "all", name: "All Categories" },
  { slug: "politics", name: "Politics" },
  { slug: "economics", name: "Economics" },
  { slug: "technology", name: "Technology" },
  { slug: "crypto", name: "Crypto" },
];

export default async function LeaderboardPage() {
  let entries: AccuracyEntry[] = [];

  try {
    const res = await fetch("http://localhost:3001/api/accuracy/leaderboard", {
      next: { revalidate: 60 },
    });
    if (res.ok) {
      const data = await res.json();
      entries = data.data ?? [];
    }
  } catch {}

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-navy">Accuracy Leaderboard</h1>
        <p className="text-gray-500 mt-1">
          Who has been historically most accurate? Tracked across all resolved questions.
        </p>
      </div>

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

      <div className="bg-blue-50 border border-blue-100 rounded-xl p-4 mb-6">
        <p className="text-sm text-blue-800">
          <span className="font-semibold">How it works:</span> We track every resolved prediction from each source and calculate their Brier Score — a mathematical measure of forecast accuracy. Sources need at least 30 resolved questions to be ranked.
        </p>
      </div>

      <LeaderboardTable entries={entries} />
    </div>
  );
}
