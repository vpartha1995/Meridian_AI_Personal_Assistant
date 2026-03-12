import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { CheckCircle2 } from "lucide-react";
import { useSettingsStore } from "@/store/settingsStore";
import { integrationApi, settingsApi } from "@/lib/tauri";
import { IntegrationCard } from "./IntegrationCard";
import { ScanSettings } from "./ScanSettings";
import type { IntegrationInfo, AppSettings } from "@/lib/types";
import { cn } from "@/lib/utils";
import { Moon, Sun, User, Puzzle, Clock, Palette, KeyRound } from "lucide-react";

type SettingsTab = "profile" | "integrations" | "schedule" | "appearance" | "oauth";

const TABS: { id: SettingsTab; label: string; icon: React.ComponentType<any> }[] = [
  { id: "profile",      label: "Profile",      icon: User     },
  { id: "integrations", label: "Integrations", icon: Puzzle   },
  { id: "schedule",     label: "Schedule",     icon: Clock    },
  { id: "appearance",   label: "Appearance",   icon: Palette  },
  { id: "oauth",        label: "OAuth Setup",  icon: KeyRound },
];

export function SettingsPage() {
  const { settings, save, theme, setTheme } = useSettingsStore();
  const [tab,          setTab]          = useState<SettingsTab>("integrations");
  const [integrations, setIntegrations] = useState<IntegrationInfo[]>([]);
  const [localSettings, setLocal]       = useState<AppSettings | null>(null);
  const [saving, setSaving]             = useState(false);
  const [saved,  setSaved]              = useState(false);

  // Slack OAuth credential state
  const [slackClientId,     setSlackClientId]     = useState("");
  const [slackClientSecret, setSlackClientSecret] = useState("");
  const [slackSaving,       setSlackSaving]       = useState(false);
  const [slackSaved,        setSlackSaved]        = useState(false);
  const [slackConfigured,   setSlackConfigured]   = useState<string | null>(null);

  useEffect(() => {
    if (settings) setLocal({ ...settings });
    integrationApi.list().then(setIntegrations).catch(() => {});
  }, [settings]);

  useEffect(() => {
    if (tab === "oauth") {
      settingsApi.getOAuthClientId("slack")
        .then((id) => setSlackConfigured(id))
        .catch(() => setSlackConfigured(null));
    }
  }, [tab]);

  async function handleSave() {
    if (!localSettings) return;
    setSaving(true);
    try {
      await save(localSettings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } finally {
      setSaving(false);
    }
  }

  async function handleSaveSlack() {
    if (!slackClientId.trim() || !slackClientSecret.trim()) return;
    setSlackSaving(true);
    try {
      await settingsApi.setOAuthCreds("slack", slackClientId.trim(), slackClientSecret.trim());
      setSlackConfigured(slackClientId.trim());
      setSlackClientId("");
      setSlackClientSecret("");
      setSlackSaved(true);
      setTimeout(() => setSlackSaved(false), 2500);
    } catch (e) {
      alert(`Failed to save Slack credentials: ${e}`);
    } finally {
      setSlackSaving(false);
    }
  }

  async function handleClearSlack() {
    await settingsApi.deleteOAuthCreds("slack").catch(() => {});
    setSlackConfigured(null);
  }

  function updateLocal(patch: Partial<AppSettings>) {
    setLocal((s) => s ? { ...s, ...patch } : s);
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex-shrink-0 px-6 py-4 border-b border-border">
        <h2 className="text-lg font-semibold text-foreground">Settings</h2>
        <p className="text-sm text-muted-foreground">Manage your Meridian configuration</p>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Tab sidebar */}
        <nav className="w-48 flex-shrink-0 border-r border-border p-3 space-y-1">
          {TABS.map((t) => {
            const Icon = t.icon;
            return (
              <button
                key={t.id}
                onClick={() => setTab(t.id)}
                className={cn(
                  "w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-all text-left",
                  tab === t.id
                    ? "bg-primary/10 text-primary font-medium"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                )}
              >
                <Icon size={15} />
                {t.label}
              </button>
            );
          })}
        </nav>

        {/* Tab content */}
        <div className="flex-1 overflow-y-auto p-6">
          <motion.div
            key={tab}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.15 }}
            className="max-w-xl space-y-6"
          >
            {/* ── Profile ── */}
            {tab === "profile" && localSettings && (
              <>
                <Section title="Your Profile">
                  <Field label="Name">
                    <input
                      value={localSettings.user_name}
                      onChange={(e) => updateLocal({ user_name: e.target.value })}
                      className="input-field"
                    />
                  </Field>
                </Section>
                <SaveButton saving={saving} saved={saved} onSave={handleSave} />
              </>
            )}

            {/* ── Integrations ── */}
            {tab === "integrations" && (
              <Section title="Connected Accounts" subtitle="Link your work tools to enable smart summaries.">
                <div className="space-y-3">
                  {integrations.map((i) => (
                    <IntegrationCard
                      key={i.id}
                      info={i}
                      onRefresh={() => integrationApi.list().then(setIntegrations)}
                    />
                  ))}
                  {integrations.length === 0 && (
                    <p className="text-sm text-muted-foreground italic">Loading integrations…</p>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-4">
                  To connect Slack, first add your Slack App credentials in the <strong>OAuth Setup</strong> tab.
                </p>
              </Section>
            )}

            {/* ── Schedule ── */}
            {tab === "schedule" && localSettings && (
              <>
                <ScanSettings settings={localSettings} onChange={updateLocal} />
                <SaveButton saving={saving} saved={saved} onSave={handleSave} />
              </>
            )}

            {/* ── Appearance ── */}
            {tab === "appearance" && (
              <Section title="Theme">
                <div className="flex gap-3">
                  {(["dark", "light"] as const).map((t) => (
                    <button
                      key={t}
                      onClick={() => setTheme(t)}
                      className={cn(
                        "flex-1 flex flex-col items-center gap-2 p-4 rounded-xl border transition-all",
                        theme === t
                          ? "border-primary bg-primary/10 text-primary"
                          : "border-border text-muted-foreground hover:border-primary/30"
                      )}
                    >
                      {t === "dark" ? <Moon size={20} /> : <Sun size={20} />}
                      <span className="text-sm font-medium capitalize">{t} Mode</span>
                    </button>
                  ))}
                </div>
              </Section>
            )}

            {/* ── OAuth Setup ── */}
            {tab === "oauth" && (
              <>
                <Section
                  title="Slack OAuth App Credentials"
                  subtitle="Meridian uses your own Slack App so your data never passes through third-party servers. Create a free app at api.slack.com/apps."
                >
                  {slackConfigured && (
                    <div className="flex items-center justify-between p-3 bg-green-500/10 border border-green-500/20 rounded-lg mb-4">
                      <div className="flex items-center gap-2 text-green-400 text-sm">
                        <CheckCircle2 size={14} />
                        <span>Configured — Client ID: <code className="font-mono text-xs">{slackConfigured}</code></span>
                      </div>
                      <button
                        onClick={handleClearSlack}
                        className="text-xs text-muted-foreground hover:text-destructive transition-colors"
                      >
                        Remove
                      </button>
                    </div>
                  )}

                  <div className="space-y-4">
                    <div className="p-3 bg-muted/50 rounded-lg text-xs text-muted-foreground space-y-1">
                      <p className="font-medium text-foreground">Setup steps:</p>
                      <ol className="list-decimal list-inside space-y-1">
                        <li>Go to <span className="font-mono">api.slack.com/apps</span> → Create New App</li>
                        <li>Under OAuth &amp; Permissions, add redirect URL: <span className="font-mono">http://localhost:*</span></li>
                        <li>Add scopes: channels:history, channels:read, im:history, search:read, users:read</li>
                        <li>Copy your Client ID and Client Secret below</li>
                      </ol>
                    </div>

                    <Field label="Client ID">
                      <input
                        value={slackClientId}
                        onChange={(e) => setSlackClientId(e.target.value)}
                        placeholder="1234567890.1234567890"
                        className="input-field font-mono text-sm"
                      />
                    </Field>
                    <Field label="Client Secret">
                      <input
                        type="password"
                        value={slackClientSecret}
                        onChange={(e) => setSlackClientSecret(e.target.value)}
                        placeholder="••••••••••••••••••••••••••••••••"
                        className="input-field font-mono text-sm"
                      />
                    </Field>

                    <button
                      onClick={handleSaveSlack}
                      disabled={!slackClientId.trim() || !slackClientSecret.trim() || slackSaving}
                      className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
                    >
                      {slackSaved ? (
                        <><CheckCircle2 size={14} className="text-green-300" /> Saved!</>
                      ) : slackSaving ? "Saving…" : "Save Slack Credentials"}
                    </button>
                  </div>
                </Section>
              </>
            )}
          </motion.div>
        </div>
      </div>
    </div>
  );
}

function Section({ title, subtitle, children }: { title: string; subtitle?: string; children: React.ReactNode }) {
  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-foreground">{title}</h3>
        {subtitle && <p className="text-xs text-muted-foreground mt-0.5">{subtitle}</p>}
      </div>
      {children}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-1.5">
      <label className="text-xs font-medium text-muted-foreground">{label}</label>
      {children}
    </div>
  );
}

function SaveButton({ saving, saved, onSave }: { saving: boolean; saved: boolean; onSave: () => void }) {
  return (
    <button
      onClick={onSave}
      disabled={saving}
      className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
    >
      {saved ? (
        <><CheckCircle2 size={14} className="text-green-300" /> Saved!</>
      ) : saving ? "Saving…" : "Save Changes"}
    </button>
  );
}
