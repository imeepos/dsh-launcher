import { useTheme } from "../hooks/useTheme";

export default function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  return (
    <div className="theme-toggle" role="group" aria-label="界面主题">
      <button
        className={theme === "dark" ? "active" : ""}
        aria-pressed={theme === "dark"}
        onClick={() => setTheme("dark")}
      >
        深
      </button>
      <button
        className={theme === "light" ? "active" : ""}
        aria-pressed={theme === "light"}
        onClick={() => setTheme("light")}
      >
        浅
      </button>
    </div>
  );
}
