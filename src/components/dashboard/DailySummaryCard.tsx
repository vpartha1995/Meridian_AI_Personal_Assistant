import { motion } from "framer-motion";
import { ExternalLink } from "lucide-react";
import { cn, sourceColor } from "@/lib/utils";

interface Props {
  icon:    string;
  title:   string;
  source:  string;
  summary: string;
}

export function DailySummaryCard({ icon, title, source, summary }: Props) {
  const isEmpty = summary.startsWith("No new") || summary.startsWith("No upcoming");

  return (
    <motion.div
      whileHover={{ scale: 1.005, y: -1 }}
      transition={{ type: "spring", stiffness: 400, damping: 30 }}
      className="bg-card border border-border rounded-xl p-4 flex flex-col gap-3 hover:border-primary/20 hover:shadow-card-hover transition-all"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-lg">{icon}</span>
          <span className={cn("text-sm font-semibold", sourceColor(source))}>{title}</span>
        </div>
        <span className={cn("text-xs px-2 py-0.5 rounded-full border", `badge-${source}`)}>
          {source}
        </span>
      </div>

      <div className="text-sm text-foreground/80 leading-relaxed">
        {isEmpty ? (
          <p className="text-muted-foreground italic">{summary}</p>
        ) : (
          <div className="space-y-1">
            {summary.split("\n").map((line, i) => (
              <p key={i} className={line.startsWith("•") ? "pl-1" : ""}>{line}</p>
            ))}
          </div>
        )}
      </div>
    </motion.div>
  );
}
