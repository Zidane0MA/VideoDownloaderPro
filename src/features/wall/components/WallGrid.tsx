import React from 'react';
import { VirtuosoMasonry } from '@virtuoso.dev/masonry';
import type { Post } from '../../../types/wall';
import { PostCard } from './PostCard';

interface WallGridProps {
    posts: Post[];
    columnCount: number;
    gap?: number;
    onPostClick?: (post: Post) => void;
}

/**
 * Virtualized Masonry Grid using @virtuoso.dev/masonry.
 *
 * - Only renders visible items (scales to 20k+)
 * - Auto-measures variable item heights
 * - Shortest-column-first distribution (true masonry)
 * - Dynamic column count (responsive via useResponsiveColumns)
 */
export function WallGrid({ posts, columnCount, onPostClick }: WallGridProps) {
    return (
        <VirtuosoMasonry
            columnCount={columnCount}
            data={posts}
            context={{ onPostClick }}
            style={{ height: '100%' }}
            initialItemCount={Math.min(posts.length, 20)}
            ItemContent={ItemContent}
        />
    );
}

const ItemContent: React.FC<{ data: Post; context?: { onPostClick?: (p: Post) => void } }> = ({ data, context }) => {
    return (
        <div style={{ padding: '6px' }}>
            <PostCard post={data} onClick={() => context?.onPostClick?.(data)} />
        </div>
    );
};
