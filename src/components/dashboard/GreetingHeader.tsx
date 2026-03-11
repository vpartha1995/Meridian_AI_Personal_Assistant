import { getGreeting, formatDate } from "@/lib/utils";

interface Props {
  userName?:    string;
  motivational?: string;
}

function getWeatherEmoji(): string {
  const h = new Date().getHours();
  if (h < 6)  return "🌙";
  if (h < 12) return "🌤";
  if (h < 17) return "☀️";
  if (h < 20) return "🌇";
  return "🌙";
}

export function GreetingHeader({ userName, motivational }: Props) {
  return (
    <div className="flex flex-col gap-0.5">
      <div className="flex items-center gap-2">
        <span>{getWeatherEmoji()}</span>
        <h1 className="text-lg font-semibold text-foreground">
          {getGreeting(userName)}
        </h1>
      </div>
      <p className="text-xs text-muted-foreground">
        {formatDate(new Date().toISOString())}
        {motivational && (
          <> &nbsp;·&nbsp; <span className="italic">"{motivational}"</span></>
        )}
      </p>
    </div>
  );
}
