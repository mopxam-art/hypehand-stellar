"use client";

import Link from "next/link";
import { useState } from "react";

interface GiftRecord {
  id: string;
  sender: string;
  amountXlm: number;
  message: string;
  timestamp: string;
  txHash: string;
}

const MOCK_GIFTS: GiftRecord[] = [
  {
    id: "gift-1",
    sender: "GAKF...4X9Z",
    amountXlm: 250,
    message: "Love the new tutorial! Keep building on Soroban 🚀",
    timestamp: "10 mins ago",
    txHash: "7b1f...d9a2",
  },
  {
    id: "gift-2",
    sender: "GC7M...2KLP",
    amountXlm: 100,
    message: "Great open-source work on the CLI binary!",
    timestamp: "2 hours ago",
    txHash: "a4c8...390e",
  },
  {
    id: "gift-3",
    sender: "GB3X...98WQ",
    amountXlm: 500,
    message: "HypeHand is the cleanest UI on Stellar so far.",
    timestamp: "1 day ago",
    txHash: "1e09...88fa",
  },
];

export default function CreatorDashboardPage() {
  const [gifts] = useState<GiftRecord[]>(MOCK_GIFTS);
  const totalBalance = 850;

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 w-full">
      {/* Header Profile Section */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between pb-8 border-b border-slate-800 gap-4">
        <div className="flex items-center space-x-4">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-tr from-cyan-500 to-indigo-600 flex items-center justify-center text-2xl font-bold text-white shadow-lg shadow-cyan-900/20">
            H
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h1 className="text-2xl font-bold text-white">Alex River</h1>
              <span className="px-2 py-0.5 rounded text-xs font-medium bg-emerald-950 text-emerald-400 border border-emerald-800/60">
                Verified Creator
              </span>
            </div>
            <p className="text-sm font-mono text-slate-400 mt-1">
              GDF8...3LP9 (Stellar Soroban Testnet)
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          <Link
            href="/gift"
            className="px-4 py-2 rounded-lg bg-slate-900 border border-slate-700 text-sm font-medium text-slate-200 hover:bg-slate-800 hover:text-white transition-colors"
          >
            Preview Gift Page
          </Link>
          <button
            onClick={() => alert("Withdraw funds initiated to connected wallet.")}
            className="px-4 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-600 text-sm font-semibold text-white shadow hover:from-cyan-400 hover:to-blue-500 transition-all"
          >
            Withdraw Funds
          </button>
        </div>
      </div>

      {/* Metrics Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 my-8">
        <div className="p-6 rounded-2xl bg-slate-900/80 border border-slate-800">
          <div className="text-sm font-medium text-slate-400">Total Available Balance</div>
          <div className="mt-2 text-3xl font-extrabold text-white">
            {totalBalance.toLocaleString()}{" "}
            <span className="text-lg font-semibold text-cyan-400">XLM</span>
          </div>
          <div className="mt-1 text-xs text-slate-500">
            ≈ ${(totalBalance * 0.12).toFixed(2)} USD (Stellar Spot)
          </div>
        </div>

        <div className="p-6 rounded-2xl bg-slate-900/80 border border-slate-800">
          <div className="text-sm font-medium text-slate-400">Total Gifts Received</div>
          <div className="mt-2 text-3xl font-extrabold text-white">48</div>
          <div className="mt-1 text-xs text-emerald-400">↑ 12% from last week</div>
        </div>

        <div className="p-6 rounded-2xl bg-slate-900/80 border border-slate-800">
          <div className="text-sm font-medium text-slate-400">Unique Supporters</div>
          <div className="mt-2 text-3xl font-extrabold text-white">35</div>
          <div className="mt-1 text-xs text-slate-500">Across 14 countries</div>
        </div>
      </div>

      {/* Gifts Activity Table */}
      <div className="mt-10">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Recent Received Gifts</h2>
          <span className="text-xs text-slate-400">Showing last 3 transactions</span>
        </div>

        <div className="overflow-x-auto rounded-2xl border border-slate-800 bg-slate-900/60">
          <table className="w-full text-left text-sm text-slate-300">
            <thead className="bg-slate-900 text-xs font-semibold uppercase text-slate-400 border-b border-slate-800">
              <tr>
                <th scope="col" className="px-6 py-4">Sender</th>
                <th scope="col" className="px-6 py-4">Amount</th>
                <th scope="col" className="px-6 py-4">Message</th>
                <th scope="col" className="px-6 py-4">Timestamp</th>
                <th scope="col" className="px-6 py-4 text-right">Onchain Tx</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800">
              {gifts.map((gift) => (
                <tr key={gift.id} className="hover:bg-slate-800/40 transition-colors">
                  <td className="px-6 py-4 font-mono text-cyan-400">{gift.sender}</td>
                  <td className="px-6 py-4 font-semibold text-white">
                    +{gift.amountXlm} XLM
                  </td>
                  <td className="px-6 py-4 text-slate-300 italic max-w-xs truncate">
                    &ldquo;{gift.message}&rdquo;
                  </td>
                  <td className="px-6 py-4 text-slate-400 text-xs">{gift.timestamp}</td>
                  <td className="px-6 py-4 text-right font-mono text-xs text-slate-500 hover:text-cyan-400 transition-colors">
                    {gift.txHash} ↗
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
