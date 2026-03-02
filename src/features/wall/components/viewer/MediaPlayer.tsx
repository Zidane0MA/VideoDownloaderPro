import { convertFileSrc } from '@tauri-apps/api/core';
import type { Media } from '../../../../types/wall';
import { Image as ImageIcon } from 'lucide-react';
import { useState, useEffect } from 'react';
import { CustomVideoPlayer } from './CustomVideoPlayer';

export function MediaPlayer({ media }: { media: Media }) {
    const [hasError, setHasError] = useState(false);
    const [subtitleUrl, setSubtitleUrl] = useState<string>();

    const src = convertFileSrc(media.file_path);
    const isVideoOrAudio = media.media_type === "VIDEO" || media.media_type === "AUDIO";
    const posterSrc = media.thumbnail_path ? convertFileSrc(media.thumbnail_path) : undefined;

    useEffect(() => {
        const checkSubtitle = async () => {
            if (isVideoOrAudio && media.file_path) {
                const lastDot = media.file_path.lastIndexOf('.');
                if (lastDot !== -1) {
                    const vttPath = media.file_path.substring(0, lastDot) + '.vtt';
                    // Temporary workaround: bypass `fs.exists` to avoid permission errors
                    // Just set the subtitle URL blindly, and let the `<track>` fail silently (404) if it doesn't exist.
                    setSubtitleUrl(convertFileSrc(vttPath));
                }
            }
        };
        checkSubtitle();
    }, [media.file_path, isVideoOrAudio]);

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
            {isVideoOrAudio ? (
                <CustomVideoPlayer
                    src={src}
                    poster={posterSrc}
                    subtitleSrc={subtitleUrl}
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
