import React, { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from './SettingsStore';
import { Folder, Sliders, Languages, HardDrive, Trash2 } from 'lucide-react';

export const Settings = () => {
    const { t, i18n } = useTranslation();
    const { settings, fetchSettings, updateSetting, isLoading } = useSettingsStore();

    useEffect(() => {
        fetchSettings();
    }, [fetchSettings]);

    const handlePathChange = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Download Directory',
            });
            if (selected && typeof selected === 'string') {
                await updateSetting('download_path', selected);
            }
        } catch (err) {
            console.error('Failed to select directory:', err);
        }
    };

    const handleLanguageChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
        const lang = e.target.value;
        await updateSetting('language', lang);
        i18n.changeLanguage(lang);
    };

    if (isLoading) {
        return (
            <div className="flex items-center justify-center p-12">
                <div className="w-8 h-8 border-4 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
        );
    }

    return (
        <div className="max-w-3xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold">{t('nav.settings')}</h2>
            </div>

            <div className="space-y-6">
                {/* General Settings */}
                <section className="bg-surface-800 border border-surface-700 rounded-2xl overflow-hidden">
                    <div className="px-6 py-4 border-b border-surface-700 bg-surface-800/50">
                        <h3 className="text-sm font-semibold text-surface-300 uppercase tracking-wider flex items-center gap-2">
                            <Sliders size={16} />
                            General
                        </h3>
                    </div>
                    <div className="p-6 space-y-6">
                        {/* Download Path */}
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-surface-200">Download Directory</label>
                            <div className="flex gap-2">
                                <div className="flex-1 flex items-center gap-3 px-4 py-2.5 bg-surface-900 border border-surface-700 rounded-xl text-sm text-surface-400 select-none overflow-hidden">
                                    <Folder size={18} className="flex-shrink-0" />
                                    <span className="truncate">{settings.download_path || 'Default (Videos folder)'}</span>
                                </div>
                                <button
                                    onClick={handlePathChange}
                                    className="px-4 py-2.5 bg-surface-700 hover:bg-surface-600 text-white text-sm font-medium rounded-xl transition-colors whitespace-nowrap"
                                >
                                    Change
                                </button>
                            </div>
                        </div>

                        {/* Concurrent Downloads */}
                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <label className="text-sm font-medium text-surface-200">Concurrent Downloads</label>
                                <span className="text-sm font-bold text-brand-400">{settings.concurrent_downloads}</span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="10"
                                value={settings.concurrent_downloads || 3}
                                onChange={(e) => updateSetting('concurrent_downloads', e.target.value)}
                                className="w-full h-2 bg-surface-900 rounded-lg appearance-none cursor-pointer accent-brand-500"
                            />
                            <p className="text-xs text-surface-400">Number of active downloads at the same time.</p>
                        </div>

                        {/* Language */}
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-surface-200 flex items-center gap-2">
                                <Languages size={18} />
                                Language
                            </label>
                            <select
                                value={settings.language || i18n.language}
                                onChange={handleLanguageChange}
                                className="w-full px-4 py-2.5 bg-surface-900 border border-surface-700 rounded-xl text-sm focus:outline-none focus:border-brand-500"
                            >
                                <option value="en">English</option>
                                <option value="es">Español</option>
                            </select>
                        </div>
                    </div>
                </section>

                {/* Advanced Settings */}
                <section className="bg-surface-800 border border-surface-700 rounded-2xl overflow-hidden">
                    <div className="px-6 py-4 border-b border-surface-700 bg-surface-800/50">
                        <h3 className="text-sm font-semibold text-surface-300 uppercase tracking-wider flex items-center gap-2">
                            <HardDrive size={16} />
                            Advanced & Storage
                        </h3>
                    </div>
                    <div className="p-6 space-y-6">

                        {/* Trash Auto Clean */}
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-surface-200 flex items-center gap-2">
                                <Trash2 size={18} />
                                Trash Auto-Clean
                            </label>
                            <select
                                value={settings.trash_auto_clean_days || '30'}
                                onChange={(e) => updateSetting('trash_auto_clean_days', e.target.value)}
                                className="w-full px-4 py-2.5 bg-surface-900 border border-surface-700 rounded-xl text-sm focus:outline-none focus:border-brand-500"
                            >
                                <option value="0">Never</option>
                                <option value="7">After 7 days</option>
                                <option value="30">After 30 days</option>
                                <option value="90">After 90 days</option>
                            </select>
                        </div>

                        {/* yt-dlp Version */}
                        <div className="pt-4 border-t border-surface-700">
                            <div className="flex items-center justify-between">
                                <div>
                                    <div>
                                        <h4 className="text-sm font-medium text-surface-200 text-left">yt-dlp Engine</h4>
                                        <p className="text-xs text-surface-400 text-left mt-1">
                                            Manage yt-dlp binary updates.
                                        </p>
                                    </div>
                                </div>
                                <button
                                    className="px-4 py-2 bg-surface-900 hover:bg-surface-700 border border-surface-700 rounded-xl text-sm font-medium transition-colors"
                                >
                                    Check for Updates
                                </button>
                            </div>
                        </div>

                    </div>
                </section>
            </div>
        </div>
    );
};
