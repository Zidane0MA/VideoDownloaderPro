import { useEffect, useRef, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { usePostsInfinite } from './api/usePostsInfinite';
import { useDownloadCompletionSync } from './api/useDownloadCompletionSync';
import { WallGrid, useResponsiveColumns } from './components/WallGrid';
import { Loader2, Image as ImageIcon } from 'lucide-react';

export function Wall() {
    const { t } = useTranslation();
    useDownloadCompletionSync();
    const { data, fetchNextPage, hasNextPage, isFetchingNextPage, status } =
        usePostsInfinite();
    const observerRef = useRef<HTMLDivElement>(null);
    const columnCount = useResponsiveColumns();

    useEffect(() => {
        const obs = new IntersectionObserver(
            ([entry]) => {
                if (entry.isIntersecting && hasNextPage && !isFetchingNextPage) {
                    fetchNextPage();
                }
            },
            { rootMargin: '500px' }
        );

        if (observerRef.current) {
            obs.observe(observerRef.current);
        }

        return () => obs.disconnect();
    }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

    // Flatten infinite query pages into a single array
    const allPosts = useMemo(() => {
        return data?.pages.flatMap((page) => page.posts) ?? [];
    }, [data]);

    if (status === 'pending') {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-surface-400">
                <Loader2 className="w-8 h-8 animate-spin mb-4 text-brand-500" />
                <p>Loading Wall...</p>
            </div>
        );
    }

    if (status === 'error') {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-red-400">
                <p>Failed to load Wall. Please try again.</p>
            </div>
        );
    }

    if (allPosts.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-surface-400">
                <div className="w-20 h-20 rounded-2xl bg-surface-800 flex items-center justify-center mb-6 border border-surface-700 shadow-xl">
                    <ImageIcon className="w-10 h-10 text-surface-500" />
                </div>
                <h2 className="text-xl font-semibold text-surface-100 mb-2">{t("app.name")} Gallery</h2>
                <p className="text-sm max-w-sm text-center">
                    Downloads will appear here once they are completed. Go to the Downloads tab to add some.
                </p>
            </div>
        );
    }

    return (
        <div className="h-[calc(100vh-140px)] flex flex-col">
            <div className="flex-1 overflow-hidden relative">
                <WallGrid posts={allPosts} columnCount={columnCount} gap={16} />

                {/* Infinite Scroll Sentinel - Placed at bottom of container */}
                <div ref={observerRef} className="h-10 w-full flex items-center justify-center text-surface-400 absolute bottom-0 left-0 pointer-events-none">
                    {isFetchingNextPage && <Loader2 className="w-5 h-5 animate-spin text-brand-500" />}
                </div>
            </div>
        </div>
    );
}
