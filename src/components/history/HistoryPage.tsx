import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Calendar, ChevronDown, ChevronRight, MessageSquare, Mail, Ticket, Video, Hash } from "lucide-react";
import { summaryApi } from "@/lib/tauri";
import type { DailySummary } from "@/lib/types";
import { format, parseISO } from "date-fns";
import { cn } from "@/lib/utils";

export function HistoryPage() {
  const [summaries, setSummaries] = useState<DailySummary[]>([]);
  const [loading, setLoading]     = useState(true);
  const [expanded, setExpanded]   = useState<Set<string>>(new Set());

  useEffect(() => {
    summaryApi.history(30)
      .then(setSummaries)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  function toggle(date: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      next.has(date) ? next.delete(date) : next.add(date);
      return next;
    });
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-shrink-0 px-6 py-4 border-b border-border">
        <h2 className="text-lg font-semibold text-foreground">History</h2>
        <p className="text-sm text-muted-foreground">Your daily AI briefings from the past 30 days</p>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {loading && (
          <div className="flex items-center justify-center h-32 text-muted-foreground text-sm">
            Loading history…
          </div>
        )}

        {!loading && summaries.length === 0 && (
          <div className="flex flex-col items-center justify-center h-64 gap-3 text-center">
            <Calendar size={40} className="text-muted-foreground/30" />
            <p className="text-muted-foreground">No history yet.</p>
            <p className="text-xs text-muted-foreground">Connect integrations and trigger a sync to generate daily briefings.</p>
          </div>
        )}

        <div className="space-y-3 max-w-2xl">
          {summaries.map((s, i) => {
            const date = s.generated_at
              ? format(parseISO(s.generated_at), "EEEE, MMMM d, yyyy")
              : "Unknown date";
            const isOpen = expanded.has(s.generated_at);

            return (
              <motion.div
                key={s.generated_at}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.04 }}
                className="border border-border rounded-xl overflow-hidden bg-card"
              >
                {/* Header row */}
                <button
                  onClick={() => toggle(s.generated_at)}
                  className="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/50 transition-colors text-left"
                >
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                      <Calendar size={14} className="text-primary" />
                    </div>
                    <div>
                      <p className="text-sm font-medium text-foreground">{date}</p>
                      <p className="text-xs text-muted-foreground">
                        {s.raw_event_count} update{s.raw_event_count !== 1 ? "s" : ""}
                        {s.motivational ? ` · ${s.motivational.slice(0, 50)}…` : ""}
                      </p>
                    </div>
                  </div>
                  {isOpen
                    ? <ChevronDown size={14} className="text-muted-foreground flex-shrink-0" />
                    : <ChevronRight size={14} className="text-muted-foreground flex-shrink-0" />}
                </button>

                {/* Expanded content */}
                {isOpen && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="border-t border-border px-4 py-4 space-y-4"
                  >
                    {s.motivational && (
                      <blockquote className="text-xs text-muted-foreground italic border-l-2 border-primary/40 pl-3">
                        {s.motivational}
                      </blockquote>
                    )}

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <SummaryBlock icon={<Mail size={12} />}           label="Emails"      text={s.email_summary}  />
                      <SummaryBlock icon={<Hash size={12} />}           label="Slack"       text={s.slack_summary}  />
                      <SummaryBlock icon={<Ticket size={12} />}         label="Jira"        text={s.jira_summary}   />
                      <SummaryBlock icon={<Video size={12} />}          label="Zoom"        text={s.zoom_summary}   />
                      <SummaryBlock icon={<MessageSquare size={12} />}  label="Google Chat" text={s.gchat_summary}  />
                    </div>

                    {s.top_priorities.length > 0 && (
                      <div>
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-2">
                          Priorities
                        </p>
                        <div className="space-y-1.5">
                          {s.top_priorities.map((p, j) => (
                            <div key={j} className="flex items-start gap-2 text-xs text-foreground">
                              <span className={cn(
                                "mt-0.5 h-1.5 w-1.5 rounded-full flex-shrink-0",
                                p.priority === "high"   ? "bg-red-400"    :
                                p.priority === "medium" ? "bg-yellow-400" : "bg-green-400"
                              )} />
                              <span>{p.title}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </motion.div>
                )}
              </motion.div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function SummaryBlock({ icon, label, text }: { icon: React.ReactNode; label: string; text: string }) {
  if (!text) return null;
  return (
    <div className="space-y-1">
      <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        {icon} {label}
      </div>
      <p className="text-xs text-foreground/80 leading-relaxed line-clamp-3">{text}</p>
    </div>
  );
}
