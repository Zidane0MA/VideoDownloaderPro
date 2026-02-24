import { useEffect, useState } from 'react';
import type { Post } from '../../../../types/wall';
import { MediaPlayer } from './MediaPlayer';
import { MediaSidebar } from './MediaSidebar';
import { X, ChevronLeft, ChevronRight } from 'lucide-react';

interface MediaViewerProps {
    post: Post;
    onClose: () => void;
}

export function MediaViewer({ post, onClose }: MediaViewerProps) {
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
                handlePrev();
            } else if (e.key === 'ArrowRight') {
                handleNext();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [currentIndex, mediaList.length]); // Re-bind when index changes 

    const handlePrev = () => {
        if (mediaList.length <= 1) return;
        setCurrentIndex((prev) => (prev > 0 ? prev - 1 : mediaList.length - 1));
    };

    const handleNext = () => {
        if (mediaList.length <= 1) return;
        setCurrentIndex((prev) => (prev < mediaList.length - 1 ? prev + 1 : 0));
    };

    if (!currentMedia) {
        return (
            <div className="fixed inset-0 z-50 bg-black/95 flex items-center justify-center">
                <p className="text-surface-400">No media available.</p>
                <button onClick={onClose} className="absolute top-4 right-4 p-2 bg-surface-800/50 hover:bg-surface-800 text-white rounded-full transition-colors">
                    <X size={24} />
                </button>
            </div>
        );
    }

    return (
        <div className="fixed inset-0 z-[100] bg-black/95 flex flex-col md:flex-row backdrop-blur-sm animate-in fade-in duration-200">
            {/* Close Button Desktop */}
            <button
                onClick={onClose}
                className="absolute top-4 right-4 z-50 p-2 bg-black/50 hover:bg-black text-white rounded-full transition-colors opacity-80 hover:opacity-100"
                aria-label="Close viewer"
            >
                <X size={24} />
            </button>

            {/* Main Player Area Left */}
            <div className="relative flex-1 flex items-center justify-center min-h-[50vh] bg-black">
                <MediaPlayer media={currentMedia} />

                {/* Gallery Navigation Controls */}
                {mediaList.length > 1 && (
                    <>
                        <button
                            onClick={handlePrev}
                            className="absolute left-4 top-1/2 -translate-y-1/2 p-3 bg-black/50 hover:bg-black text-white rounded-full transition-all hover:scale-110"
                            aria-label="Previous item"
                        >
                            <ChevronLeft size={32} />
                        </button>
                        <button
                            onClick={handleNext}
                            className="absolute right-4 top-1/2 -translate-y-1/2 p-3 bg-black/50 hover:bg-black text-white rounded-full transition-all hover:scale-110"
                            aria-label="Next item"
                        >
                            <ChevronRight size={32} />
                        </button>

                        {/* Pagination Indicator */}
                        <div className="absolute bottom-4 left-1/2 -translate-x-1/2 px-4 py-1.5 bg-black/50 rounded-full text-white text-sm font-medium tracking-widest backdrop-blur-md">
                            {currentIndex + 1} / {mediaList.length}
                        </div>
                    </>
                )}
            </div>

            {/* Sidebar Details Area Right */}
            <MediaSidebar post={post} media={currentMedia} onClose={onClose} />
        </div>
    );
}
