import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from './SettingsStore';
import { Folder, Sliders, Languages, HardDrive, Trash2, Moon, Sun, Monitor, Info, Users, RefreshCw, Plus } from 'lucide-react';
import { AccountCard } from '../../components/settings/AccountCard';
import { PlatformPickerModal } from '../../components/settings/PlatformPickerModal';
import { ConnectAccountModal } from '../../components/settings/ConnectAccountModal';
import { PLATFORMS } from '../../types/auth';
import { useAuthStatus, useVerifyAllSessions } from '../../hooks/useAuth';

export const Settings = () => {
    const { t, i18n } = useTranslation();
    const { settings, fetchSettings, updateSetting, isLoading } = useSettingsStore();

    const { data: authStatus } = useAuthStatus();
    const verifyAll = useVerifyAllSessions();
    const theme = settings.theme || 'dark';
    const [showPicker, setShowPicker] = useState(false);
    const [selectedPlatformId, setSelectedPlatformId] = useState<string | null>(null);
    const [isUpdatingYtDlp, setIsUpdatingYtDlp] = useState(false);
    const [updateMessage, setUpdateMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);
    const [ytdlpVersion, setYtdlpVersion] = useState<string | null>(null);

    const activeSessions = authStatus?.filter(s => s.status !== 'NONE') || [];
    const connectedPlatforms = PLATFORMS.filter(p => activeSessions.some(s => s.platform_id === p.id));
    const selectedPlatform = PLATFORMS.find(p => p.id === selectedPlatformId);

    useEffect(() => {
        fetchSettings();
        invoke<string>('get_ytdlp_version')
            .then(setYtdlpVersion)
            .catch(() => setYtdlpVersion(null));
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

    const handleUpdateYtDlp = async () => {
        try {
            setIsUpdatingYtDlp(true);
            setUpdateMessage(null);

            // Check for active downloads — yt-dlp binary can't be replaced while in use
            const queue = await invoke<{ is_paused: boolean; tasks: { status: string }[] }>('get_queue_status');
            const activeStatuses = ['DOWNLOADING', 'PROCESSING', 'FETCHING_META'];
            const hasActive = queue.tasks.some(t => activeStatuses.includes(t.status));
            if (hasActive) {
                setUpdateMessage({ text: 'Cannot update while downloads are active. Wait for them to finish.', type: 'error' });
                return;
            }

            const oldVersion = ytdlpVersion;
            const newVersion = await invoke<string>('update_ytdlp');
            setYtdlpVersion(newVersion);

            if (oldVersion && oldVersion === newVersion) {
                setUpdateMessage({ text: `Already up to date (v${newVersion})`, type: 'success' });
            } else {
                setUpdateMessage({ text: oldVersion ? `Updated from v${oldVersion} → v${newVersion}` : `Updated to v${newVersion}`, type: 'success' });
            }
        } catch (error) {
            setUpdateMessage({ text: `Update failed: ${error}`, type: 'error' });
        } finally {
            setIsUpdatingYtDlp(false);
        }
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
                                    className="px-4 py-2.5 bg-surface-900 hover:bg-surface-700 border border-surface-700 text-surface-200 hover:text-surface-100 text-sm font-medium rounded-xl transition-colors whitespace-nowrap"
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

                        {/* Rate Limit */}
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-surface-200">Rate Limit</label>
                            <select
                                value={settings.rate_limit || ""}
                                onChange={(e) => updateSetting('rate_limit', e.target.value)}
                                className="w-full px-4 py-2.5 bg-surface-900 border border-surface-700 rounded-xl text-sm focus:outline-none focus:border-brand-500"
                            >
                                <option value="">Unlimited</option>
                                <option value="1M">1 MB/s</option>
                                <option value="5M">5 MB/s</option>
                                <option value="10M">10 MB/s</option>
                                <option value="25M">25 MB/s</option>
                                <option value="50M">50 MB/s</option>
                            </select>
                            <p className="text-xs text-surface-400">Maximum download speed per active task.</p>
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
                                        <h4 className="text-sm font-medium text-surface-200 text-left">
                                            yt-dlp Engine
                                            {ytdlpVersion && <span className="ml-2 text-xs font-mono text-brand-400">v{ytdlpVersion}</span>}
                                        </h4>
                                        <p className="text-xs text-surface-400 text-left mt-1">
                                            Manage yt-dlp binary updates.
                                        </p>
                                    </div>
                                </div>
                                <button
                                    onClick={handleUpdateYtDlp}
                                    disabled={isUpdatingYtDlp}
                                    className="px-4 py-2 bg-surface-900 hover:bg-surface-700 border border-surface-700 rounded-xl text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                >
                                    {isUpdatingYtDlp && <RefreshCw size={14} className="animate-spin" />}
                                    {isUpdatingYtDlp ? 'Updating...' : 'Check for Updates'}
                                </button>
                            </div>
                            {updateMessage && (
                                <div className={`mt-3 p-3 rounded-xl text-sm ${updateMessage.type === 'success' ? 'bg-green-500/10 text-green-400 border border-green-500/20' : 'bg-red-500/10 text-red-400 border border-red-500/20'}`}>
                                    {updateMessage.text}
                                </div>
                            )}
                        </div>

                    </div>
                </section>

                {/* Accounts Section */}
                <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
                    <div className="px-6 py-4 border-b border-surface-700 flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-surface-800/50">
                        <h3 className="text-sm font-semibold text-surface-300 uppercase tracking-wider flex items-center gap-2">
                            <Users size={16} />
                            {t('settings.accounts', 'Accounts')}
                        </h3>
                        <div className="flex items-center gap-2">
                            <button
                                onClick={() => setShowPicker(true)}
                                className="px-3 py-1.5 text-xs bg-brand-600 hover:bg-brand-500 text-white rounded-lg flex items-center gap-1.5 transition-colors"
                            >
                                <Plus size={14} /> Add Account
                            </button>
                            <button
                                onClick={() => verifyAll.mutate()}
                                disabled={verifyAll.isPending}
                                className="px-3 py-1.5 text-xs bg-surface-700 hover:bg-surface-600 text-surface-200 rounded-lg flex items-center gap-2 transition-colors border border-surface-600 disabled:opacity-50"
                            >
                                <RefreshCw size={14} className={verifyAll.isPending ? 'animate-spin' : ''} />
                                {verifyAll.isPending ? 'Verifying...' : 'Verify All'}
                            </button>
                        </div>
                    </div>
                    <div className="p-6 space-y-4">
                        <p className="text-sm text-surface-400">
                            Manage your connected accounts for restricted content.
                        </p>
                        {connectedPlatforms.length === 0 ? (
                            <div className="text-center py-8 text-surface-500 border-2 border-dashed border-surface-700 rounded-xl">
                                No accounts connected yet. Click "Add Account" to get started.
                            </div>
                        ) : (
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {connectedPlatforms.map((platform) => {
                                    const session = activeSessions.find((s) => s.platform_id === platform.id);
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
                        )}
                    </div>
                </section>

                {showPicker && (
                    <PlatformPickerModal
                        onClose={() => setShowPicker(false)}
                        onSelect={(id) => {
                            setShowPicker(false);
                            setSelectedPlatformId(id);
                        }}
                    />
                )}

                {selectedPlatformId && selectedPlatform && (
                    <ConnectAccountModal
                        platformId={selectedPlatformId}
                        platformName={selectedPlatform.name}
                        onClose={() => setSelectedPlatformId(null)}
                    />
                )}

                {/* Appearance Section */}
                <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
                    <div className="px-6 py-4 border-b border-surface-700 bg-surface-800/50">
                        <h3 className="text-sm font-semibold text-surface-300 uppercase tracking-wider flex items-center gap-2">
                            <Monitor size={16} />
                            Appearance
                        </h3>
                    </div>
                    <div className="p-6 space-y-4">
                        <div className="flex items-center justify-between">
                            <div>
                                <p className="text-sm font-medium text-surface-200">Theme</p>
                                <p className="text-xs text-surface-400">Customize the application look and feel</p>
                            </div>
                            <div className="flex bg-surface-900 rounded-lg p-1 border border-surface-700/50">
                                <button
                                    onClick={() => updateSetting('theme', 'dark')}
                                    className={`p-2 rounded-md transition-all ${theme === 'dark' ? 'bg-surface-700 text-surface-100 shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                                >
                                    <Moon size={18} />
                                </button>
                                <button
                                    onClick={() => updateSetting('theme', 'light')}
                                    className={`p-2 rounded-md transition-all ${theme === 'light' ? 'bg-surface-700 text-surface-100 shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                                >
                                    <Sun size={18} />
                                </button>
                                <button
                                    onClick={() => updateSetting('theme', 'system')}
                                    className={`p-2 rounded-md transition-all ${theme === 'system' ? 'bg-surface-700 text-surface-100 shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}
                                >
                                    <Monitor size={18} />
                                </button>
                            </div>
                        </div>
                    </div>
                </section>

                {/* About Section */}
                <section className="bg-surface-800 border border-surface-700 rounded-xl overflow-hidden">
                    <div className="px-6 py-4 border-b border-surface-700 bg-surface-800/50">
                        <h3 className="text-sm font-semibold text-surface-300 uppercase tracking-wider flex items-center gap-2">
                            <Info size={16} />
                            About
                        </h3>
                    </div>
                    <div className="p-6">
                        <div className="flex items-center justify-between">
                            <div>
                                <p className="text-sm font-medium text-surface-200">Video Downloader Pro</p>
                                <p className="text-xs text-surface-400">Version 0.1.0-alpha</p>
                            </div>
                            <div className="text-right">
                                <p className="text-xs text-brand-500">Built with Tauri v2 & React</p>
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    );
};
