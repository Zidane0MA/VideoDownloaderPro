import { convertFileSrc } from '@tauri-apps/api/core';
import { useState, useRef, useEffect, memo } from 'react';
import { Image as ImageIcon } from 'lucide-react';

export const LazyThumbnail = memo(function LazyThumbnail({
    filePath,
    alt,
}: {
    filePath: string | null;
    alt: string;
}) {
    const [isVisible, setIsVisible] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);
    const [hasError, setHasError] = useState(false);
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const obs = new IntersectionObserver(
            ([e]) => {
                if (e.isIntersecting) {
                    setIsVisible(true);
                    obs.disconnect();
                }
            },
            { rootMargin: '200px', threshold: 0 }
        );
        if (ref.current) obs.observe(ref.current);
        return () => obs.disconnect();
    }, []);

    const src = filePath ? convertFileSrc(filePath) : null;

    return (
        <div ref={ref} className="relative w-full aspect-video bg-surface-800 rounded-lg overflow-hidden flex items-center justify-center">
            {isVisible && src && !hasError ? (
                <img
                    src={src}
                    alt={alt}
                    loading="lazy"
                    onLoad={() => setIsLoaded(true)}
                    onError={() => setHasError(true)}
                    className={`absolute top-0 left-0 w-full h-full object-cover transition-opacity duration-300 ${isLoaded ? 'opacity-100' : 'opacity-0'
                        }`}
                />
            ) : (
                <div className="text-surface-600 flex flex-col items-center justify-center">
                    {hasError ? (
                        <span className="text-xs text-brand-400">Error loading thumbnail</span>
                    ) : (
                        <ImageIcon size={24} className="opacity-50" />
                    )}
                </div>
            )}
        </div>
    );
});
