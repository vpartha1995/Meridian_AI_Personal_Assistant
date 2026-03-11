import { motion } from "framer-motion";
import { ExternalLink } from "lucide-react";
import { PriorityBadge } from "@/components/shared/PriorityBadge";
import { windowApi } from "@/lib/tauri";
import { sourceIcon, truncate, cn } from "@/lib/utils";
import type { PriorityItem } from "@/lib/types";

interface Props { item: PriorityItem }

export function PriorityItemCard({ item }: Props) {
  const borderColor = {
    high:   "border-red-500/30 hover:border-red-500/50",
    medium: "border-yellow-500/30 hover:border-yellow-500/50",
    low:    "border-border hover:border-border",
  }[item.priority] ?? "border-border";

  const bgColor = {
    high:   "bg-red-950/20",
    medium: "bg-yellow-950/10",
    low:    "bg-card",
  }[item.priority] ?? "bg-card";

  return (
    <motion.div
      whileHover={{ scale: 1.005 }}
      className={cn(
        "flex items-center gap-3 p-3 rounded-xl border transition-all cursor-default",
        borderColor, bgColor
      )}
    >
      <span className="text-lg flex-shrink-0">{sourceIcon(item.source)}</span>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground truncate">{item.title}</p>
        <p className="text-xs text-muted-foreground mt-0.5 truncate">{item.action}</p>
      </div>
      <div className="flex items-center gap-2 flex-shrink-0">
        <PriorityBadge priority={item.priority as any} label />
        {item.url && (
          <button
            onClick={() => windowApi.openUrl(item.url!)}
            className="text-muted-foreground hover:text-primary transition-colors"
            title="Open"
          >
            <ExternalLink size={14} />
          </button>
        )}
      </div>
    </motion.div>
  );
}
