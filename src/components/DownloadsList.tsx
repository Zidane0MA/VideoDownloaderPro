import React from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { DownloadItem } from './DownloadItem';
import { DownloadStatus } from '../types/download';

export const DownloadsList: React.FC = () => {
  const { tasks, isQueuePaused, pauseQueue, resumeQueue } = useDownloadManager();

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

  return (
    <div className="space-y-8">
      {/* Active Downloads Section */}
      <section>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-white">Active Downloads</h2>
          <div className="flex items-center gap-2">
            {!isQueuePaused ? (
              <button 
                onClick={pauseQueue}
                className="text-sm text-surface-400 hover:text-white transition-colors"
              >
                Pause All
              </button>
            ) : (
              <button 
                onClick={resumeQueue}
                className="text-sm text-green-400 hover:text-green-300 transition-colors"
              >
                Resume All
              </button>
            )}
          </div>
        </div>
        
        {activeTasks.length > 0 ? (
          <div className="grid gap-4">
            {activeTasks.map(task => (
              <DownloadItem key={task.id} task={task} />
            ))}
          </div>
        ) : (
          <div className="p-8 text-center border-2 border-dashed border-surface-700/50 rounded-2xl">
            <p className="text-surface-400">No active downloads</p>
          </div>
        )}
      </section>

      {/* History Section */}
      {historyTasks.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold text-white mb-4">History</h2>
          <div className="grid gap-4">
            {historyTasks.map(task => (
              <DownloadItem key={task.id} task={task} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
};
