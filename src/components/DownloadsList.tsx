import React, { useState, useMemo } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { DownloadItem } from './DownloadItem';
import { PlaylistGroup } from './PlaylistGroup';
import { DownloadStatus } from '../types/download';
import type { DownloadTask } from '../types/download';
import { DownloadCloud, Play, Pause, History, Download, Trash2, RotateCw } from 'lucide-react';

export const DownloadsList: React.FC = () => {
  const { tasks, isQueuePaused, pauseQueue, resumeQueue, clearHistory, retryAllFailed } = useDownloadManager();
  const [activeTab, setActiveTab] = useState<'active' | 'history'>('active');

  const activeTasks = tasks.filter(task =>
    task.status === DownloadStatus.Processing ||
    task.status === DownloadStatus.Paused ||
    task.status === DownloadStatus.Queued
  );

  const historyTasks = tasks.filter(task =>
    task.status === DownloadStatus.Completed ||
    task.status === DownloadStatus.Failed ||
    task.status === DownloadStatus.Cancelled
  );

  const failedCount = historyTasks.filter(t => t.status === DownloadStatus.Failed).length;

  const currentTasks = activeTab === 'active' ? activeTasks : historyTasks;

  // We add a little margin to the pill based on the tab
  const pillOffset = activeTab === 'active' ? '4px' : 'calc(50% + 2px)';

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Segmented Control Header */}
      <div className="flex items-center justify-between">
        <div className="inline-flex bg-surface-900/60 p-1 rounded-xl border border-surface-700/50 backdrop-blur-md relative shadow-inner">
          {/* Active Tab Background Pill */}
          <div
            className="absolute inset-y-1 bg-surface-700 rounded-lg shadow-sm transition-all duration-300 ease-out"
            style={{
              width: 'calc(50% - 6px)',
              left: pillOffset,
            }}
            aria-hidden="true"
          />

          <button
            onClick={() => setActiveTab('active')}
            className={`relative flex items-center justify-center gap-2 px-6 py-2 text-sm font-medium rounded-lg transition-colors z-10 w-36 ${activeTab === 'active' ? 'text-surface-100' : 'text-surface-400 hover:text-surface-200'
              }`}
          >
            <Download size={16} /> Active
            {activeTasks.length > 0 && (
              <span className={`flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full text-[10px] tabular-nums transition-colors ${activeTab === 'active' ? 'bg-brand-500/20 text-brand-300 border border-brand-500/30' : 'bg-surface-800 text-surface-500'
                }`}>
                {activeTasks.length}
              </span>
            )}
          </button>

          <button
            onClick={() => setActiveTab('history')}
            className={`relative flex items-center justify-center gap-2 px-6 py-2 text-sm font-medium rounded-lg transition-colors z-10 w-36 ${activeTab === 'history' ? 'text-surface-100' : 'text-surface-400 hover:text-surface-200'
              }`}
          >
            <History size={16} /> History
          </button>
        </div>

        {/* Actions based on tab */}
        {activeTab === 'active' && (
          <div className="flex items-center gap-2 animate-in fade-in duration-300 zoom-in-95">
            {!isQueuePaused ? (
              <button
                onClick={pauseQueue}
                disabled={activeTasks.length === 0}
                className="flex items-center gap-1.5 text-xs font-medium px-4 py-2 rounded-lg bg-surface-800 hover:bg-surface-700 text-surface-300 hover:text-surface-100 transition-all disabled:opacity-50 disabled:cursor-not-allowed border border-surface-700 shadow-sm"
              >
                <Pause size={14} className="fill-current" /> Pause Queue
              </button>
            ) : (
              <button
                onClick={resumeQueue}
                className="flex items-center gap-1.5 text-xs font-medium px-4 py-2 rounded-lg bg-yellow-500/10 hover:bg-yellow-500/20 text-yellow-500 transition-all border border-yellow-500/30 shadow-sm"
              >
                <Play size={14} className="fill-current" /> Resume Queue
              </button>
            )}
          </div>
        )}

        {activeTab === 'history' && historyTasks.length > 0 && (
          <div className="flex items-center gap-2 animate-in fade-in duration-300 zoom-in-95">
            {failedCount > 0 && (
              <button
                onClick={retryAllFailed}
                className="flex items-center gap-1.5 text-xs font-medium px-4 py-2 rounded-lg bg-brand-500/10 hover:bg-brand-500/20 text-brand-400 transition-all border border-brand-500/20 shadow-sm"
              >
                <RotateCw size={14} /> Retry Failed
                <span className="min-w-[18px] h-[18px] flex items-center justify-center rounded-full bg-brand-500/20 text-[10px] text-brand-300 px-1" style={{ fontVariantNumeric: 'tabular-nums' }}>
                  {failedCount}
                </span>
              </button>
            )}
            <button
              onClick={clearHistory}
              className="flex items-center gap-1.5 text-xs font-medium px-4 py-2 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 transition-all border border-red-500/20 shadow-sm"
            >
              <Trash2 size={14} /> Clear History
            </button>
          </div>
        )}
      </div>

      {/* List Container */}
      <div className="min-h-[400px] relative">
        {currentTasks.length > 0 ? (
          <GroupedTaskList tasks={currentTasks} />
        ) : (
          <div className="absolute inset-x-0 flex flex-col items-center justify-center p-16 border border-dashed border-surface-700/70 rounded-2xl bg-surface-800/30 animate-in fade-in slide-in-from-bottom-2 duration-500">
            <div className="p-5 bg-surface-900/80 rounded-full mb-4 border border-surface-700/50 shadow-inner">
              {activeTab === 'active' ? (
                <DownloadCloud className="w-10 h-10 text-surface-500" />
              ) : (
                <History className="w-10 h-10 text-surface-500" />
              )}
            </div>
            <p className="text-lg text-surface-200 font-medium mb-1">
              {activeTab === 'active' ? 'No active downloads' : 'No download history'}
            </p>
            <p className="text-sm text-surface-400">
              {activeTab === 'active' ? 'Add a URL to get started' : 'Your completed and failed downloads will appear here'}
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

// ─── Grouped Rendering ──────────────────────────────────────────────────────

interface GroupedTaskListProps {
  tasks: DownloadTask[];
}

/**
 * Separates standalone downloads from playlist-grouped ones.
 * Tasks with a `source_id` are grouped into a collapsible `PlaylistGroup`.
 * Tasks without a `source_id` are rendered individually.
 */
const GroupedTaskList: React.FC<GroupedTaskListProps> = ({ tasks }) => {
  const { standaloneTasks, groups } = useMemo(() => {
    const standalone: DownloadTask[] = [];
    const groupMap = new Map<string, { name: string; tasks: DownloadTask[] }>();

    for (const task of tasks) {
      if (task.source_id && task.source_name) {
        const existing = groupMap.get(task.source_id);
        if (existing) {
          existing.tasks.push(task);
        } else {
          groupMap.set(task.source_id, {
            name: task.source_name,
            tasks: [task],
          });
        }
      } else {
        standalone.push(task);
      }
    }

    return {
      standaloneTasks: standalone,
      groups: Array.from(groupMap.entries()),
    };
  }, [tasks]);

  return (
    <div className="space-y-3">
      {/* Playlist Groups */}
      {groups.map(([sourceId, group]) => (
        <PlaylistGroup
          key={sourceId}
          sourceId={sourceId}
          sourceName={group.name}
          tasks={group.tasks}
        />
      ))}

      {/* Standalone Downloads */}
      {standaloneTasks.map(task => (
        <div key={task.id} className="px-0.5">
          <DownloadItem task={task} />
        </div>
      ))}
    </div>
  );
};
