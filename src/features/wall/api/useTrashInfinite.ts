import { useInfiniteQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { PostsPage } from '../../../types/wall';

export function useTrashInfinite(filters?: Record<string, any>) {
    return useInfiniteQuery({
        queryKey: ['trash', filters],
        queryFn: async ({ pageParam = 1 }) => {
            return await invoke<PostsPage>('get_trash_posts', {
                page: pageParam,
                limit: 50,
                ...filters,
            });
        },
        getNextPageParam: (lastPage) => {
            return lastPage.page < lastPage.total_pages ? lastPage.page + 1 : undefined;
        },
        initialPageParam: 1,
        staleTime: 5 * 60 * 1000, // 5 minutes
        gcTime: 30 * 60 * 1000, // 30 minutes
    });
}
