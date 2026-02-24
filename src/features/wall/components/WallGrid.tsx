import { useRef, useMemo, useState, useEffect } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { Post } from '../../../types/wall';
import { PostCard } from './PostCard';

interface WallGridProps {
    posts: Post[];
    columnCount: number;
    gap?: number;
}

export function WallGrid({ posts, columnCount, gap = 16 }: WallGridProps) {
    const parentRef = useRef<HTMLDivElement>(null);

    // Distribute posts into columns for masonry layout
    const columns = useMemo(() => {
        const cols: Post[][] = Array.from({ length: columnCount }, () => []);
        const colHeights = new Array(columnCount).fill(0);

        posts.forEach((post) => {
            // Find shortest column
            const shortestCol = colHeights.indexOf(Math.min(...colHeights));
            cols[shortestCol].push(post);

            // Estimate height based on aspect ratio (default 16:9 if unknown)
            const media = post.media[0];
            const ar = media?.width && media?.height ? media.height / media.width : 0.56; // 16:9 is 0.56, 9:16 is 1.77
            // 250px base width estimation + 80px for info area
            colHeights[shortestCol] += ar * 250 + 80;
        });

        return cols;
    }, [posts, columnCount]);

    return (
        <div
            ref={parentRef}
            style={{
                display: 'grid',
                gridTemplateColumns: `repeat(${columnCount}, minmax(0, 1fr))`,
                gap: `${gap}px`,
                height: '100%',
                overflowY: 'auto',
                overflowX: 'hidden',
                paddingRight: '4px',
            }}
            className="custom-scrollbar"
        >
            {columns.map((colPosts, i) => (
                <VirtualColumn key={i} posts={colPosts} parentRef={parentRef} gap={gap} />
            ))}
        </div>
    );
}

function VirtualColumn({
    posts,
    parentRef,
    gap,
}: {
    posts: Post[];
    parentRef: React.RefObject<HTMLDivElement | null>;
    gap: number;
}) {
    const virtualizer = useVirtualizer({
        count: posts.length,
        getScrollElement: () => parentRef.current as HTMLDivElement,
        estimateSize: (i) => {
            const media = posts[i].media[0];
            const ar = media?.width && media?.height ? media.height / media.width : 0.56;
            // Rough estimation: assuming column width is ~300px on average
            return ar * 300 + 100 + gap; // info height + gap
        },
        overscan: 5,
    });

    return (
        <div
            style={{
                height: virtualizer.getTotalSize(),
                position: 'relative',
                width: '100%',
            }}
        >
            {virtualizer.getVirtualItems().map((vi) => (
                <div
                    key={posts[vi.index].id}
                    ref={virtualizer.measureElement}
                    data-index={vi.index}
                    style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        width: '100%',
                        transform: `translateY(${vi.start}px)`,
                        paddingBottom: `${gap}px`,
                    }}
                >
                    <PostCard post={posts[vi.index]} />
                </div>
            ))}
        </div>
    );
}

// Hook to responsively determine columns based on window width
export function useResponsiveColumns(): number {
    const [cols, setCols] = useState(getColumns(window.innerWidth));

    useEffect(() => {
        let t: ReturnType<typeof setTimeout>;
        const handler = () => {
            clearTimeout(t);
            t = setTimeout(() => setCols(getColumns(window.innerWidth)), 150);
        };

        window.addEventListener('resize', handler);
        return () => {
            window.removeEventListener('resize', handler);
            clearTimeout(t);
        };
    }, []);

    return cols;
}

function getColumns(w: number) {
    if (w < 600) return 2;
    if (w < 900) return 3;
    if (w < 1200) return 4;
    if (w < 1600) return 5;
    return 6;
}
