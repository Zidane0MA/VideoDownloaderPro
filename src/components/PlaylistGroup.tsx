import React from 'react';
import { DownloadTask, DownloadStatus } from '../types/download';
import {
    ChevronDown,
    ChevronRight,
    ListVideo,
    CheckCircle2,
    Loader2,
    AlertCircle,
    Pause,
} from 'lucide-react';

interface PlaylistGroupProps {
    sourceName: string;
    sourceId: number;
    tasks: DownloadTask[];
    isExpanded: boolean;
    onToggle: () => void;
}

export const PlaylistGroup: React.FC<PlaylistGroupProps> = ({
    sourceName,
    tasks,
    isExpanded,
    onToggle,
}) => {

    // Calculate aggregate stats
    const completed = tasks.filter(t => t.status === DownloadStatus.Completed).length;
    const failed = tasks.filter(t => t.status === DownloadStatus.Failed).length;
    const active = tasks.filter(
        t => t.status === DownloadStatus.Processing || t.status === DownloadStatus.Queued
    ).length;
    const paused = tasks.filter(t => t.status === DownloadStatus.Paused).length;
    const total = tasks.length;

    // Aggregate progress across all tasks
    const avgProgress =
        total > 0
            ? tasks.reduce((sum, t) => {
                if (t.status === DownloadStatus.Completed) return sum + 100;
                return sum + t.progress;
            }, 0) / total
            : 0;

    // Determine dominant status icon
    const StatusIcon = () => {
        if (active > 0)
            return <Loader2 size={14} className="animate-spin text-brand-400" />;
        if (paused > 0) return <Pause size={14} className="text-yellow-400 fill-current" />;
        if (failed > 0 && completed === 0)
            return <AlertCircle size={14} className="text-red-400" />;
        if (completed === total)
            return <CheckCircle2 size={14} className="text-green-400" />;
        return <ListVideo size={14} className="text-surface-400" />;
    };

    return (
        <div
            className={`
                border border-surface-700 bg-surface-800/60 overflow-hidden transition-all
                ${isExpanded ? 'rounded-t-xl border-b-surface-700/50' : 'rounded-xl'}
            `}
        >
            {/* Group Header (clickable) */}
            <button
                onClick={onToggle}
                className="w-full flex items-center gap-3 px-4 py-3 hover:bg-surface-700/30 transition-colors text-left"
            >
                {/* Expand Chevron */}
                <div className="text-surface-500 transition-transform duration-200">
                    {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                </div>

                {/* Status Icon */}
                <StatusIcon />

                {/* Playlist Name */}
                <span className="text-sm font-medium text-surface-100 truncate flex-1" title={sourceName}>
                    {sourceName}
                </span>

                {/* Progress Pill */}
                <div className="flex items-center gap-2 flex-shrink-0">
                    {/* Completion fraction */}
                    <span className="text-[11px] text-surface-400 tabular-nums">
                        {completed}/{total}
                    </span>

                    {/* Micro progress bar */}
                    <div className="w-16 h-1.5 bg-surface-900 rounded-full overflow-hidden">
                        <div
                            className="h-full rounded-full transition-all duration-500 bg-brand-500"
                            style={{ width: `${avgProgress}%` }}
                        />
                    </div>

                    {/* Error badge */}
                    {failed > 0 && (
                        <span className="flex items-center justify-center min-w-[18px] h-[18px] px-1 rounded-full bg-red-500/15 text-[10px] text-red-400 tabular-nums">
                            {failed}
                        </span>
                    )}
                </div>
            </button>

            {/* Expanded Content - REMOVED: Managed by Virtuoso now */}
        </div>
    );
};
