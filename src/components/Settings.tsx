import React, { useState } from 'react';
import { Moon, Sun, Monitor, Info, Users } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { AccountCard } from './settings/AccountCard';
import { PLATFORMS } from '../types/auth';
import { useAuthStatus } from '../hooks/useAuth';

export const Settings: React.FC = () => {
  const { t } = useTranslation();
  const { data: authStatus } = useAuthStatus();
  // Placeholder for theme state - in a real app this would come from a theme store/context
  const [theme, setTheme] = useState<'dark' | 'light' | 'system'>('dark'); 

  return (
    <div className="max-w-3xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div>
        <h2 className="text-2xl font-bold text-white mb-2">{t('nav.settings')}</h2>
        <p className="text-surface-400">Manage your application preferences and download settings.</p>
      </div>

      <div className="space-y-6">
        {/* Accounts Section */}
        <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
          <div className="px-6 py-4 border-b border-surface-700">
            <h3 className="font-semibold text-white flex items-center gap-2">
              <Users className="w-4 h-4 text-brand-400" />
              {t('settings.accounts', 'Accounts')}
            </h3>
          </div>
          <div className="p-6 space-y-4">
            <p className="text-sm text-surface-400">
              Manage your connected accounts for restricted content.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {PLATFORMS.map((platform) => {
                const session = authStatus?.find((s) => s.platform_id === platform.id);
                return (
                  <AccountCard
                    key={platform.id}
                    platformId={platform.id}
                    name={platform.name}
                    session={session}
                  />
                );
              })}
            </div>
          </div>
        </section>

        {/* Appearance Section */}
        <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
          <div className="px-6 py-4 border-b border-surface-700">
            <h3 className="font-semibold text-white flex items-center gap-2">
              <Monitor className="w-4 h-4 text-brand-400" />
              Appearance
            </h3>
          </div>
          <div className="p-6 space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-surface-200 font-medium">Theme</p>
                <p className="text-sm text-surface-400">Customize the application look and feel</p>
              </div>
              <div className="flex bg-surface-900 rounded-lg p-1 border border-surface-700/50">
                <button
                  onClick={() => setTheme('dark')}
                  className={`p-2 rounded-md transition-all ${theme === 'dark' ? 'bg-surface-700 text-white shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                >
                  <Moon size={18} />
                </button>
                <button
                  onClick={() => setTheme('light')}
                  className={`p-2 rounded-md transition-all ${theme === 'light' ? 'bg-surface-700 text-white shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                >
                  <Sun size={18} />
                </button>
                <button
                  onClick={() => setTheme('system')}
                  className={`p-2 rounded-md transition-all ${theme === 'system' ? 'bg-surface-700 text-white shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                >
                  <Monitor size={18} />
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* About Section */}
        <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
             <div className="px-6 py-4 border-b border-surface-700">
            <h3 className="font-semibold text-white flex items-center gap-2">
              <Info className="w-4 h-4 text-surface-400" />
              About
            </h3>
          </div>
          <div className="p-6">
             <div className="flex items-center justify-between">
                 <div>
                    <p className="text-surface-200 font-medium">Video Downloader Pro</p>
                    <p className="text-sm text-surface-400">Version 0.1.0-alpha</p>
                 </div>
                 <div className="text-right">
                    <p className="text-xs text-surface-500">Built with Tauri v2 & React</p>
                 </div>
             </div>
          </div>
        </section>
      </div>
    </div>
  );
};
