import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { MOTIVATIONAL_QUOTES, HEALTH_TIPS, PRODUCTIVITY_TIPS } from "@/lib/utils";

interface Props {
  message?: string;
  progress?: number;
}

export function LoadingScreen({ message = "Preparing your morning brief…", progress }: Props) {
  const [quote]       = useState(() => MOTIVATIONAL_QUOTES[Math.floor(Math.random() * MOTIVATIONAL_QUOTES.length)]);
  const [tip]         = useState(() => {
    const all = [...HEALTH_TIPS, ...PRODUCTIVITY_TIPS];
    return all[Math.floor(Math.random() * all.length)];
  });
  const [dots, setDots] = useState("");

  useEffect(() => {
    const id = setInterval(() => setDots((d) => (d.length >= 3 ? "" : d + ".")), 500);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="fixed inset-0 bg-gray-950 flex flex-col items-center justify-center gap-10 z-50">
      {/* Logo */}
      <motion.div
        initial={{ scale: 0.7, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.6, type: "spring" }}
        className="flex flex-col items-center gap-4"
      >
        <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center shadow-glow-primary">
          <span className="text-4xl font-bold text-white">M</span>
        </div>
        <div className="text-center">
          <h1 className="text-3xl font-bold gradient-text">Meridian</h1>
          <p className="text-gray-500 text-sm mt-1">Your workday, perfectly centered.</p>
        </div>
      </motion.div>

      {/* Quote */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="text-center max-w-md px-8"
      >
        <blockquote className="text-lg text-gray-300 italic leading-relaxed">
          "{quote}"
        </blockquote>
      </motion.div>

      {/* Status + progress */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="flex flex-col items-center gap-4 w-72"
      >
        <p className="text-sm text-gray-400">{message}{dots}</p>
        <div className="w-full h-1 bg-gray-800 rounded-full overflow-hidden">
          {progress !== undefined ? (
            <motion.div
              className="h-full bg-gradient-to-r from-indigo-500 to-violet-500 rounded-full"
              animate={{ width: `${progress}%` }}
              transition={{ duration: 0.4 }}
            />
          ) : (
            <motion.div
              className="h-full w-1/3 bg-gradient-to-r from-indigo-500 to-violet-500 rounded-full"
              animate={{ x: ["-100%", "300%"] }}
              transition={{ duration: 1.5, repeat: Infinity, ease: "easeInOut" }}
            />
          )}
        </div>
      </motion.div>

      {/* Tip */}
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1 }}
        className="text-xs text-gray-600 text-center max-w-xs px-8 absolute bottom-8"
      >
        {tip}
      </motion.p>
    </div>
  );
}
