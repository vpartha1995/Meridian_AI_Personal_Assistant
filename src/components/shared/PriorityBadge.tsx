import { cn } from "@/lib/utils";

interface Props {
  priority: "high" | "medium" | "low";
  size?:    "sm" | "md";
  label?:   boolean;
}

export function PriorityBadge({ priority, size = "sm", label = false }: Props) {
  const styles = {
    high:   { dot: "bg-red-500",    text: "text-red-400",    bg: "bg-red-500/10 border-red-500/20"    },
    medium: { dot: "bg-yellow-500", text: "text-yellow-400", bg: "bg-yellow-500/10 border-yellow-500/20" },
    low:    { dot: "bg-gray-500",   text: "text-gray-400",   bg: "bg-gray-500/10 border-gray-600/20"  },
  };
  const s = styles[priority] ?? styles.low;

  if (!label) {
    return <span className={cn("w-2 h-2 rounded-full flex-shrink-0", s.dot)} />;
  }

  return (
    <span className={cn(
      "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-xs font-medium",
      s.bg, s.text,
      size === "md" && "px-2.5 py-1 text-sm"
    )}>
      <span className={cn("w-1.5 h-1.5 rounded-full", s.dot)} />
      {priority.charAt(0).toUpperCase() + priority.slice(1)}
    </span>
  );
}
