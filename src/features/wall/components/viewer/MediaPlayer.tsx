import { convertFileSrc } from '@tauri-apps/api/core';
import type { Media } from '../../../../types/wall';
import { Image as ImageIcon } from 'lucide-react';
import { useState } from 'react';
import { CustomVideoPlayer } from './CustomVideoPlayer';

export function MediaPlayer({ media }: { media: Media }) {
    const [hasError, setHasError] = useState(false);
    const src = convertFileSrc(media.file_path);
    const isVideo = media.media_type === "VIDEO";

    if (hasError) {
        return (
            <div className="w-full h-full flex flex-col items-center justify-center bg-black/95 text-surface-400">
                <ImageIcon size={48} className="mb-4 opacity-50" />
                <p>Failed to load media file.</p>
                <p className="text-xs mt-2 opacity-50 break-all px-8 text-center">{media.file_path}</p>
            </div>
        );
    }

    return (
        <div className="w-full h-full flex items-center justify-center bg-black/95">
            {isVideo ? (
                <CustomVideoPlayer
                    src={src}
                    onError={() => setHasError(true)}
                />
            ) : (
                <img
                    src={src}
                    alt="Media"
                    onError={() => setHasError(true)}
                    className="max-w-full max-h-full object-contain"
                />
            )}
        </div>
    );
}
