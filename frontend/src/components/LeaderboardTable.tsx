import { AccuracyEntry } from "@/lib/api";

interface Props {
  entries: AccuracyEntry[];
}

export default function LeaderboardTable({ entries }: Props) {
  return (
    <div className="bg-white rounded-xl border border-gray-100 overflow-hidden">
      <table className="w-full">
        <thead>
          <tr className="border-b border-gray-100">
            <th className="text-left text-xs font-medium text-gray-400 uppercase tracking-wider px-6 py-3">Rank</th>
            <th className="text-left text-xs font-medium text-gray-400 uppercase tracking-wider px-6 py-3">Source</th>
            <th className="text-right text-xs font-medium text-gray-400 uppercase tracking-wider px-6 py-3">Accuracy</th>
            <th className="text-right text-xs font-medium text-gray-400 uppercase tracking-wider px-6 py-3">Brier Score</th>
            <th className="text-right text-xs font-medium text-gray-400 uppercase tracking-wider px-6 py-3">Resolved</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-50">
          {entries.map((entry) => (
            <tr key={entry.source_slug} className="hover:bg-gray-50 transition-colors">
              <td className="px-6 py-4">
                <span className={`inline-flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold ${
                  entry.rank === 1 ? "bg-amber-100 text-amber-700" :
                  entry.rank === 2 ? "bg-gray-100 text-gray-600" :
                  entry.rank === 3 ? "bg-orange-100 text-orange-700" :
                  "bg-gray-50 text-gray-500"
                }`}>
                  {entry.rank}
                </span>
              </td>
              <td className="px-6 py-4 font-medium text-navy">{entry.source_name}</td>
              <td className="px-6 py-4 text-right">
                <span className="font-mono font-semibold text-up">
                  {entry.accuracy_pct ? `${Number(entry.accuracy_pct).toFixed(1)}%` : "\u2014"}
                </span>
              </td>
              <td className="px-6 py-4 text-right font-mono text-sm text-gray-500">
                {entry.brier_score ? Number(entry.brier_score).toFixed(4) : "\u2014"}
              </td>
              <td className="px-6 py-4 text-right text-sm text-gray-500">
                {entry.total_resolved}
              </td>
            </tr>
          ))}
          {entries.length === 0 && (
            <tr>
              <td colSpan={5} className="px-6 py-12 text-center text-gray-400">
                No accuracy data available yet. Data is being collected.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
