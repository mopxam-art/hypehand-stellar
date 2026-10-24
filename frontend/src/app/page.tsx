import Link from "next/link";

export default function HomePage() {
  return (
    <div className="flex-1 flex flex-col justify-center">
      {/* Hero Section */}
      <section className="relative overflow-hidden py-20 sm:py-32">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(14,165,233,0.15),rgba(255,255,255,0))]" />
        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-cyan-950/60 border border-cyan-800/50 text-cyan-400 text-xs font-medium mb-8">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
            <span>Powered by Stellar Soroban Contracts</span>
          </div>

          <h1 className="text-4xl sm:text-6xl lg:text-7xl font-extrabold tracking-tight text-white max-w-4xl mx-auto leading-tight">
            Next-Gen Gifting & Tipping for{" "}
            <span className="bg-gradient-to-r from-cyan-400 via-sky-300 to-indigo-400 bg-clip-text text-transparent">
              Web3 Creators
            </span>
          </h1>

          <p className="mt-6 text-lg sm:text-xl text-slate-400 max-w-2xl mx-auto leading-relaxed">
            Send instant gifts, tips, and micro-donations directly to creators on
            Stellar Soroban with sub-second finality and near-zero fees.
          </p>

          <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/gift"
              className="w-full sm:w-auto px-8 py-3.5 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold shadow-lg shadow-cyan-900/30 hover:from-cyan-400 hover:to-blue-500 transition-all text-base"
            >
              Send a Gift Now →
            </Link>
            <Link
              href="/dashboard"
              className="w-full sm:w-auto px-8 py-3.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-200 font-medium hover:bg-slate-800 hover:text-white transition-all text-base"
            >
              Creator Dashboard
            </Link>
          </div>
        </div>
      </section>

      {/* Feature Highlights */}
      <section className="py-16 bg-slate-900/50 border-y border-slate-900">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="p-6 rounded-2xl bg-slate-950/70 border border-slate-800/80 hover:border-cyan-800/60 transition-colors">
              <div className="w-12 h-12 rounded-xl bg-cyan-950/80 border border-cyan-800/60 flex items-center justify-center text-xl text-cyan-400 mb-4">
                ⚡
              </div>
              <h3 className="text-lg font-semibold text-white">Sub-Second Finality</h3>
              <p className="mt-2 text-sm text-slate-400">
                Gifts settle on Stellar Soroban in under 5 seconds with cryptographic receipts.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-slate-950/70 border border-slate-800/80 hover:border-cyan-800/60 transition-colors">
              <div className="w-12 h-12 rounded-xl bg-indigo-950/80 border border-indigo-800/60 flex items-center justify-center text-xl text-indigo-400 mb-4">
                💎
              </div>
              <h3 className="text-lg font-semibold text-white">Micro-Cent Fees</h3>
              <p className="mt-2 text-sm text-slate-400">
                Average transaction costs under $0.0001, allowing creators to keep 99.99% of proceeds.
              </p>
            </div>

            <div className="p-6 rounded-2xl bg-slate-950/70 border border-slate-800/80 hover:border-cyan-800/60 transition-colors">
              <div className="w-12 h-12 rounded-xl bg-sky-950/80 border border-sky-800/60 flex items-center justify-center text-xl text-sky-400 mb-4">
                🛡️
              </div>
              <h3 className="text-lg font-semibold text-white">Smart Contract Escrow</h3>
              <p className="mt-2 text-sm text-slate-400">
                Audited Soroban contracts manage accounts, milestones, and automated payouts securely.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Metrics Row */}
      <section className="py-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
            <div className="p-4">
              <div className="text-3xl sm:text-4xl font-extrabold text-cyan-400">50K+</div>
              <div className="text-xs sm:text-sm text-slate-400 mt-1">Gifts Delivered</div>
            </div>
            <div className="p-4">
              <div className="text-3xl sm:text-4xl font-extrabold text-white">&lt; 5s</div>
              <div className="text-xs sm:text-sm text-slate-400 mt-1">Average Finality</div>
            </div>
            <div className="p-4">
              <div className="text-3xl sm:text-4xl font-extrabold text-cyan-400">&lt; $0.0001</div>
              <div className="text-xs sm:text-sm text-slate-400 mt-1">Network Fee</div>
            </div>
            <div className="p-4">
              <div className="text-3xl sm:text-4xl font-extrabold text-white">100%</div>
              <div className="text-xs sm:text-sm text-slate-400 mt-1">Onchain Verifiable</div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
