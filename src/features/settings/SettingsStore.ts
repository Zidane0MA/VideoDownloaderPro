import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface SettingsState {
    settings: Record<string, string>;
    isLoading: boolean;
    error: string | null;
    fetchSettings: () => Promise<void>;
    updateSetting: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
    settings: {},
    isLoading: false,
    error: null,

    fetchSettings: async () => {
        set({ isLoading: true, error: null });
        try {
            const dbSettings = await invoke<Record<string, string>>('get_settings');

            // Merge with some defaults if not present
            const defaultSettings: Record<string, string> = {
                download_path: '', // Handled by backend/Rust default if empty, but we can store it here
                concurrent_downloads: '3',
                language: 'en',
                trash_auto_clean_days: '30',
            };

            set({
                settings: { ...defaultSettings, ...dbSettings },
                isLoading: false
            });
        } catch (err: any) {
            set({ error: err.toString(), isLoading: false });
        }
    },

    updateSetting: async (key: string, value: string) => {
        try {
            await invoke('update_setting', { key, value });
            set((state) => ({
                settings: {
                    ...state.settings,
                    [key]: value,
                },
            }));
        } catch (err: any) {
            set({ error: err.toString() });
            throw err;
        }
    },
}));
