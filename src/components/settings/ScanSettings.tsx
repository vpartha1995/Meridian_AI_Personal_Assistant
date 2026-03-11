import type { AppSettings } from "@/lib/types";

interface Props {
  settings: AppSettings;
  onChange: (patch: Partial<AppSettings>) => void;
}

export function ScanSettings({ settings, onChange }: Props) {
  return (
    <div className="space-y-5">
      <div className="space-y-1.5">
        <label className="text-xs font-medium text-muted-foreground">Sync Interval</label>
        <select
          value={settings.scan_interval_mins}
          onChange={(e) => onChange({ scan_interval_mins: Number(e.target.value) })}
          className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
        >
          <option value={30}>Every 30 minutes</option>
          <option value={60}>Every hour</option>
          <option value={120}>Every 2 hours</option>
          <option value={0}>On login only</option>
        </select>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">Work Day Start</label>
          <input
            type="number"
            min={0} max={23}
            value={settings.working_hours_start}
            onChange={(e) => onChange({ working_hours_start: Number(e.target.value) })}
            className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
          />
        </div>
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">Work Day End</label>
          <input
            type="number"
            min={0} max={23}
            value={settings.working_hours_end}
            onChange={(e) => onChange({ working_hours_end: Number(e.target.value) })}
            className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
          />
        </div>
      </div>

      <Toggle
        label="Health Nudges"
        description="Remind you to drink water, take breaks, and check posture"
        value={settings.health_nudges}
        onChange={(v) => onChange({ health_nudges: v })}
      />
      <Toggle
        label="Desktop Notifications"
        description="Show native OS notifications for reminders and alerts"
        value={settings.notifications}
        onChange={(v) => onChange({ notifications: v })}
      />
    </div>
  );
}

function Toggle({ label, description, value, onChange }: {
  label:       string;
  description: string;
  value:       boolean;
  onChange:    (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div>
        <p className="text-sm text-foreground">{label}</p>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      <button
        onClick={() => onChange(!value)}
        className={`relative w-11 h-6 rounded-full transition-colors ${value ? "bg-primary" : "bg-muted"}`}
      >
        <span className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${value ? "translate-x-5" : "translate-x-0"}`} />
      </button>
    </div>
  );
}
