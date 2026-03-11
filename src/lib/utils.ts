import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { formatDistanceToNow, format, parseISO, isToday, isYesterday } from "date-fns";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatRelativeTime(dateStr: string): string {
  try {
    const date = parseISO(dateStr);
    if (isToday(date))     return formatDistanceToNow(date, { addSuffix: true });
    if (isYesterday(date)) return "Yesterday";
    return format(date, "MMM d");
  } catch {
    return dateStr;
  }
}

export function formatTime(dateStr: string): string {
  try { return format(parseISO(dateStr), "h:mm a"); }
  catch { return ""; }
}

export function formatDate(dateStr: string): string {
  try { return format(parseISO(dateStr), "EEEE, MMMM d"); }
  catch { return dateStr; }
}

export function getGreeting(name?: string): string {
  const hour = new Date().getHours();
  const greeting =
    hour < 12 ? "Good morning" :
    hour < 17 ? "Good afternoon" :
                "Good evening";
  return name ? `${greeting}, ${name.split(" ")[0]}` : greeting;
}

export function priorityColor(priority: string): string {
  switch (priority) {
    case "high":   return "text-red-400";
    case "medium": return "text-yellow-400";
    case "low":    return "text-gray-500";
    default:       return "text-gray-400";
  }
}

export function priorityBg(priority: string): string {
  switch (priority) {
    case "high":   return "bg-red-500/10 border-red-500/20";
    case "medium": return "bg-yellow-500/10 border-yellow-500/20";
    case "low":    return "bg-gray-500/10 border-gray-600/20";
    default:       return "bg-gray-500/10 border-gray-600/20";
  }
}

export function sourceColor(source: string): string {
  const map: Record<string, string> = {
    gmail:   "text-red-400",
    outlook: "text-blue-400",
    slack:   "text-green-400",
    jira:    "text-blue-300",
    zoom:    "text-sky-400",
    gchat:   "text-emerald-400",
  };
  return map[source] ?? "text-gray-400";
}

export function sourceBadgeClass(source: string): string {
  return `badge-${source}`;
}

export function sourceIcon(source: string): string {
  const map: Record<string, string> = {
    gmail:   "📧",
    outlook: "📨",
    slack:   "💬",
    jira:    "🎫",
    zoom:    "🎥",
    gchat:   "💭",
  };
  return map[source] ?? "📋";
}

export function truncate(str: string, max: number): string {
  return str.length > max ? str.slice(0, max) + "…" : str;
}

export const MOTIVATIONAL_QUOTES = [
  "The secret of getting ahead is getting started.",
  "Focus on what matters. Let the rest wait.",
  "Small steps, consistently taken, create enormous results.",
  "Clarity is the prerequisite of execution.",
  "Deep work is the superpower of the knowledge economy.",
  "The most productive people don't just work harder — they work deliberately.",
  "One decision made well today is worth a hundred decisions deferred.",
];

export const HEALTH_TIPS = [
  "💧 Drinking water every hour improves cognitive performance by up to 30%.",
  "🧘 A 2-minute breathing break can reset focus better than coffee.",
  "🚶 Standing for 5 minutes every hour reduces fatigue significantly.",
  "👁️ The 20-20-20 rule: every 20 min, look at something 20 feet away for 20 seconds.",
  "🌱 A short walk outside boosts creativity by up to 60% (Stanford study).",
];

export const PRODUCTIVITY_TIPS = [
  "🎯 Single-tasking beats multitasking for complex work.",
  "📵 Turning off notifications for 2 hours creates deep work blocks.",
  "✅ Writing down 3 MIT (Most Important Tasks) each morning drives results.",
  "⏱️ Time-boxing tasks to 90-minute sprints matches your ultradian rhythm.",
  "🔕 Silent mornings before 10 AM are the most cognitively powerful time.",
];
