import React, { useState, useMemo } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { DownloadItem } from './DownloadItem';
import { PlaylistGroup } from './PlaylistGroup';
import { DownloadStatus, DownloadTask } from '../types/download';
import { DownloadCloud, Play, Pause, History, Download, Trash2, RotateCw } from 'lucide-react';
import { GroupedVirtuoso } from 'react-virtuoso';

export const DownloadsList: React.FC = () => {
  const { 
    tasks, 
    expandedGroups, 
    toggleGroup,
    isQueuePaused, 
    pauseQueue, 
    resumeQueue, 
    clearHistory, 
    retryAllFailed 
  } = useDownloadManager();
  
  const [activeTab, setActiveTab] = useState<'active' | 'history'>('active');

  const activeTasks = useMemo(() => tasks.filter(task =>
    task.status === DownloadStatus.Processing ||
    task.status === DownloadStatus.Paused ||
    task.status === DownloadStatus.Queued
  ), [tasks]);

  const historyTasks = useMemo(() => tasks.filter(task =>
    task.status === DownloadStatus.Completed ||
    task.status === DownloadStatus.Failed ||
    task.status === DownloadStatus.Cancelled
  ), [tasks]);

  const currentTasks = activeTab === 'active' ? activeTasks : historyTasks;
  const failedCount = historyTasks.filter(t => t.status === DownloadStatus.Failed).length;
  const pillOffset = activeTab === 'active' ? '4px' : 'calc(50% + 2px)';

  // ─── Grouped Logic for Virtuoso ───────────────────────────────────────────
  
  const { flattenedGroups, groupCounts, allItems } = useMemo(() => {
    const standalone: DownloadTask[] = [];
    const groupMap = new Map<number, { name: string; tasks: DownloadTask[] }>();

    for (const task of currentTasks) {
      if (task.source_id !== undefined && task.source_id !== null && task.source_name) {
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

    const groups = Array.from(groupMap.entries());
    const flattenedGroups: { sourceId: number; name: string; tasks: DownloadTask[] }[] = [];
    const groupCounts: number[] = [];
    const allItems: DownloadTask[] = [];

    // Add Standalone first (or as a single group)
    if (standalone.length > 0) {
      flattenedGroups.push({ sourceId: -1, name: 'Standalone', tasks: standalone });
      groupCounts.push(standalone.length);
      allItems.push(...standalone);
    }

    // Add actual playlist groups
    for (const [sourceId, g] of groups) {
      const isExpanded = expandedGroups[sourceId] !== false; // Default to true
      flattenedGroups.push({ sourceId, name: g.name, tasks: g.tasks });
      
      if (isExpanded) {
        groupCounts.push(g.tasks.length);
        allItems.push(...g.tasks);
      } else {
        groupCounts.push(0);
      }
    }

    return { flattenedGroups, groupCounts, allItems };
  }, [currentTasks, expandedGroups]);

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 min-h-[600px]">
      {/* Segmented Control Header */}
      <div className="flex items-center justify-between">
        <div className="inline-flex bg-surface-900/60 p-1 rounded-xl border border-surface-700/50 backdrop-blur-md relative shadow-inner">
          <div
            className="absolute inset-y-1 bg-surface-700 rounded-lg shadow-sm transition-all duration-300 ease-out"
            style={{ width: 'calc(50% - 6px)', left: pillOffset }}
            aria-hidden="true"
          />

          <button
            onClick={() => setActiveTab('active')}
            className={`relative flex items-center justify-center gap-2 px-6 py-2 text-sm font-medium rounded-lg transition-colors z-10 w-36 ${activeTab === 'active' ? 'text-surface-100' : 'text-surface-400 hover:text-surface-200'}`}
          >
            <Download size={16} /> Active
            {activeTasks.length > 0 && (
              <span className={`flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full text-[10px] tabular-nums transition-colors ${activeTab === 'active' ? 'bg-brand-500/20 text-brand-300 border border-brand-500/30' : 'bg-surface-800 text-surface-500'}`}>
                {activeTasks.length}
              </span>
            )}
          </button>

          <button
            onClick={() => setActiveTab('history')}
            className={`relative flex items-center justify-center gap-2 px-6 py-2 text-sm font-medium rounded-lg transition-colors z-10 w-36 ${activeTab === 'history' ? 'text-surface-100' : 'text-surface-400 hover:text-surface-200'}`}
          >
            <History size={16} /> History
          </button>
        </div>

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
            <button onClick={clearHistory} className="flex items-center gap-1.5 text-xs font-medium px-4 py-2 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-400 transition-all border border-red-500/20 shadow-sm">
              <Trash2 size={14} /> Clear History
            </button>
          </div>
        )}
      </div>

      {/* List Container */}
      <div className="relative min-h-[400px]">
        {currentTasks.length > 0 ? (
          <GroupedVirtuoso
            useWindowScroll
            overscan={400}
            style={{ width: '100%' }}
            groupCounts={groupCounts}
            groupContent={(index) => {
              const group = flattenedGroups[index];
              if (!group) return null;
              if (group.sourceId === -1) return <div className="h-4" />; // Spacer for standalone
              return (
                <div className="py-2 bg-surface-900 sticky top-0 z-20">
                  <PlaylistGroup
                    sourceName={group.name}
                    sourceId={group.sourceId}
                    tasks={group.tasks}
                    isExpanded={expandedGroups[group.sourceId] !== false}
                    onToggle={() => toggleGroup(group.sourceId)}
                  />
                </div>
              );
            }}
            itemContent={(index) => {
              const task = allItems[index];
              if (!task) return null;
              return (
                <div className="px-1 py-1">
                  <DownloadItem task={task} />
                </div>
              );
            }}
          />
        ) : (
          <div className="absolute inset-0 flex flex-col items-center justify-center p-16 border border-dashed border-surface-700/70 rounded-2xl bg-surface-800/30 animate-in fade-in slide-in-from-bottom-2 duration-500">
            <div className="p-5 bg-surface-900/80 rounded-full mb-4 border border-surface-700/50 shadow-inner">
              {activeTab === 'active' ? <DownloadCloud className="w-10 h-10 text-surface-500" /> : <History className="w-10 h-10 text-surface-500" />}
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
