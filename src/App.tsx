import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Download, Images, Settings, Zap, Plus } from "lucide-react";
import { AddDownloadModal } from "./components/AddDownloadModal";
import { DownloadsList } from "./components/DownloadsList";

function App() {
  const { t, i18n } = useTranslation();
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  const toggleLanguage = () => {
    const newLang = i18n.language === "en" ? "es" : "en";
    i18n.changeLanguage(newLang);
  };

  return (
    <div className="min-h-screen bg-surface-900 text-surface-100">
      <AddDownloadModal 
        isOpen={isAddModalOpen} 
        onClose={() => setIsAddModalOpen(false)} 
      />
      
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
            <button 
              onClick={() => setIsAddModalOpen(true)}
              className="w-full flex items-center justify-between px-5 py-4 bg-surface-800 border border-surface-700 rounded-2xl text-surface-200/40 hover:border-brand-500/50 hover:text-surface-200 transition-all cursor-text text-left"
            >
              <span>{t("placeholder.paste_url")}</span>
              <div className="px-5 py-2.5 bg-brand-600 group-hover:bg-brand-500 text-white text-sm font-medium rounded-xl transition-colors shadow-lg shadow-brand-600/25 flex items-center gap-2">
                <Plus size={18} />
                {t("actions.download")}
              </div>
            </button>
          </div>
        </div>

        {/* Downloads List */}
        <div className="max-w-2xl mx-auto mt-12">
          <DownloadsList />
        </div>
      </main>
    </div>
  );
}

export default App;
