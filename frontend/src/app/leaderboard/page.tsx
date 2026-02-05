"use client";

import { useState, useEffect, useCallback } from "react";
import LeaderboardTable from "@/components/LeaderboardTable";
import { AccuracyEntry } from "@/lib/api";

const categories = [
  { slug: "all", name: "All Categories" },
  { slug: "politics", name: "Politics" },
  { slug: "economics", name: "Economics" },
  { slug: "technology", name: "Technology" },
  { slug: "crypto", name: "Crypto" },
  { slug: "climate", name: "Climate" },
  { slug: "sports", name: "Sports" },
  { slug: "science", name: "Science" },
];

export default function LeaderboardPage() {
  const [entries, setEntries] = useState<AccuracyEntry[]>([]);
  const [category, setCategory] = useState("all");
  const [loading, setLoading] = useState(true);

  const fetchLeaderboard = useCallback(async () => {
    setLoading(true);
    try {
      const qs = category !== "all" ? `?category=${category}` : "";
      const res = await fetch(`/api/accuracy/leaderboard${qs}`);
      if (res.ok) {
        const data = await res.json();
        setEntries(data.data ?? []);
      }
    } catch {
      // API not available
    } finally {
      setLoading(false);
    }
  }, [category]);

  useEffect(() => {
    fetchLeaderboard();
  }, [fetchLeaderboard]);

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-navy">Accuracy Leaderboard</h1>
        <p className="text-gray-500 mt-1">
          Who has been historically most accurate? Tracked across all resolved questions.
        </p>
      </div>

      <div className="flex items-center gap-2 mb-6 overflow-x-auto pb-2">
        {categories.map((cat) => (
          <button
            key={cat.slug}
            onClick={() => setCategory(cat.slug)}
            className={`px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors ${
              cat.slug === category
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

      {loading ? (
        <div className="flex justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-brand"></div>
        </div>
      ) : (
        <LeaderboardTable entries={entries} />
      )}
    </div>
  );
}
