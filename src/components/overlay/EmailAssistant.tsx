import { useState } from "react";
import { ArrowLeft, Loader2, Copy, Check } from "lucide-react";
import { aiApi } from "@/lib/tauri";

interface Props { onBack: () => void }

type Tab = "draft" | "rephrase";

const TONES = ["professional", "friendly", "concise", "assertive", "empathetic"];

export function EmailAssistant({ onBack }: Props) {
  const [tab,       setTab]      = useState<Tab>("draft");
  const [input,     setInput]    = useState("");
  const [tone,      setTone]     = useState("professional");
  const [output,    setOutput]   = useState("");
  const [loading,   setLoading]  = useState(false);
  const [copied,    setCopied]   = useState(false);

  async function handleGenerate() {
    if (!input.trim()) return;
    setLoading(true);
    setOutput("");
    try {
      if (tab === "draft") {
        const result = await aiApi.draftEmail("", input);
        setOutput(result);
      } else {
        const result = await aiApi.rephrase(input, tone);
        setOutput(result);
      }
    } catch (e) {
      setOutput("AI is not available. Make sure Ollama is running with phi3:mini.");
    } finally {
      setLoading(false);
    }
  }

  function copyOutput() {
    navigator.clipboard.writeText(output);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <button onClick={onBack} className="text-white/40 hover:text-white/80 transition-colors">
          <ArrowLeft size={14} />
        </button>
        <p className="text-sm font-medium text-white">Email Assistant</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-white/5 rounded-lg p-0.5">
        {(["draft", "rephrase"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex-1 py-1.5 rounded-md text-xs font-medium transition-all ${
              tab === t ? "bg-indigo-600 text-white" : "text-white/50 hover:text-white/80"
            }`}
          >
            {t === "draft" ? "Draft" : "Rephrase"}
          </button>
        ))}
      </div>

      <textarea
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder={tab === "draft" ? "Describe the email you need to write…" : "Paste text to rephrase…"}
        rows={3}
        className="w-full bg-white/5 border border-white/10 rounded-xl px-3 py-2.5 text-xs text-white placeholder:text-white/30 outline-none focus:border-indigo-500/50 resize-none"
      />

      {tab === "rephrase" && (
        <div className="flex gap-1 flex-wrap">
          {TONES.map((t) => (
            <button
              key={t}
              onClick={() => setTone(t)}
              className={`px-2 py-0.5 rounded-full text-xs border transition-all ${
                tone === t
                  ? "bg-indigo-600/40 border-indigo-500/50 text-indigo-300"
                  : "bg-white/5 border-white/10 text-white/50 hover:text-white/80"
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      )}

      <button
        onClick={handleGenerate}
        disabled={!input.trim() || loading}
        className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white text-xs font-medium transition-colors"
      >
        {loading ? <><Loader2 size={12} className="animate-spin" /> Generating…</> : "Generate"}
      </button>

      {output && (
        <div className="relative bg-white/5 border border-white/10 rounded-xl p-3">
          <p className="text-xs text-white/80 leading-relaxed whitespace-pre-wrap">{output}</p>
          <button
            onClick={copyOutput}
            className="absolute top-2 right-2 text-white/40 hover:text-white/80 transition-colors"
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
          </button>
        </div>
      )}
    </div>
  );
}
