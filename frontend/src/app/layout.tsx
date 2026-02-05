import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Forecise - Precise Forecasting Intelligence",
  description: "The single source of truth for prediction market intelligence. Aggregated odds, accuracy tracking, and AI-powered consensus forecasts.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet" />
      </head>
      <body className="antialiased">
        <div className="min-h-screen">
          <Navigation />
          <main>{children}</main>
        </div>
      </body>
    </html>
  );
}

function Navigation() {
  return (
    <nav className="border-b border-gray-200 bg-white sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex items-center gap-8">
            <a href="/" className="flex items-center gap-2">
              <div className="w-8 h-8 bg-brand rounded-lg flex items-center justify-center">
                <span className="text-white font-bold text-sm">F</span>
              </div>
              <span className="font-bold text-xl text-navy">Forecise</span>
            </a>
            <div className="hidden md:flex items-center gap-6">
              <a href="/" className="text-sm font-medium text-gray-600 hover:text-navy transition-colors">Dashboard</a>
              <a href="/markets" className="text-sm font-medium text-gray-600 hover:text-navy transition-colors">Markets</a>
              <a href="/leaderboard" className="text-sm font-medium text-gray-600 hover:text-navy transition-colors">Accuracy</a>
              <a href="/ask" className="text-sm font-medium text-gray-600 hover:text-navy transition-colors">Ask Forecise</a>
              <a href="/briefing" className="text-sm font-medium text-gray-600 hover:text-navy transition-colors">Briefing</a>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="relative">
              <input
                type="text"
                placeholder="Search markets..."
                className="w-64 pl-10 pr-4 py-2 text-sm border border-gray-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-brand/20 focus:border-brand"
              />
              <svg className="absolute left-3 top-2.5 h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
}
