"use client";

import { useEffect, useRef, useState } from "react";
import { createChart, ColorType, IChartApi, ISeriesApi, LineData, Time } from "lightweight-charts";

interface OddsPoint {
  time: string;
  source_market_id: string;
  probability: number;
}

interface Props {
  marketId: string;
}

const timeframes = [
  { label: "1D", value: "1d" },
  { label: "1W", value: "1w" },
  { label: "1M", value: "1m" },
  { label: "3M", value: "3m" },
  { label: "1Y", value: "1y" },
  { label: "ALL", value: "all" },
];

export default function OddsChart({ marketId }: Props) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Area"> | null>(null);
  const [activeTimeframe, setActiveTimeframe] = useState("1m");
  const [loading, setLoading] = useState(true);
  const [noData, setNoData] = useState(false);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "white" },
        textColor: "#64748b",
        fontFamily: "Inter, sans-serif",
      },
      grid: {
        vertLines: { color: "#f1f5f9" },
        horzLines: { color: "#f1f5f9" },
      },
      width: chartContainerRef.current.clientWidth,
      height: 300,
      rightPriceScale: {
        borderColor: "#e2e8f0",
        scaleMargins: { top: 0.1, bottom: 0.1 },
      },
      timeScale: {
        borderColor: "#e2e8f0",
        timeVisible: true,
      },
      crosshair: {
        vertLine: { color: "#3b82f6", width: 1, style: 2 },
        horzLine: { color: "#3b82f6", width: 1, style: 2 },
      },
    });

    const series = chart.addAreaSeries({
      lineColor: "#3b82f6",
      topColor: "rgba(59, 130, 246, 0.2)",
      bottomColor: "rgba(59, 130, 246, 0.02)",
      lineWidth: 2,
      priceFormat: {
        type: "custom",
        formatter: (price: number) => `${(price * 100).toFixed(1)}%`,
      },
    });

    chartRef.current = chart;
    seriesRef.current = series;

    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({ width: chartContainerRef.current.clientWidth });
      }
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };
  }, []);

  useEffect(() => {
    async function fetchOdds() {
      setLoading(true);
      setNoData(false);

      try {
        const res = await fetch(`/api/markets/${marketId}/odds?timeframe=${activeTimeframe}`);
        if (!res.ok) throw new Error("Failed to fetch");

        const json = await res.json();
        const odds: OddsPoint[] = json.data ?? [];

        if (odds.length === 0) {
          setNoData(true);
          setLoading(false);
          return;
        }

        // Group by source_market_id and aggregate (take latest per time bucket)
        const timeMap = new Map<string, number>();
        for (const point of odds) {
          const date = point.time.split("T")[0]; // Group by day
          const prob = Number(point.probability);
          if (!isNaN(prob)) {
            timeMap.set(date, prob);
          }
        }

        const chartData: LineData[] = Array.from(timeMap.entries())
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([time, value]) => ({
            time: time as Time,
            value,
          }));

        if (seriesRef.current && chartData.length > 0) {
          seriesRef.current.setData(chartData);
          chartRef.current?.timeScale().fitContent();
        } else {
          setNoData(true);
        }
      } catch {
        setNoData(true);
      } finally {
        setLoading(false);
      }
    }

    fetchOdds();
  }, [marketId, activeTimeframe]);

  return (
    <div className="bg-white rounded-xl border border-gray-100 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-navy">Price History</h2>
        <div className="flex gap-1">
          {timeframes.map((tf) => (
            <button
              key={tf.value}
              onClick={() => setActiveTimeframe(tf.value)}
              className={`px-3 py-1 text-xs font-medium rounded-md transition-colors ${
                activeTimeframe === tf.value
                  ? "bg-navy text-white"
                  : "text-gray-500 hover:bg-gray-100"
              }`}
            >
              {tf.label}
            </button>
          ))}
        </div>
      </div>

      <div className="relative">
        <div ref={chartContainerRef} className="w-full" />
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-white/80">
            <div className="animate-pulse text-gray-400 text-sm">Loading chart data...</div>
          </div>
        )}
        {noData && !loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-gray-50/80 rounded-lg">
            <p className="text-gray-400 text-sm">No historical data available yet. Data will appear after ingestion runs.</p>
          </div>
        )}
      </div>
    </div>
  );
}
