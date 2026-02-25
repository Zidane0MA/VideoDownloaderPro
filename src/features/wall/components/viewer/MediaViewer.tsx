import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import type { Post } from '../../../../types/wall';
import { MediaPlayer } from './MediaPlayer';
import { MediaSidebar } from './MediaSidebar';
import { X, ChevronLeft, ChevronRight, ArrowLeft, ArrowRight } from 'lucide-react';

interface MediaViewerProps {
    post: Post;
    onClose: () => void;
    onNextPost?: () => void;
    onPrevPost?: () => void;
}

export function MediaViewer({ post, onClose, onNextPost, onPrevPost }: MediaViewerProps) {
    const [currentIndex, setCurrentIndex] = useState(0);

    // Filter out missing/invalid media, but trust what we got from the backend
    const mediaList = post.media || [];
    const currentMedia = mediaList[currentIndex];

    // Handle keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            } else if (e.key === 'ArrowLeft') {
                if (mediaList.length > 1) {
                    handlePrev();
                } else if (onPrevPost) {
                    onPrevPost();
                }
            } else if (e.key === 'ArrowRight') {
                if (mediaList.length > 1) {
                    handleNext();
                } else if (onNextPost) {
                    onNextPost();
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [currentIndex, mediaList.length, onPrevPost, onNextPost]); // Re-bind when index changes 

    const handlePrev = () => {
        if (mediaList.length <= 1) return;
        setCurrentIndex((prev) => (prev > 0 ? prev - 1 : mediaList.length - 1));
    };

    const handleNext = () => {
        if (mediaList.length <= 1) return;
        setCurrentIndex((prev) => (prev < mediaList.length - 1 ? prev + 1 : 0));
    };

    if (!currentMedia) {
        return createPortal(
            <div className="fixed inset-0 z-[9999] bg-black/95 flex items-center justify-center">
                <p className="text-surface-400">No media available.</p>
                <button onClick={onClose} className="absolute top-4 right-4 p-2 bg-surface-800/50 hover:bg-surface-800 text-white rounded-full transition-colors">
                    <X size={24} />
                </button>
            </div>,
            document.body
        );
    }

    return createPortal(
        <div className="fixed inset-0 z-[9999] bg-black/95 flex flex-col md:flex-row backdrop-blur-sm animate-in fade-in duration-200">
            {/* Close Button Desktop */}
            <button
                onClick={onClose}
                className="absolute top-4 right-4 z-50 p-2 bg-black/50 hover:bg-black text-white rounded-full transition-colors opacity-80 hover:opacity-100"
                aria-label="Close viewer"
            >
                <X size={24} />
            </button>

            {/* Main Player Area Left */}
            <div className="relative flex-1 flex items-center justify-center min-h-[50vh] bg-black group/nav">
                <MediaPlayer media={currentMedia} />

                {/* Post Navigation (Edge hot areas) */}
                {onPrevPost && (
                    <button
                        onClick={(e) => { e.stopPropagation(); onPrevPost(); }}
                        className="absolute left-0 top-0 bottom-0 w-24 flex items-center justify-start pl-4 opacity-0 hover:opacity-100 group-hover/nav:opacity-100 transition-opacity bg-gradient-to-r from-black/50 to-transparent text-white"
                        aria-label="Previous post"
                    >
                        <div className="p-3 rounded-full bg-black/50 hover:bg-brand-500 hover:scale-110 transition-all">
                            <ArrowLeft size={32} />
                        </div>
                    </button>
                )}

                {onNextPost && (
                    <button
                        onClick={(e) => { e.stopPropagation(); onNextPost(); }}
                        className="absolute right-0 top-0 bottom-0 w-24 flex items-center justify-end pr-4 opacity-0 hover:opacity-100 group-hover/nav:opacity-100 transition-opacity bg-gradient-to-l from-black/50 to-transparent text-white"
                        aria-label="Next post"
                    >
                        <div className="p-3 rounded-full bg-black/50 hover:bg-brand-500 hover:scale-110 transition-all">
                            <ArrowRight size={32} />
                        </div>
                    </button>
                )}

                {/* Gallery Navigation Controls */}
                {mediaList.length > 1 && (
                    <div className="absolute bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-4 px-4 py-2 bg-black/50 rounded-full text-white backdrop-blur-md opacity-0 group-hover/nav:opacity-100 transition-opacity">
                        <button
                            onClick={(e) => { e.stopPropagation(); handlePrev(); }}
                            className="p-1 hover:text-brand-400 transition-colors"
                            aria-label="Previous item in post"
                        >
                            <ChevronLeft size={24} />
                        </button>
                        <span className="text-sm font-medium tracking-widest min-w-[3rem] text-center">
                            {currentIndex + 1} / {mediaList.length}
                        </span>
                        <button
                            onClick={(e) => { e.stopPropagation(); handleNext(); }}
                            className="p-1 hover:text-brand-400 transition-colors"
                            aria-label="Next item in post"
                        >
                            <ChevronRight size={24} />
                        </button>
                    </div>
                )}
            </div>

            {/* Sidebar Details Area Right */}
            <MediaSidebar post={post} media={currentMedia} onClose={onClose} />
        </div>,
        document.body
    );
}
