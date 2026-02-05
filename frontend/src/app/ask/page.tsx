"use client";

import { useState } from "react";

interface RelevantMarket {
  id: string;
  title: string;
  consensus_probability: number | null;
  source_count: number;
  relevance_score: number;
}

interface AskResponse {
  answer: string;
  relevant_markets: RelevantMarket[];
  disclaimer: string;
}

const suggestedQuestions = [
  "Will the ECB cut rates in March?",
  "What are the chances of a US recession?",
  "Will Bitcoin reach $100k?",
  "Trump tariffs on EU?",
  "Will OpenAI IPO this year?",
];

export default function AskPage() {
  const [question, setQuestion] = useState("");
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<AskResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleAsk(q?: string) {
    const query = q || question;
    if (!query.trim()) return;

    setLoading(true);
    setError(null);
    setResponse(null);

    try {
      const res = await fetch("/api/ask", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ question: query }),
      });

      if (!res.ok) {
        throw new Error("Failed to get answer");
      }

      const data = await res.json();
      setResponse(data.data);
    } catch {
      setError("Could not reach the Forecise API. Make sure the backend is running.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold text-navy">Ask Forecise</h1>
        <p className="text-gray-500 mt-2">
          Ask any question about the future. We'll find relevant prediction markets and synthesize an answer.
        </p>
      </div>

      {/* Search input */}
      <div className="relative mb-6">
        <input
          type="text"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleAsk()}
          placeholder="What do you want to know about the future?"
          className="w-full px-6 py-4 text-lg border-2 border-gray-200 rounded-2xl focus:outline-none focus:border-brand focus:ring-4 focus:ring-brand/10 transition-all"
        />
        <button
          onClick={() => handleAsk()}
          disabled={loading || !question.trim()}
          className="absolute right-3 top-3 px-5 py-2 bg-brand text-white rounded-xl font-medium hover:bg-brand-dark disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loading ? "Thinking..." : "Ask"}
        </button>
      </div>

      {/* Suggested questions */}
      {!response && !loading && (
        <div className="mb-8">
          <p className="text-sm text-gray-400 mb-3">Try asking:</p>
          <div className="flex flex-wrap gap-2">
            {suggestedQuestions.map((q) => (
              <button
                key={q}
                onClick={() => {
                  setQuestion(q);
                  handleAsk(q);
                }}
                className="px-4 py-2 text-sm bg-white border border-gray-200 rounded-lg text-gray-600 hover:border-brand hover:text-brand transition-colors"
              >
                {q}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="bg-white rounded-xl border border-gray-100 p-8 text-center">
          <div className="animate-pulse">
            <div className="h-4 bg-gray-200 rounded w-3/4 mx-auto mb-3" />
            <div className="h-4 bg-gray-200 rounded w-1/2 mx-auto mb-3" />
            <div className="h-4 bg-gray-200 rounded w-2/3 mx-auto" />
          </div>
          <p className="text-gray-400 text-sm mt-4">Searching prediction markets...</p>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-xl p-4">
          <p className="text-red-800 text-sm">{error}</p>
        </div>
      )}

      {/* Response */}
      {response && (
        <div className="space-y-6">
          {/* Answer */}
          <div className="bg-white rounded-xl border border-gray-100 p-6">
            <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider mb-3">Forecise Answer</h2>
            <div className="text-navy whitespace-pre-line leading-relaxed">
              {response.answer}
            </div>
          </div>

          {/* Relevant markets */}
          {response.relevant_markets.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-100 p-6">
              <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider mb-4">Related Markets</h2>
              <div className="space-y-3">
                {response.relevant_markets.map((market) => (
                  <a
                    key={market.id}
                    href={`/markets/${market.id}`}
                    className="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-navy truncate">{market.title}</p>
                      <p className="text-xs text-gray-400 mt-0.5">
                        {market.source_count} source{market.source_count !== 1 ? "s" : ""}
                      </p>
                    </div>
                    <div className="ml-4 text-right">
                      <span className="text-lg font-mono font-semibold text-navy">
                        {market.consensus_probability !== null
                          ? `${(Number(market.consensus_probability) * 100).toFixed(1)}%`
                          : "\u2014"}
                      </span>
                    </div>
                  </a>
                ))}
              </div>
            </div>
          )}

          {/* Disclaimer */}
          <div className="bg-gray-50 rounded-xl p-4">
            <p className="text-xs text-gray-400">{response.disclaimer}</p>
          </div>
        </div>
      )}
    </div>
  );
}
