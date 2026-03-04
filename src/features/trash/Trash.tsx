import { useEffect, useRef, useMemo, useState } from 'react';
import { useTrashInfinite } from '../wall/api/useTrashInfinite';
import { WallGrid } from '../wall/components/WallGrid';
import { useResponsiveColumns } from '../wall/hooks/useResponsiveColumns';
import { MediaViewer } from '../wall/components/viewer/MediaViewer';
import { Loader2, Trash2 as TrashIcon } from 'lucide-react';
import type { Post } from '../../types/wall';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { ConfirmModal } from '../../components/ui/ConfirmModal';

export function Trash() {
    const queryClient = useQueryClient();

    const { data, fetchNextPage, hasNextPage, isFetchingNextPage, status } =
        useTrashInfinite();
    const observerRef = useRef<HTMLDivElement>(null);
    const columnCount = useResponsiveColumns();
    const [selectedPost, setSelectedPost] = useState<Post | null>(null);
    const [isEmptying, setIsEmptying] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);

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

    const allPosts = useMemo(() => {
        return data?.pages.flatMap((page) => page.posts) ?? [];
    }, [data]);

    const handleEmptyTrashClick = () => {
        setIsConfirmModalOpen(true);
    };

    const handleConfirmEmptyTrash = async () => {
        setIsConfirmModalOpen(false);
        setIsEmptying(true);
        setError(null);
        try {
            await invoke('empty_trash_command');
            queryClient.invalidateQueries({ queryKey: ['trash'] });
            setSelectedPost(null);
        } catch (err) {
            console.error(err);
            setError(String(err) || "Failed to empty trash.");
        } finally {
            setIsEmptying(false);
        }
    };

    if (status === 'pending') {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-surface-400">
                <Loader2 className="w-8 h-8 animate-spin mb-4 text-brand-500" />
                <p>Loading Trash...</p>
            </div>
        );
    }

    if (status === 'error') {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-red-400">
                <p>Failed to load Trash. Please try again.</p>
            </div>
        );
    }

    if (allPosts.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center h-[calc(100vh-140px)] text-surface-400">
                <div className="w-20 h-20 rounded-2xl bg-surface-800 flex items-center justify-center mb-6 border border-surface-700 shadow-xl">
                    <TrashIcon className="w-10 h-10 text-surface-500" />
                </div>
                <h2 className="text-xl font-semibold text-surface-100 mb-2">Trash is completely empty</h2>
                <p className="text-sm max-w-sm text-center">
                    Items you delete from the Wall will appear here for 30 days before being permanently removed.
                </p>
            </div>
        );
    }

    const currentIndex = selectedPost ? allPosts.findIndex(p => p.id === selectedPost.id) : -1;
    const hasNextPost = currentIndex !== -1 && currentIndex < allPosts.length - 1;
    const hasPrevPost = currentIndex > 0;

    const handleNextPost = () => {
        if (hasNextPost) {
            setSelectedPost(allPosts[currentIndex + 1]);
            if (currentIndex >= allPosts.length - 3 && hasNextPage && !isFetchingNextPage) {
                fetchNextPage();
            }
        }
    };

    const handlePrevPost = () => {
        if (hasPrevPost) {
            setSelectedPost(allPosts[currentIndex - 1]);
        }
    };

    return (
        <div className="h-[calc(100vh-140px)] flex flex-col relative">

            <div className="flex items-center justify-between mb-4 px-1">
                <h2 className="text-xl font-semibold text-surface-100 flex items-center gap-2">
                    <TrashIcon className="text-brand-400" />
                    Trash
                </h2>
                <button
                    onClick={handleEmptyTrashClick}
                    disabled={isEmptying}
                    className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-red-600/90 hover:bg-red-500 rounded-lg transition-colors disabled:opacity-50"
                >
                    {isEmptying ? <Loader2 size={16} className="animate-spin" /> : <TrashIcon size={16} />}
                    Empty Trash
                </button>
            </div>

            {error && (
                <div className="text-red-500 text-sm bg-red-500/10 border border-red-500/20 px-3 py-2 rounded-lg break-words mb-4">
                    {error}
                </div>
            )}

            <WallGrid posts={allPosts} columnCount={columnCount} gap={16} onPostClick={setSelectedPost} />

            {/* Infinite Scroll Sentinel - Placed at bottom of container */}
            <div ref={observerRef} className="h-10 w-full flex items-center justify-center text-surface-400 absolute bottom-0 left-0 pointer-events-none">
                {isFetchingNextPage && <Loader2 className="w-5 h-5 animate-spin text-brand-500" />}
            </div>

            {selectedPost && (
                <MediaViewer
                    post={selectedPost}
                    onClose={() => setSelectedPost(null)}
                    onNextPost={hasNextPost ? handleNextPost : undefined}
                    onPrevPost={hasPrevPost ? handlePrevPost : undefined}
                    isTrashMode={true}
                />
            )}

            <ConfirmModal
                isOpen={isConfirmModalOpen}
                title="Empty Trash"
                message="Are you sure you want to permanently delete all items in the trash? This action cannot be undone and files will be moved to your system's Recycle Bin."
                onConfirm={handleConfirmEmptyTrash}
                onCancel={() => setIsConfirmModalOpen(false)}
                confirmText="Empty Trash"
                isDanger={true}
            />
        </div>
    );
}
