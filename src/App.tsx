import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Download, Images, Settings as SettingsIcon, Zap, Plus } from "lucide-react";
import { AddDownloadModal } from "./components/AddDownloadModal";
import { DownloadsList } from "./components/DownloadsList";
import { Settings } from "./features/settings/Settings";
import { useSettingsStore } from "./features/settings/SettingsStore";
import { Wall } from "./features/wall/Wall";
import { useDownloadCompletionSync } from "./features/wall/api/useDownloadCompletionSync";

type View = 'downloads' | 'wall' | 'settings';

function App() {
  const { t, i18n } = useTranslation();
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [currentView, setCurrentView] = useState<View>('downloads');
  const [newWallItems, setNewWallItems] = useState(0);

  // Global listener — alive regardless of which tab is active.
  // When a download completes and we're NOT on wall, increment the badge counter.
  const handleNewContent = useCallback(() => {
    setNewWallItems((prev) => prev + 1);
  }, []);
  useDownloadCompletionSync(handleNewContent);

  const switchView = (view: View) => {
    setCurrentView(view);
    if (view === 'wall') setNewWallItems(0); // Clear badge on enter
  };

  const { settings, fetchSettings } = useSettingsStore();

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  useEffect(() => {
    // Get theme from SettingsStore or fallback to localStorage, then default to 'dark'
    const theme = settings.theme || localStorage.getItem('vdp_theme') || 'dark';
    const root = window.document.documentElement;

    const applyTheme = (isDark: boolean) => {
      if (isDark) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    };

    if (theme === 'system') {
      const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      applyTheme(systemPrefersDark);

      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const listener = (e: MediaQueryListEvent) => applyTheme(e.matches);
      mediaQuery.addEventListener('change', listener);
      return () => mediaQuery.removeEventListener('change', listener);
    } else {
      applyTheme(theme === 'dark');
    }
  }, [settings.theme]);

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
            { id: 'downloads', icon: Download, label: t("nav.downloads") },
            { id: 'wall', icon: Images, label: t("nav.wall") },
            { id: 'settings', icon: SettingsIcon, label: t("nav.settings") },
          ].map((item) => (
            <button
              key={item.id}
              onClick={() => switchView(item.id as View)}
              className={`relative flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors ${currentView === item.id
                ? "border-brand-500 text-brand-400"
                : "border-transparent text-surface-200/60 hover:text-surface-100"
                }`}
            >
              <item.icon className="w-4 h-4" />
              {item.label}
              {item.id === 'wall' && newWallItems > 0 && currentView !== 'wall' && (
                <span className="absolute -top-0.5 right-1 flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold text-white bg-brand-500 rounded-full shadow-lg shadow-brand-500/40 animate-badge-pop">
                  {newWallItems > 99 ? '99+' : newWallItems}
                </span>
              )}
            </button>
          ))}
        </div>
      </nav>

      {/* Main Content */}
      <main className="p-6">
        {currentView === 'downloads' && (
          <>
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
          </>
        )}

        {currentView === 'wall' && (
          <Wall />
        )}

        {currentView === 'settings' && (
          <Settings />
        )}
      </main>
    </div>
  );
}

export default App;
