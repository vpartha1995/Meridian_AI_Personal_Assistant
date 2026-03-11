import { Video } from "lucide-react";

export function MeetingWidget() {
  return (
    <div className="bg-card border border-border rounded-xl p-4 flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-lg">📅</span>
        <span className="text-sm font-semibold text-sky-400">Calendar</span>
      </div>
      <p className="text-xs text-muted-foreground italic">
        Calendar integration coming soon. Connect Google Calendar or Outlook Calendar in Settings.
      </p>
    </div>
  );
}
