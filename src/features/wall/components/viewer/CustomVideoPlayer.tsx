import { useState, useRef, useEffect, MouseEvent as ReactMouseEvent } from 'react';
import { useSettingsStore } from '../../../settings/SettingsStore';
import { Play, Pause, Volume2, VolumeX, Maximize, FastForward } from 'lucide-react';

interface CustomVideoPlayerProps {
    src: string;
    onError: () => void;
}

export function CustomVideoPlayer({ src, onError }: CustomVideoPlayerProps) {
    const { settings, updateSetting } = useSettingsStore();
    const savedVolume = parseFloat(settings.player_volume) || 1;

    const videoRef = useRef<HTMLVideoElement>(null);
    const [isPlaying, setIsPlaying] = useState(true); // autoPlay by default
    const [isMuted, setIsMuted] = useState(false);
    const [volume, setVolume] = useState(savedVolume);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [isHoldingToSpeed, setIsHoldingToSpeed] = useState(false);
    const [showControls, setShowControls] = useState(true);

    const holdTimeoutRef = useRef<number | null>(null);
    const controlsTimeoutRef = useRef<number | null>(null);
    const saveVolumeRef = useRef<number | null>(null);

    // Initial setup
    useEffect(() => {
        if (videoRef.current) {
            // Reset state for the new source
            setCurrentTime(0);
            setDuration(0);
            setIsPlaying(true);
            setIsHoldingToSpeed(false);
            videoRef.current.volume = volume;
            videoRef.current.play().catch(() => {
                setIsPlaying(false);
            });
        }
    }, [src]);

    // Format time (e.g., 01:23)
    const formatTime = (time: number) => {
        if (isNaN(time)) return "0:00";
        const mins = Math.floor(time / 60);
        const secs = Math.floor(time % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const togglePlay = () => {
        if (videoRef.current) {
            if (isPlaying) {
                videoRef.current.pause();
            } else {
                videoRef.current.play();
            }
            setIsPlaying(!isPlaying);
        }
    };

    const toggleMute = (e: ReactMouseEvent) => {
        e.stopPropagation();
        if (videoRef.current) {
            const newMuted = !isMuted;
            videoRef.current.muted = newMuted;
            setIsMuted(newMuted);
        }
    };

    const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        e.stopPropagation();
        const value = parseFloat(e.target.value);
        if (videoRef.current) {
            videoRef.current.volume = value;
            setVolume(value);
            if (value > 0 && isMuted) {
                videoRef.current.muted = false;
                setIsMuted(false);
            }
        }
        // Debounced persist to settings DB
        if (saveVolumeRef.current) clearTimeout(saveVolumeRef.current);
        saveVolumeRef.current = window.setTimeout(() => {
            updateSetting('player_volume', value.toString());
        }, 300);
    };

    const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
        e.stopPropagation();
        const value = parseFloat(e.target.value);
        if (videoRef.current) {
            videoRef.current.currentTime = value;
            setCurrentTime(value);
        }
    };

    const handleTimeUpdate = () => {
        if (videoRef.current) {
            setCurrentTime(videoRef.current.currentTime);
        }
    };

    const handleLoadedMetadata = () => {
        if (videoRef.current) {
            setDuration(videoRef.current.duration);
        }
    };

    const toggleFullScreen = (e: ReactMouseEvent) => {
        e.stopPropagation();
        if (videoRef.current) {
            if (document.fullscreenElement) {
                document.exitFullscreen();
            } else {
                videoRef.current.requestFullscreen();
            }
        }
    };

    // Hold to speed up logic
    const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
        // Only react to left click
        if (e.button !== 0) return;

        holdTimeoutRef.current = window.setTimeout(() => {
            setIsHoldingToSpeed(true);
            if (videoRef.current) {
                videoRef.current.playbackRate = 2.0;
            }
        }, 400); // 400ms threshold for "hold"
    };

    const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
        if (e.button !== 0) return;

        if (holdTimeoutRef.current) {
            clearTimeout(holdTimeoutRef.current);
            holdTimeoutRef.current = null;
        }

        if (isHoldingToSpeed) {
            // It was a hold -> release it and go back to normal speed
            setIsHoldingToSpeed(false);
            if (videoRef.current) {
                videoRef.current.playbackRate = 1.0;
            }
        } else {
            // It was a quick tap -> toggle play
            togglePlay();
        }
    };

    const handlePointerLeave = () => {
        if (holdTimeoutRef.current) {
            clearTimeout(holdTimeoutRef.current);
            holdTimeoutRef.current = null;
        }
        if (isHoldingToSpeed) {
            setIsHoldingToSpeed(false);
            if (videoRef.current) {
                videoRef.current.playbackRate = 1.0;
            }
        }
    };

    // Auto-hide controls
    const resetControlsTimeout = () => {
        setShowControls(true);
        if (controlsTimeoutRef.current) {
            clearTimeout(controlsTimeoutRef.current);
        }
        controlsTimeoutRef.current = window.setTimeout(() => {
            if (isPlaying) {
                setShowControls(false);
            }
        }, 3000);
    };

    useEffect(() => {
        resetControlsTimeout();
        return () => {
            if (controlsTimeoutRef.current) clearTimeout(controlsTimeoutRef.current);
        };
    }, [isPlaying]);

    return (
        <div
            className="relative w-full h-full flex items-center justify-center bg-black group"
            onMouseMove={resetControlsTimeout}
            onMouseLeave={() => isPlaying && setShowControls(false)}
        >
            <video
                ref={videoRef}
                src={src}
                className="max-w-full max-h-full object-contain"
                onTimeUpdate={handleTimeUpdate}
                onLoadedMetadata={handleLoadedMetadata}
                onEnded={() => setIsPlaying(false)}
                onError={onError}
                loop
            />

            {/* Interaction Overlay (handles play/pause & hold-to-speedup for entire area) */}
            <div
                className="absolute inset-0 z-[5] cursor-pointer"
                onPointerDown={handlePointerDown}
                onPointerUp={handlePointerUp}
                onPointerLeave={handlePointerLeave}
            />

            {/* Hold to speed up indicator */}
            <div
                className={`absolute top-8 left-1/2 -translate-x-1/2 bg-black/30 backdrop-blur-sm text-white px-6 py-3 rounded-full flex items-center gap-3 transition-all duration-300 pointer-events-none z-30 ${isHoldingToSpeed ? 'opacity-100 translate-y-0 scale-100' : 'opacity-0 -translate-y-4 scale-95'
                    }`}
            >
                <FastForward className="w-5 h-5 animate-pulse" />
                <span className="font-semibold tracking-wider">Fast-forward 2x</span>
            </div>

            {/* Play/Pause center big button (when paused and not interacting) */}
            {!isPlaying && !isHoldingToSpeed && (
                <div
                    className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-20 h-20 bg-black/30 hover:bg-black/50 text-white rounded-full flex items-center justify-center backdrop-blur-sm cursor-pointer transition-all hover:scale-110 z-[10] shadow-xl shadow-brand-500/20"
                    onClick={togglePlay}
                >
                    <Play className="w-10 h-10 ml-1" />
                </div>
            )}

            {/* Bottom Controls Bar */}
            <div
                className={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent pt-16 pb-6 px-6 transition-all duration-300 z-20 ${showControls ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4 pointer-events-none'
                    }`}
                onClick={(e) => e.stopPropagation()}
            >
                <div className="max-w-screen-lg mx-auto flex flex-col gap-3">
                    {/* Progress Bar */}
                    <div className="flex items-center gap-4">
                        <span className="text-white text-xs font-medium min-w-[3rem] text-right">
                            {formatTime(currentTime)}
                        </span>

                        <div className="relative flex-1 group/slider h-5 flex items-center">
                            <input
                                type="range"
                                min="0"
                                max={duration || 100}
                                value={currentTime}
                                onChange={handleSeek}
                                className="absolute w-full h-1.5 opacity-0 cursor-pointer z-10"
                            />
                            {/* Visual Progress Bar */}
                            <div className="w-full h-1.5 bg-white/20 rounded-full overflow-hidden group-hover/slider:h-2 transition-all">
                                <div
                                    className="h-full bg-brand-500"
                                    style={{ width: `${(currentTime / (duration || 1)) * 100}%` }}
                                />
                            </div>
                            {/* Thumb */}
                            <div
                                className="absolute h-3 w-3 bg-white rounded-full pointer-events-none scale-0 group-hover/slider:scale-100 transition-transform shadow-lg"
                                style={{
                                    left: `calc(${(currentTime / (duration || 1)) * 100}% - 6px)`
                                }}
                            />
                        </div>

                        <span className="text-white/70 text-xs font-medium min-w-[3rem]">
                            {formatTime(duration)}
                        </span>
                    </div>

                    {/* Secondary Controls (Play/Pause, Volume, Fullscreen) */}
                    <div className="flex items-center justify-between mt-1">
                        <div className="flex items-center gap-6">
                            <button
                                onClick={togglePlay}
                                className="text-white hover:text-brand-400 transition-colors"
                            >
                                {isPlaying ? <Pause size={24} /> : <Play size={24} />}
                            </button>

                            <div className="flex items-center gap-3 group/volume">
                                <button
                                    onClick={toggleMute}
                                    className="text-white hover:text-brand-400 transition-colors"
                                >
                                    {isMuted || volume === 0 ? <VolumeX size={20} /> : <Volume2 size={20} />}
                                </button>
                                <div className="w-0 overflow-hidden group-hover/volume:w-[108px] transition-all duration-300 ease-out flex items-center">
                                    <div className="relative w-24 ml-1.5 group/volume-slider h-5 flex items-center">
                                        <input
                                            type="range"
                                            min="0"
                                            max="1"
                                            step="0.01"
                                            value={isMuted ? 0 : volume}
                                            onChange={handleVolumeChange}
                                            className="absolute w-full h-1.5 opacity-0 cursor-pointer z-10"
                                        />
                                        {/* Visual Volume Bar */}
                                        <div className="w-full h-1.5 bg-white/20 rounded-full overflow-hidden group-hover/volume-slider:h-2 transition-all">
                                            <div
                                                className="h-full bg-brand-500"
                                                style={{ width: `${(isMuted ? 0 : volume) * 100}%` }}
                                            />
                                        </div>
                                        {/* Thumb */}
                                        <div
                                            className="absolute h-3 w-3 bg-white rounded-full pointer-events-none scale-0 group-hover/volume-slider:scale-100 transition-transform shadow-lg"
                                            style={{
                                                left: `calc(${(isMuted ? 0 : volume) * 100}% - 6px)`
                                            }}
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <button
                            onClick={toggleFullScreen}
                            className="text-white hover:text-brand-400 transition-colors"
                        >
                            <Maximize size={20} />
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
