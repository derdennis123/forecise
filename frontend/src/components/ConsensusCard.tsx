import { ConsensusInfo, SourceMarketSummary } from "@/lib/api";

interface Props {
  consensus: ConsensusInfo | null;
  sources: SourceMarketSummary[];
}

export default function ConsensusCard({ consensus, sources }: Props) {
  if (!consensus) {
    return (
      <div className="bg-white rounded-xl border border-gray-100 p-6">
        <h2 className="text-lg font-semibold text-navy mb-4">Forecise Consensus</h2>
        <p className="text-gray-400 text-sm">No consensus data available yet.</p>
      </div>
    );
  }

  const prob = Number(consensus.probability) * 100;
  const confidence = consensus.confidence ? Number(consensus.confidence) * 100 : null;
  const agreement = consensus.agreement ? Number(consensus.agreement) * 100 : null;

  return (
    <div className="bg-white rounded-xl border border-gray-100 p-6">
      <h2 className="text-lg font-semibold text-navy mb-4">Forecise Consensus</h2>

      {/* Main probability */}
      <div className="text-center py-4 mb-4 bg-gray-50 rounded-lg">
        <div className="text-5xl font-mono font-bold text-navy">
          {prob.toFixed(1)}%
        </div>
        <div className="flex items-center justify-center gap-4 mt-2 text-sm">
          {confidence !== null && (
            <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
              confidence > 70 ? "bg-green-100 text-up" :
              confidence > 40 ? "bg-amber-100 text-warning" :
              "bg-red-100 text-down"
            }`}>
              {confidence > 70 ? "HIGH" : confidence > 40 ? "MEDIUM" : "LOW"} confidence
            </span>
          )}
          {agreement !== null && (
            <span className="text-gray-500">
              {agreement.toFixed(0)}% agreement
            </span>
          )}
          <span className="text-gray-500">
            {consensus.source_count} sources
          </span>
        </div>
      </div>

      {/* Source breakdown */}
      <div className="space-y-3">
        <h3 className="text-sm font-medium text-gray-500">Source Breakdown</h3>
        {sources.map((source) => (
          <div key={source.source_slug} className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              <span className="font-medium text-navy">{source.source_name}</span>
              {source.accuracy_pct && (
                <span className="text-xs text-gray-400">
                  ({Number(source.accuracy_pct).toFixed(0)}% acc.)
                </span>
              )}
            </div>
            <div className="flex items-center gap-3">
              {source.probability !== null && (
                <span className="font-mono font-medium">
                  {(Number(source.probability) * 100).toFixed(1)}%
                </span>
              )}
              {source.external_url && (
                <a
                  href={source.external_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-brand hover:text-brand-dark text-xs"
                >
                  View
                </a>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
