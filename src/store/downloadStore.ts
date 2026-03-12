import { create } from 'zustand';
import { DownloadTask } from '../types/download';

interface DownloadState {
  tasks: Record<string, DownloadTask>;
  expandedGroups: Record<number, boolean>;
  isQueuePaused: boolean;

  // Actions
  setTasks: (tasks: DownloadTask[]) => void;
  addTask: (task: DownloadTask) => void;
  updateTask: (id: number, updates: Partial<DownloadTask>) => void;
  removeTask: (id: number) => void;
  setQueuePaused: (isPaused: boolean) => void;
  toggleGroup: (sourceId: number) => void;
}

export const useDownloadStore = create<DownloadState>((set) => ({
  tasks: {},
  expandedGroups: {},
  isQueuePaused: false,

  setTasks: (taskList) => set({
    tasks: taskList.reduce((acc, task) => {
      acc[task.id] = task;
      return acc;
    }, {} as Record<string, DownloadTask>)
  }),

  addTask: (task) => set((state) => ({
    tasks: { ...state.tasks, [task.id]: task }
  })),

  updateTask: (id, updates) => set((state) => {
    const currentTask = state.tasks[id];
    if (!currentTask) return state; // No-op if task not found

    return {
      tasks: {
        ...state.tasks,
        [id]: { ...currentTask, ...updates }
      }
    };
  }),

  removeTask: (id) => set((state) => {
    const { [id]: _, ...rest } = state.tasks;
    return { tasks: rest };
  }),

  setQueuePaused: (isPaused) => set({ isQueuePaused: isPaused }),
 
  toggleGroup: (sourceId) => set((state) => {
    // If it's undefined, it's currently expanded (default), so first toggle should make it false.
    const current = state.expandedGroups[sourceId];
    const nextValue = current === undefined ? false : !current;
    
    return {
      expandedGroups: {
        ...state.expandedGroups,
        [sourceId]: nextValue
      }
    };
  }),
}));
