import { memo } from 'react';
import { Layers } from 'lucide-react';
import type { Post } from '../../../types/wall';
import { LazyThumbnail } from './LazyThumbnail';

export const PostCard = memo(
    function PostCard({ post, onClick }: { post: Post; onClick?: () => void }) {
        // Prefer the small thumbnail for wall gallery
        const thumb = post.media[0]?.thumbnail_sm_path ?? post.media[0]?.thumbnail_path;

        return (
            <article
                onClick={onClick}
                className="group relative flex flex-col gap-2 rounded-xl bg-surface-800/50 p-2 hover:bg-surface-800 transition-colors border border-transparent hover:border-surface-700 shadow-sm cursor-pointer"
            >
                <div className="relative w-full rounded-lg overflow-hidden">
                    <LazyThumbnail filePath={thumb} alt={post.title ?? 'Media'} />

                    {post.media.length > 1 && (
                        <div className="absolute top-2 right-2 bg-black/60 backdrop-blur-sm px-2 py-1 rounded-md flex items-center gap-1.5 shadow-sm border border-white/10">
                            <Layers size={14} className="text-white" />
                            <span className="text-xs font-medium text-white">{post.media.length}</span>
                        </div>
                    )}
                </div>

                <div className="px-1 pb-1">
                    <h3 className="text-sm font-medium text-surface-100 line-clamp-2 leading-snug group-hover:text-brand-400 transition-colors">
                        {post.title || 'Untitled'}
                    </h3>
                    <div className="mt-1 flex items-center gap-2">
                        {post.creator_avatar ? (
                            <img
                                src={post.creator_avatar}
                                alt={post.creator_name ?? 'Creator'}
                                className="w-5 h-5 rounded-full object-cover ring-1 ring-surface-700"
                            />
                        ) : (
                            <div className="w-5 h-5 rounded-full bg-surface-700 flex items-center justify-center text-[10px] font-bold text-surface-400">
                                {(post.creator_name ?? '?')[0].toUpperCase()}
                            </div>
                        )}
                        <span className="text-xs text-surface-400 truncate font-medium">
                            {post.creator_name}
                        </span>
                    </div>
                </div>
            </article>
        );
    },
    (prev, next) => prev.post.id === next.post.id && prev.post.status === next.post.status
);
