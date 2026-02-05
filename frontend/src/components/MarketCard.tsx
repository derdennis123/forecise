import { Market } from "@/lib/api";

function formatProbability(prob: number | null): string {
  if (prob === null) return "\u2014";
  return `${(Number(prob) * 100).toFixed(1)}%`;
}

function probColor(prob: number | null): string {
  if (prob === null) return "text-gray-400";
  const p = Number(prob);
  if (p >= 0.7) return "text-up";
  if (p <= 0.3) return "text-down";
  return "text-warning";
}

export default function MarketCard({ market }: { market: Market }) {
  return (
    <a
      href={`/markets/${market.id}`}
      className="block bg-white rounded-xl border border-gray-100 p-5 hover:border-brand/30 hover:shadow-md transition-all duration-200"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          {market.category_name && (
            <span className="inline-block px-2 py-0.5 text-xs font-medium bg-gray-100 text-gray-600 rounded-full mb-2">
              {market.category_name}
            </span>
          )}
          <h3 className="text-sm font-semibold text-navy leading-snug line-clamp-2">
            {market.title}
          </h3>
          <div className="flex items-center gap-3 mt-2 text-xs text-gray-400">
            <span>{market.source_count} source{market.source_count !== 1 ? "s" : ""}</span>
            <span>{market.status}</span>
          </div>
        </div>
        <div className="text-right shrink-0">
          <div className={`text-2xl font-mono font-semibold ${probColor(market.consensus_probability)}`}>
            {formatProbability(market.consensus_probability)}
          </div>
          <div className="text-xs text-gray-400 mt-1">consensus</div>
        </div>
      </div>
    </a>
  );
}
