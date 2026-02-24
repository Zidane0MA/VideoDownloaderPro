---
name: react-virtual-performance
description: "High-performance React gallery patterns: @tanstack/react-virtual, Masonry Grid, lazy thumbnail loading via Tauri convertFileSrc, memoized components, and infinite scroll. Use when building the Wall/Gallery view in VideoDownloaderPro."
---

# React Virtual Performance Patterns

Patterns for building high-performance media galleries in React + Tauri. Covers virtualization, Masonry layout, lazy loading, and memoization for thousands of items.

## When to Use

- Building the "Wall of Content" gallery view
- Rendering 1000+ thumbnails performantly
- Implementing Masonry Grid (Pinterest-style) layouts
- Loading local files via `convertFileSrc`

---

## Pattern 1: Virtualized Masonry Grid

```bash
npm install @tanstack/react-virtual
```

```tsx
import { useRef, useMemo } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { Post } from '../types';

interface WallGridProps {
  posts: Post[];
  columnCount: number;
  gap: number;
}

export function WallGrid({ posts, columnCount, gap }: WallGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);

  const columns = useMemo(() => {
    const cols: Post[][] = Array.from({ length: columnCount }, () => []);
    const colHeights = new Array(columnCount).fill(0);
    posts.forEach((post) => {
      const shortestCol = colHeights.indexOf(Math.min(...colHeights));
      cols[shortestCol].push(post);
      const ar = post.media[0]
        ? (post.media[0].height ?? 300) / (post.media[0].width ?? 200)
        : 1.5;
      colHeights[shortestCol] += ar * 200 + 80;
    });
    return cols;
  }, [posts, columnCount]);

  return (
    <div ref={parentRef} style={{
      display: 'grid',
      gridTemplateColumns: `repeat(${columnCount}, 1fr)`,
      gap: `${gap}px`,
      height: '100%',
      overflow: 'auto',
    }}>
      {columns.map((colPosts, i) => (
        <VirtualColumn key={i} posts={colPosts} parentRef={parentRef} />
      ))}
    </div>
  );
}

function VirtualColumn({ posts, parentRef }: {
  posts: Post[];
  parentRef: React.RefObject<HTMLDivElement>;
}) {
  const virtualizer = useVirtualizer({
    count: posts.length,
    getScrollElement: () => parentRef.current,
    estimateSize: (i) => {
      const m = posts[i].media[0];
      return m?.width && m.height ? (m.height / m.width) * 250 + 80 : 350;
    },
    overscan: 5,
  });

  return (
    <div style={{ height: virtualizer.getTotalSize(), position: 'relative', width: '100%' }}>
      {virtualizer.getVirtualItems().map((vi) => (
        <div key={posts[vi.index].id}
          ref={virtualizer.measureElement}
          data-index={vi.index}
          style={{
            position: 'absolute', top: 0, left: 0, width: '100%',
            transform: `translateY(${vi.start}px)`,
          }}>
          <PostCard post={posts[vi.index]} />
        </div>
      ))}
    </div>
  );
}
```

---

## Pattern 2: Lazy Thumbnail via `convertFileSrc`

```tsx
import { convertFileSrc } from '@tauri-apps/api/core';
import { useState, useRef, useEffect, memo } from 'react';

export const LazyThumbnail = memo(function LazyThumbnail({
  filePath, alt,
}: { filePath: string | null; alt: string }) {
  const [isVisible, setIsVisible] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const obs = new IntersectionObserver(
      ([e]) => { if (e.isIntersecting) { setIsVisible(true); obs.disconnect(); } },
      { rootMargin: '200px', threshold: 0 }
    );
    if (ref.current) obs.observe(ref.current);
    return () => obs.disconnect();
  }, []);

  const src = filePath ? convertFileSrc(filePath) : null;

  return (
    <div ref={ref} className="thumbnail-container">
      {isVisible && src && !hasError ? (
        <img src={src} alt={alt} loading="lazy"
          onLoad={() => setIsLoaded(true)}
          onError={() => setHasError(true)}
          className={`thumbnail-img ${isLoaded ? 'loaded' : 'loading'}`} />
      ) : (
        <div className="thumbnail-placeholder">{hasError ? '⚠️' : ''}</div>
      )}
    </div>
  );
});
```

**CSS:**
```css
.thumbnail-img { width: 100%; opacity: 0; transition: opacity 0.3s ease; }
.thumbnail-img.loaded { opacity: 1; }
.thumbnail-placeholder {
  width: 100%; aspect-ratio: 16/9;
  background: var(--color-surface-secondary);
}
```

> Always use `thumb_sm.jpg` (300px) for the Wall, not the original thumbnail.

---

## Pattern 3: Memoized PostCard

```tsx
export const PostCard = memo(function PostCard({ post }: { post: Post }) {
  const thumb = post.media[0]?.thumbnailSmPath ?? post.media[0]?.thumbnailPath;
  return (
    <article className="post-card" id={`post-${post.id}`}>
      <LazyThumbnail filePath={thumb} alt={post.title ?? 'Media'} />
      <div className="post-card-info">
        <h3>{post.title ?? 'Untitled'}</h3>
        <span>{post.creatorName}</span>
        {post.media.length > 1 && <span>{post.media.length} items</span>}
      </div>
    </article>
  );
}, (prev, next) => prev.post.id === next.post.id && prev.post.title === next.post.title);
```

---

## Pattern 4: Infinite Scroll with React Query

```tsx
import { useInfiniteQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function usePostsInfinite(filters: Record<string, any>) {
  return useInfiniteQuery({
    queryKey: ['posts', filters],
    queryFn: ({ pageParam = 1 }) =>
      invoke<PostsPage>('get_posts', { page: pageParam, limit: 50, ...filters }),
    getNextPageParam: (last) =>
      last.page < last.totalPages ? last.page + 1 : undefined,
    initialPageParam: 1,
    staleTime: 5 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
  });
}
```

Trigger next page with Intersection Observer on a sentinel `<div>` with `rootMargin: '500px'`.

---

## Pattern 5: Responsive Columns

```tsx
import { useState, useEffect } from 'react';

export function useResponsiveColumns(): number {
  const [cols, setCols] = useState(getColumns(window.innerWidth));
  useEffect(() => {
    let t: ReturnType<typeof setTimeout>;
    const h = () => { clearTimeout(t); t = setTimeout(() => setCols(getColumns(window.innerWidth)), 150); };
    window.addEventListener('resize', h);
    return () => { window.removeEventListener('resize', h); clearTimeout(t); };
  }, []);
  return cols;
}

function getColumns(w: number) {
  if (w < 600) return 2; if (w < 900) return 3;
  if (w < 1200) return 4; if (w < 1600) return 5; return 6;
}
```

---

## Pattern 6: Download Completion → Gallery Sync

```tsx
import { useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

export function useDownloadCompletionSync() {
  const qc = useQueryClient();
  useEffect(() => {
    const unlisten = listen<{ newStatus: string }>('download-status-changed', (e) => {
      if (e.payload.newStatus === 'COMPLETED') {
        qc.invalidateQueries({ queryKey: ['posts'] });
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [qc]);
}
```

---

## Performance Checklist

- [ ] Virtualize gallery — only visible cards in DOM
- [ ] `memo()` on PostCard with custom comparator
- [ ] Use 300px thumbnails (`thumb_sm.jpg`), never originals
- [ ] `overscan: 5` for smooth scrolling
- [ ] React Query pagination: `staleTime` 5min, `gcTime` 30min
- [ ] Intersection Observer lazy loading with 200px `rootMargin`
- [ ] Debounced responsive columns (150ms)

## Dependencies

```json
{
  "@tanstack/react-virtual": "^3.0.0",
  "@tanstack/react-query": "^5.0.0",
  "@tauri-apps/api": "^2.0.0"
}
```
