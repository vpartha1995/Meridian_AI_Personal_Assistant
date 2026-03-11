import { useState } from "react";
import { CheckCircle2, XCircle, Loader2, Link, Unlink } from "lucide-react";
import { integrationApi } from "@/lib/tauri";
import { cn, sourceIcon } from "@/lib/utils";
import type { IntegrationInfo } from "@/lib/types";

interface Props {
  info:      IntegrationInfo;
  onRefresh: () => void;
}

export function IntegrationCard({ info, onRefresh }: Props) {
  const [loading, setLoading] = useState(false);
  const [error,   setError]   = useState<string | null>(null);

  const connected = info.status.status === "connected";
  const account   = connected ? (info.status as any).account : null;

  async function handleConnect() {
    setLoading(true);
    setError(null);
    try {
      await integrationApi.connect(info.id);
      onRefresh();
    } catch (e: any) {
      setError(e?.toString() ?? "Failed to connect");
    } finally {
      setLoading(false);
    }
  }

  async function handleDisconnect() {
    setLoading(true);
    try {
      await integrationApi.disconnect(info.id);
      onRefresh();
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={cn(
      "flex items-center gap-4 p-4 rounded-xl border transition-all",
      connected ? "bg-card border-border" : "bg-card border-dashed border-border"
    )}>
      {/* Icon */}
      <div className="w-10 h-10 rounded-xl bg-muted flex items-center justify-center text-lg flex-shrink-0">
        {sourceIcon(info.id)}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground">{info.name}</p>
        <p className="text-xs text-muted-foreground truncate">
          {connected ? account : "Not connected"}
        </p>
        {error && <p className="text-xs text-destructive mt-0.5">{error}</p>}
      </div>

      {/* Status + action */}
      <div className="flex items-center gap-2 flex-shrink-0">
        {connected
          ? <CheckCircle2 size={14} className="text-green-400" />
          : <XCircle     size={14} className="text-muted-foreground" />}

        <button
          onClick={connected ? handleDisconnect : handleConnect}
          disabled={loading}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors",
            connected
              ? "text-muted-foreground hover:text-destructive hover:bg-destructive/10"
              : "bg-primary/10 text-primary hover:bg-primary/20",
            loading && "opacity-50 pointer-events-none"
          )}
        >
          {loading
            ? <Loader2 size={12} className="animate-spin" />
            : connected ? <><Unlink size={12} /> Disconnect</> : <><Link size={12} /> Connect</>}
        </button>
      </div>
    </div>
  );
}
