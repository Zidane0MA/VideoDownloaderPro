import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Download, Images, Settings, Zap } from "lucide-react";

function App() {
  const { t, i18n } = useTranslation();
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  const toggleLanguage = () => {
    const newLang = i18n.language === "en" ? "es" : "en";
    i18n.changeLanguage(newLang);
  };

  return (
    <div className="min-h-screen bg-surface-900 text-surface-100">
      {/* Header */}
      <header className="border-b border-surface-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-brand-500 to-brand-700 shadow-lg shadow-brand-500/25">
              <Zap className="w-5 h-5 text-white" />
            </div>
            <div>
              <h1 className="text-lg font-semibold tracking-tight">
                {t("app.name")}
              </h1>
              <p className="text-xs text-surface-200/60">{t("app.tagline")}</p>
            </div>
          </div>
          <button
            onClick={toggleLanguage}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-surface-700 hover:bg-surface-700/80 transition-colors"
          >
            {i18n.language.toUpperCase()}
          </button>
        </div>
      </header>

      {/* Navigation */}
      <nav className="border-b border-surface-700 px-6">
        <div className="flex gap-1">
          {[
            { icon: Download, label: t("nav.downloads"), active: true },
            { icon: Images, label: t("nav.wall"), active: false },
            { icon: Settings, label: t("nav.settings"), active: false },
          ].map((item) => (
            <button
              key={item.label}
              className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                item.active
                  ? "border-brand-500 text-brand-400"
                  : "border-transparent text-surface-200/60 hover:text-surface-100"
              }`}
            >
              <item.icon className="w-4 h-4" />
              {item.label}
            </button>
          ))}
        </div>
      </nav>

      {/* Main Content */}
      <main className="p-6">
        {/* URL Input */}
        <div className="max-w-2xl mx-auto">
          <div className="relative group">
            <input
              type="text"
              placeholder={t("placeholder.paste_url")}
              className="w-full px-5 py-4 bg-surface-800 border border-surface-700 rounded-2xl text-surface-100 placeholder-surface-200/40 focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all"
            />
            <button className="absolute right-2 top-1/2 -translate-y-1/2 px-5 py-2.5 bg-brand-600 hover:bg-brand-500 text-white text-sm font-medium rounded-xl transition-colors shadow-lg shadow-brand-600/25">
              {t("actions.download")}
            </button>
          </div>
        </div>

        {/* IPC Test Section */}
        <div className="max-w-2xl mx-auto mt-12">
          <div className="p-6 rounded-2xl bg-surface-800/50 border border-surface-700">
            <h2 className="text-sm font-semibold text-surface-200/60 uppercase tracking-wider mb-4">
              IPC Test
            </h2>
            <form
              className="flex gap-3"
              onSubmit={(e) => {
                e.preventDefault();
                greet();
              }}
            >
              <input
                id="greet-input"
                onChange={(e) => setName(e.currentTarget.value)}
                placeholder="Enter a name..."
                className="flex-1 px-4 py-2.5 bg-surface-900 border border-surface-700 rounded-xl text-surface-100 placeholder-surface-200/40 focus:outline-none focus:ring-2 focus:ring-brand-500/50"
              />
              <button
                type="submit"
                className="px-6 py-2.5 bg-surface-700 hover:bg-surface-700/80 text-surface-100 text-sm font-medium rounded-xl transition-colors"
              >
                Greet
              </button>
            </form>
            {greetMsg && (
              <p className="mt-4 text-sm text-brand-400 animate-pulse">
                {greetMsg}
              </p>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
