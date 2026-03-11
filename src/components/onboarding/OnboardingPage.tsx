import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowRight, CheckCircle2, Download, Loader2, AlertCircle } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/store/settingsStore";

interface Props { onComplete: () => void }

const STEPS = [
  { id: "welcome", title: "Welcome to Meridian",  desc: "Your AI-powered personal productivity assistant. Let's get you set up in 3 quick steps." },
  { id: "profile", title: "What's your name?",    desc: "We'll use this to personalize your daily greetings." },
  { id: "ai",      title: "Local AI Setup",        desc: "Meridian runs AI completely on your device — no data ever leaves your machine." },
  { id: "ready",   title: "You're all set!",       desc: "Head to Settings to connect Gmail, Slack, Jira and more." },
];

type PullStatus = "idle" | "downloading" | "done" | "error";

interface PullProgress {
  model:     string;
  status:    string;
  total:     number;
  completed: number;
  percent:   number;
}

export function OnboardingPage({ onComplete }: Props) {
  const [step,       setStep]       = useState(0);
  const [name,       setName]       = useState("");
  const [pullStatus, setPullStatus] = useState<PullStatus>("idle");
  const [pullPct,    setPullPct]    = useState(0);
  const [pullMsg,    setPullMsg]    = useState("");
  const [pullError,  setPullError]  = useState("");
  const unlistenRef = useRef<(() => void) | null>(null);
  const { save, settings } = useSettingsStore();

  // Subscribe to Ollama pull-progress events once.
  useEffect(() => {
    let cleanup: (() => void) | null = null;
    listen<PullProgress>("ollama://pull-progress", (event) => {
      const { percent, status } = event.payload;
      setPullPct(percent);
      setPullMsg(status);
      if (status === "success") {
        setPullStatus("done");
      }
    }).then((fn) => {
      cleanup = fn;
      unlistenRef.current = fn;
    });
    return () => { cleanup?.(); };
  }, []);

  async function startDownload() {
    setPullStatus("downloading");
    setPullError("");
    setPullPct(0);
    setPullMsg("Connecting to Ollama...");
    try {
      await invoke("pull_model", { model: "phi3:mini" });
      // If the command resolves without error and the event hasn't set "done" yet, set it now.
      setPullStatus((prev) => (prev === "downloading" ? "done" : prev));
    } catch (err: unknown) {
      setPullStatus("error");
      setPullError(String(err));
    }
  }

  async function next() {
    if (step === 1 && name.trim() && settings) {
      await save({ ...settings, user_name: name.trim() });
    }
    if (step < STEPS.length - 1) {
      setStep(step + 1);
    } else {
      unlistenRef.current?.();
      onComplete();
    }
  }

  const current = STEPS[step];

  return (
    <div className="fixed inset-0 bg-gray-950 flex items-center justify-center p-8">
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          className="max-w-lg w-full space-y-8 text-center"
        >
          {/* Logo */}
          <div className="flex justify-center">
            <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-glow-primary">
              <span className="text-4xl font-bold text-white">M</span>
            </div>
          </div>

          {/* Step text */}
          <div className="space-y-3">
            <h1 className="text-3xl font-bold text-foreground">{current.title}</h1>
            <p className="text-muted-foreground">{current.desc}</p>
          </div>

          {/* ── Name input ────────────────────────────────────────────── */}
          {step === 1 && (
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && next()}
              placeholder="Your name"
              className="w-full max-w-sm mx-auto block bg-card border border-border rounded-xl px-5 py-3 text-lg text-center text-foreground outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground"
            />
          )}

          {/* ── AI / Ollama setup ─────────────────────────────────────── */}
          {step === 2 && (
            <div className="bg-card border border-border rounded-xl p-5 space-y-4 text-left">
              <p className="text-sm font-medium text-foreground">
                Phi-3 Mini — 2.3 GB, fast local AI model
              </p>

              {pullStatus === "idle" && (
                <div className="space-y-3">
                  {[
                    "Runs 100% on-device — no internet after download",
                    "Powers smart summaries, email drafting & assistant chat",
                    "~2.3 GB one-time download, uses ~2 GB RAM",
                  ].map((s) => (
                    <div key={s} className="flex items-start gap-2">
                      <CheckCircle2 size={14} className="text-green-400 mt-0.5 flex-shrink-0" />
                      <p className="text-sm text-muted-foreground">{s}</p>
                    </div>
                  ))}
                  <button
                    onClick={startDownload}
                    className="w-full flex items-center justify-center gap-2 mt-2 px-4 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors"
                  >
                    <Download size={15} />
                    Download Phi-3 Mini Now
                  </button>
                  <button
                    onClick={next}
                    className="w-full text-xs text-muted-foreground hover:text-foreground transition-colors py-1"
                  >
                    Skip — set up AI later
                  </button>
                </div>
              )}

              {pullStatus === "downloading" && (
                <div className="space-y-3">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 size={14} className="animate-spin text-primary flex-shrink-0" />
                    <span className="truncate">{pullMsg || "Downloading..."}</span>
                    <span className="ml-auto font-mono text-foreground tabular-nums">{pullPct}%</span>
                  </div>
                  <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                    <motion.div
                      className="h-full bg-gradient-to-r from-indigo-500 to-violet-500 rounded-full"
                      animate={{ width: `${pullPct}%` }}
                      transition={{ duration: 0.3 }}
                    />
                  </div>
                  <p className="text-xs text-muted-foreground text-center">
                    Keep this window open — download in progress
                  </p>
                </div>
              )}

              {pullStatus === "done" && (
                <div className="flex items-center gap-3 py-2">
                  <CheckCircle2 size={20} className="text-green-400 flex-shrink-0" />
                  <div>
                    <p className="text-sm font-medium text-foreground">phi3:mini ready!</p>
                    <p className="text-xs text-muted-foreground">AI features are fully enabled.</p>
                  </div>
                </div>
              )}

              {pullStatus === "error" && (
                <div className="space-y-3">
                  <div className="flex items-start gap-2 text-red-400">
                    <AlertCircle size={16} className="mt-0.5 flex-shrink-0" />
                    <p className="text-sm">{pullError || "Download failed — check that Ollama is running."}</p>
                  </div>
                  <button
                    onClick={startDownload}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors"
                  >
                    <Download size={15} />
                    Retry Download
                  </button>
                  <button
                    onClick={next}
                    className="w-full text-xs text-muted-foreground hover:text-foreground transition-colors py-1"
                  >
                    Skip for now
                  </button>
                </div>
              )}
            </div>
          )}

          {/* ── Integration previews ──────────────────────────────────── */}
          {step === 3 && (
            <div className="grid grid-cols-2 gap-3">
              {["Email", "Slack", "Jira", "Zoom"].map((i) => (
                <div key={i} className="bg-card border border-border rounded-xl p-3 text-sm text-muted-foreground">
                  {i}
                </div>
              ))}
            </div>
          )}

          {/* Progress dots */}
          <div className="flex justify-center gap-2">
            {STEPS.map((_, i) => (
              <div
                key={i}
                className={`h-1.5 rounded-full transition-all ${
                  i === step ? "w-6 bg-primary" : "w-1.5 bg-muted"
                }`}
              />
            ))}
          </div>

          {/* Next / finish button — hidden while a download is in progress */}
          {pullStatus !== "downloading" && (
            <button
              onClick={next}
              className="inline-flex items-center gap-2 px-6 py-3 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-colors"
            >
              {step === STEPS.length - 1 ? "Go to Dashboard" : "Continue"}
              <ArrowRight size={16} />
            </button>
          )}
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
