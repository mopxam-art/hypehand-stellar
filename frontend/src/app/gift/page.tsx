"use client";

import Link from "next/link";
import { useState } from "react";

export default function SendGiftPage() {
  const [recipient, setRecipient] = useState("GDF8KL39M0927XZWPAL1829374092182049182390");
  const [amount, setAmount] = useState<number>(50);
  const [message, setMessage] = useState("");
  const [isAnonymous, setIsAnonymous] = useState(false);
  const [status, setStatus] = useState<"idle" | "submitting" | "success">("idle");
  const [txReceipt, setTxReceipt] = useState<string | null>(null);

  const presets = [25, 50, 100, 250];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!recipient || amount <= 0) return;

    setStatus("submitting");

    // Simulate Soroban smart contract interaction and receipt generation
    setTimeout(() => {
      const mockHash = "7f" + Math.random().toString(16).substring(2, 10) + "..." + Math.random().toString(16).substring(2, 6);
      setTxReceipt(mockHash);
      setStatus("success");
    }, 1200);
  };

  return (
    <div className="max-w-2xl mx-auto px-4 sm:px-6 py-12 w-full">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-extrabold text-white">Send a Soroban Gift</h1>
        <p className="mt-2 text-sm text-slate-400">
          Support your favorite creator with instant onchain micro-tips on Stellar.
        </p>
      </div>

      {status === "success" ? (
        <div className="p-8 rounded-2xl bg-slate-900 border border-emerald-800/60 text-center shadow-xl">
          <div className="w-16 h-16 rounded-full bg-emerald-950/80 border border-emerald-500/40 text-emerald-400 text-3xl flex items-center justify-center mx-auto mb-4">
            ✓
          </div>
          <h2 className="text-xl font-bold text-white">Gift Successfully Sent!</h2>
          <p className="text-sm text-slate-400 mt-2">
            You sent <strong className="text-white">{amount} XLM</strong> to creator.
          </p>

          <div className="my-6 p-4 rounded-xl bg-slate-950 border border-slate-800 text-left text-xs font-mono">
            <div className="text-slate-500">Transaction Receipt:</div>
            <div className="text-cyan-400 mt-1 break-all">{txReceipt}</div>
            <div className="text-slate-500 mt-2">Network: Stellar Soroban Testnet</div>
          </div>

          <div className="flex gap-4 justify-center">
            <button
              onClick={() => {
                setStatus("idle");
                setMessage("");
              }}
              className="px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-sm font-medium text-slate-200 transition-colors"
            >
              Send Another Gift
            </button>
            <Link
              href="/dashboard"
              className="px-5 py-2.5 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 text-sm font-semibold text-white transition-all shadow hover:from-cyan-400 hover:to-blue-500"
            >
              View in Dashboard
            </Link>
          </div>
        </div>
      ) : (
        <form
          onSubmit={handleSubmit}
          className="p-8 rounded-2xl bg-slate-900/90 border border-slate-800 shadow-xl space-y-6"
        >
          {/* Recipient Input */}
          <div>
            <label className="block text-xs font-medium uppercase tracking-wider text-slate-300 mb-2">
              Creator Stellar Address
            </label>
            <input
              type="text"
              required
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              className="w-full px-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-slate-100 text-sm font-mono focus:outline-none focus:border-cyan-500 transition-colors"
              placeholder="G..."
            />
          </div>

          {/* Amount Presets & Custom */}
          <div>
            <label className="block text-xs font-medium uppercase tracking-wider text-slate-300 mb-2">
              Gift Amount (XLM)
            </label>
            <div className="grid grid-cols-4 gap-3 mb-3">
              {presets.map((val) => (
                <button
                  type="button"
                  key={val}
                  onClick={() => setAmount(val)}
                  className={`py-2.5 rounded-xl text-sm font-bold border transition-all ${
                    amount === val
                      ? "bg-cyan-500/20 border-cyan-500 text-cyan-300 shadow-sm"
                      : "bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-200"
                  }`}
                >
                  {val} XLM
                </button>
              ))}
            </div>

            <div className="relative">
              <input
                type="number"
                min="1"
                step="1"
                value={amount}
                onChange={(e) => setAmount(Number(e.target.value))}
                className="w-full px-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-slate-100 text-sm font-semibold focus:outline-none focus:border-cyan-500"
                placeholder="Custom amount"
              />
              <span className="absolute right-4 top-3 text-sm text-slate-500">XLM</span>
            </div>
          </div>

          {/* Personalized Message */}
          <div>
            <label className="block text-xs font-medium uppercase tracking-wider text-slate-300 mb-2">
              Personalized Note (Optional)
            </label>
            <textarea
              rows={3}
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              className="w-full px-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-slate-100 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              placeholder="Say something inspiring to the creator..."
            />
          </div>

          {/* Anonymous toggle */}
          <div className="flex items-center space-x-2">
            <input
              type="checkbox"
              id="anon-check"
              checked={isAnonymous}
              onChange={(e) => setIsAnonymous(e.target.checked)}
              className="rounded bg-slate-950 border-slate-800 text-cyan-500 focus:ring-0 focus:ring-offset-0"
            />
            <label htmlFor="anon-check" className="text-xs text-slate-400 cursor-pointer">
              Send anonymously (hide my address from public board)
            </label>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            disabled={status === "submitting"}
            className="w-full py-3.5 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white font-semibold shadow-lg shadow-cyan-900/30 transition-all disabled:opacity-50 text-sm"
          >
            {status === "submitting" ? "Invoking Soroban Contract..." : `Confirm & Send ${amount} XLM`}
          </button>
        </form>
      )}
    </div>
  );
}
