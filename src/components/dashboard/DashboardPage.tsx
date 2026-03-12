import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { RefreshCw, WifiOff, Download, X, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { useSummaryStore } from "@/store/summaryStore";
import { useSettingsStore } from "@/store/settingsStore";
import { useTaskStore } from "@/store/taskStore";
import { summaryApi, aiApi } from "@/lib/tauri";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { LoadingScreen } from "@/components/shared/LoadingScreen";
import { GreetingHeader } from "./GreetingHeader";
import { DailySummaryCard } from "./DailySummaryCard";
import { PriorityItemCard } from "./PriorityItemCard";
import { TaskWidget } from "./TaskWidget";
import { MeetingWidget } from "./MeetingWidget";
import type { OllamaStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

type PullState = "idle" | "downloading" | "done" | "error";

export function DashboardPage() {
  const { summary, setSummary, isGenerating, setIsGenerating } = useSummaryStore();
  const { settings }   = useSettingsStore();
  const { tasks }      = useTaskStore();
  const [ready, setReady]         = useState(false);
  const [aiStatus, setAiStatus]   = useState<OllamaStatus | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [aiBannerDismissed, setAiBannerDismissed] = useState(
    () => !!sessionStorage.getItem("ai-banner-dismissed")
  );
  const [pullState,   setPullState]   = useState<PullState>("idle");
  const [pullPct,     setPullPct]     = useState(0);
  const [pullMsg,     setPullMsg]     = useState("");

  useEffect(() => {
    initDashboard();
    aiApi.status().then(setAiStatus).catch(() => {});
  }, []);

  async function startModelDownload() {
    setPullState("downloading");
    setPullPct(0);
    setPullMsg("Connecting to Ollama…");

    const unlisten = await listen<{ percent: number; status: string }>(
      "ollama://pull-progress",
      ({ payload }) => {
        setPullPct(payload.percent);
        setPullMsg(payload.status);
        if (payload.status === "success") {
          setPullState("done");
          aiApi.status().then(setAiStatus).catch(() => {});
        }
      }
    );

    try {
      await invoke("pull_model", { model: "phi3:mini" });
      setPullState((prev) => prev === "downloading" ? "done" : prev);
    } catch (e) {
      setPullState("error");
      setPullMsg(String(e));
    } finally {
      unlisten();
    }
  }

  async function initDashboard() {
    setIsGenerating(true);
    try {
      const existing = await summaryApi.getToday();
      if (existing) {
        setSummary(existing);
        setReady(true);
        setIsGenerating(false);
        return;
      }
      const fresh = await summaryApi.generate();
      setSummary(fresh);
    } catch (e) {
      console.error("Dashboard init failed:", e);
    } finally {
      setIsGenerating(false);
      setReady(true);
    }
  }

  async function handleRefresh() {
    setRefreshing(true);
    try {
      const fresh = await summaryApi.generate();
      setSummary(fresh);
    } catch (e) {
      console.error("Refresh failed:", e);
    } finally {
      setRefreshing(false);
    }
  }

  if (!ready || isGenerating) {
    return <LoadingScreen message="Analyzing your updates" />;
  }

  const pendingTasks = tasks.filter((t) => !t.completed);

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Topbar */}
      <div className="flex-shrink-0 flex items-center justify-between px-6 py-3 border-b border-border bg-surface-raised">
        <GreetingHeader userName={settings?.user_name} motivational={summary?.motivational} />
        <div className="flex items-center gap-3">
          {/* AI status pill */}
          {aiStatus && (
            <div className={cn(
              "flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs border",
              aiStatus.available && aiStatus.model_ready
                ? "bg-green-500/10 border-green-500/20 text-green-400"
                : "bg-yellow-500/10 border-yellow-500/20 text-yellow-400"
            )}>
              {aiStatus.available && aiStatus.model_ready
                ? <><span className="pulse-dot" /> AI Ready</>
                : <><WifiOff size={10} /> AI Offline</>}
            </div>
          )}
          <button
            onClick={handleRefresh}
            disabled={refreshing}
            className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw size={14} className={refreshing ? "animate-spin" : ""} />
            {refreshing ? "Syncing…" : "Refresh"}
          </button>
        </div>
      </div>

      {/* AI model setup banner — shown when Ollama is running but model not pulled */}
      <AnimatePresence>
        {aiStatus && aiStatus.available && !aiStatus.model_ready && !aiBannerDismissed && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="flex-shrink-0 border-b border-yellow-500/20 bg-yellow-500/5 px-6 py-3"
          >
            {pullState === "idle" && (
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-2 text-yellow-400 text-sm">
                  <AlertCircle size={14} className="flex-shrink-0" />
                  <span>AI model (phi3:mini) is not downloaded. Download it to enable smart summaries, email drafting, and the assistant.</span>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  <button
                    onClick={startModelDownload}
                    className="flex items-center gap-1.5 px-3 py-1.5 bg-yellow-500/20 hover:bg-yellow-500/30 border border-yellow-500/30 text-yellow-300 rounded-lg text-xs font-medium transition-colors"
                  >
                    <Download size={12} /> Download (~2.3 GB)
                  </button>
                  <button
                    onClick={() => { setAiBannerDismissed(true); sessionStorage.setItem("ai-banner-dismissed", "1"); }}
                    className="text-yellow-400/50 hover:text-yellow-400 transition-colors"
                  >
                    <X size={14} />
                  </button>
                </div>
              </div>
            )}
            {pullState === "downloading" && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-2 text-yellow-400 text-xs">
                  <Loader2 size={12} className="animate-spin flex-shrink-0" />
                  <span className="truncate">{pullMsg || "Downloading phi3:mini…"}</span>
                  <span className="ml-auto font-mono tabular-nums">{pullPct}%</span>
                </div>
                <div className="w-full h-1.5 bg-yellow-500/10 rounded-full overflow-hidden">
                  <motion.div
                    className="h-full bg-yellow-400 rounded-full"
                    animate={{ width: `${pullPct}%` }}
                    transition={{ duration: 0.3 }}
                  />
                </div>
              </div>
            )}
            {pullState === "done" && (
              <div className="flex items-center gap-2 text-green-400 text-sm">
                <CheckCircle2 size={14} />
                <span>phi3:mini downloaded — AI features are now active!</span>
              </div>
            )}
            {pullState === "error" && (
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-2 text-red-400 text-xs">
                  <AlertCircle size={12} />
                  <span className="truncate">{pullMsg || "Download failed. Check that Ollama is running."}</span>
                </div>
                <button
                  onClick={startModelDownload}
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-red-500/20 border border-red-500/30 text-red-300 rounded-lg text-xs font-medium transition-colors flex-shrink-0"
                >
                  Retry
                </button>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Scrollable body */}
      <div className="flex-1 overflow-y-auto">
        <div className="flex gap-0 h-full">
          {/* Main content */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6">
            <AnimatePresence>
              {summary && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="space-y-6"
                >
                  {/* Priority items */}
                  {summary.top_priorities.length > 0 && (
                    <section>
                      <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-widest mb-3">
                        Needs Your Attention
                      </h3>
                      <div className="space-y-2">
                        {summary.top_priorities.map((item, i) => (
                          <motion.div
                            key={i}
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ delay: i * 0.08 }}
                          >
                            <PriorityItemCard item={item} />
                          </motion.div>
                        ))}
                      </div>
                    </section>
                  )}

                  {/* Summary cards */}
                  <section>
                    <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-widest mb-3">
                      Morning Brief — {summary.raw_event_count} update{summary.raw_event_count !== 1 ? "s" : ""}
                    </h3>
                    <div className="grid grid-cols-1 xl:grid-cols-2 gap-3">
                      <DailySummaryCard icon="📧" title="Emails"       source="gmail"   summary={summary.email_summary}  />
                      <DailySummaryCard icon="💬" title="Slack"        source="slack"   summary={summary.slack_summary}  />
                      <DailySummaryCard icon="🎫" title="Jira"         source="jira"    summary={summary.jira_summary}   />
                      <DailySummaryCard icon="🎥" title="Zoom"         source="zoom"    summary={summary.zoom_summary}   />
                      <DailySummaryCard icon="💭" title="Google Chat"  source="gchat"   summary={summary.gchat_summary}  />
                      <MeetingWidget />
                    </div>
                  </section>
                </motion.div>
              )}

              {!summary && (
                <div className="flex flex-col items-center justify-center h-64 gap-4 text-center">
                  <p className="text-4xl">🌅</p>
                  <p className="text-muted-foreground">No updates yet. Connect integrations in Settings to get started.</p>
                </div>
              )}
            </AnimatePresence>
          </div>

          {/* Right panel — tasks */}
          <div className="w-72 flex-shrink-0 border-l border-border overflow-y-auto p-4">
            <TaskWidget tasks={pendingTasks} />
          </div>
        </div>
      </div>
    </div>
  );
}
