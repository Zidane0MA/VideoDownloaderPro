import { useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

export function useDownloadCompletionSync() {
    const qc = useQueryClient();

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<string>('download-completed', () => {
                // Invalidate posts query to trigger a refetch of the wall when a download completes
                qc.invalidateQueries({ queryKey: ['posts'] });
            });
            return unlisten;
        };

        const unlistenPromise = setupListener();

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, [qc]);
}
