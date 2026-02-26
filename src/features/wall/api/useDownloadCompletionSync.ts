import { useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

/**
 * Global hook that listens for `download-completed` events from the backend
 * and invalidates the Wall's `['posts']` cache so new content appears immediately.
 *
 * Must be mounted at the App root (not inside Wall) because Wall unmounts
 * when the user switches tabs, which would tear down the listener.
 *
 * @param onNewContent - Optional callback fired when a download completes.
 *                       Used by App.tsx to show a "new content" badge on the Wall tab.
 */
export function useDownloadCompletionSync(onNewContent?: () => void) {
    const qc = useQueryClient();

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<string>('download-completed', () => {
                // Mark all posts queries as stale → triggers refetch when Wall mounts
                qc.invalidateQueries({ queryKey: ['posts'] });
                onNewContent?.();
            });
            return unlisten;
        };

        const unlistenPromise = setupListener();

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, [qc, onNewContent]);
}
