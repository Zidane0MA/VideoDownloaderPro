import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ListVideo, Trash2, RefreshCw, Layers, Plus, X, Loader2, ToggleLeft, ToggleRight } from 'lucide-react';
import { ConfirmModal } from '../../components/ui/ConfirmModal';
import { QUICK_ACTIONS } from './config/quickActions';

export interface SourceResponse {
    id: string;
    platform_id: string;
    name: string;
    url: string;
    source_type: string;
    sync_mode: string;
    is_active: boolean;
    last_checked: string | null;
    post_count: number;
}

function formatRelativeTime(isoDate: string): string {
    const date = new Date(isoDate);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);

    if (diffSecs < 60) return 'just now';
    const diffMins = Math.floor(diffSecs / 60);
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
}

export const Sources: React.FC = () => {
    const [sources, setSources] = useState<SourceResponse[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [showAddInput, setShowAddInput] = useState(false);
    const [addUrl, setAddUrl] = useState('');
    const [isAdding, setIsAdding] = useState(false);

    // Confirm Modal state
    const [isConfirmOpen, setIsConfirmOpen] = useState(false);
    const [sourceToDelete, setSourceToDelete] = useState<{ id: string, name: string } | null>(null);

    const fetchSources = useCallback(async () => {
        try {
            setIsLoading(true);
            setError(null);
            const data = await invoke<SourceResponse[]>('get_sources_command');
            setSources(data);
        } catch (err) {
            console.error('Failed to fetch sources:', err);
            setError(String(err) || 'Failed to load sources');
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchSources();
    }, [fetchSources]);

    const handleDeleteClick = (id: string, name: string) => {
        setSourceToDelete({ id, name });
        setIsConfirmOpen(true);
    };

    const handleConfirmDelete = async () => {
        if (!sourceToDelete) return;
        setIsConfirmOpen(false);
        setError(null);
        try {
            await invoke('delete_source_command', { sourceId: sourceToDelete.id });
            fetchSources();
        } catch (err) {
            console.error(err);
            setError(String(err) || 'Failed to delete source');
        } finally {
            setSourceToDelete(null);
        }
    };

    const handleToggleActive = async (source: SourceResponse) => {
        setError(null);
        try {
            await invoke('update_source_command', {
                request: {
                    source_id: source.id,
                    is_active: !source.is_active,
                },
            });
            fetchSources();
        } catch (err) {
            console.error(err);
            setError(String(err) || 'Failed to update source');
        }
    };

    const handleAddSource = async () => {
        if (!addUrl.trim()) return;
        setIsAdding(true);
        setError(null);
        try {
            const result = await invoke<{ source_id: string; items_queued: number }>('add_source_command', { url: addUrl.trim() });
            setAddUrl('');
            setShowAddInput(false);
            fetchSources();
            // Brief success feedback in console
            console.info(`Source added: ${result.source_id}, ${result.items_queued} items queued`);
        } catch (err) {
            console.error(err);
            setError(String(err) || 'Failed to add source');
        } finally {
            setIsAdding(false);
        }
    };

    const handleQuickAction = (presetUrl: string) => {
        setAddUrl(presetUrl);
        if (!showAddInput) {
            setShowAddInput(true);
        }
        // Small timeout to allow input to mount and focus if it was hidden
        setTimeout(() => {
            const input = document.getElementById('source-url-input');
            if (input) input.focus();
        }, 50);
    };

    if (isLoading) {
        return (
            <div className="flex justify-center items-center h-64 text-surface-400">
                <RefreshCw className="animate-spin" />
                <span className="ml-2">Loading content sources...</span>
            </div>
        );
    }

    return (
        <div className="space-y-4 animate-in fade-in duration-300">
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-semibold text-surface-100 flex items-center gap-2">
                    <ListVideo className="text-brand-400" />
                    Content Sources
                </h2>
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => setShowAddInput(!showAddInput)}
                        className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-white bg-brand-600 hover:bg-brand-500 rounded-lg transition-colors shadow-md shadow-brand-600/20"
                    >
                        {showAddInput ? <X size={14} /> : <Plus size={14} />}
                        {showAddInput ? 'Cancel' : 'Add Source'}
                    </button>
                    <button
                        onClick={fetchSources}
                        className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-surface-300 hover:text-brand-400 bg-surface-800 hover:bg-surface-700 rounded-lg transition-colors border border-surface-700"
                    >
                        <RefreshCw size={14} />
                        Refresh
                    </button>
                </div>
            </div>

            {/* Add Source Input */}
            {showAddInput && (
                <div className="animate-in slide-in-from-top-2 fade-in duration-200 p-4 bg-surface-800 border border-surface-700 rounded-xl mb-6 shadow-lg shadow-surface-900/50">

                    {/* Quick Actions */}
                    <div className="mb-4">
                        <label className="block text-xs font-semibold text-brand-400 mb-2 uppercase tracking-wider">
                            Quick Actions
                        </label>
                        <div className="flex flex-wrap gap-2">
                            {QUICK_ACTIONS.map(action => (
                                <button
                                    key={action.id}
                                    onClick={() => handleQuickAction(action.actionUrl)}
                                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-surface-200 bg-surface-900 border border-surface-700 hover:border-surface-500 hover:bg-surface-700 rounded-lg transition-all"
                                >
                                    <action.icon size={14} className={action.iconColorClass} />
                                    {action.label}
                                </button>
                            ))}
                        </div>
                    </div>

                    <label htmlFor="source-url-input" className="block text-sm font-medium text-surface-400 mb-2">
                        Playlist, Channel or Profile URL
                    </label>
                    <div className="flex gap-2">
                        <input
                            id="source-url-input"
                            type="text"
                            value={addUrl}
                            onChange={(e) => setAddUrl(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && addUrl.trim() && !isAdding) handleAddSource();
                            }}
                            disabled={isAdding}
                            placeholder="https://youtube.com/playlist?list=..."
                            className="flex-1 bg-surface-900 border border-surface-700 rounded-lg px-4 py-2 text-surface-100 placeholder-surface-500 focus:outline-none focus:ring-2 focus:ring-brand-500 disabled:opacity-50"
                            autoFocus
                        />
                        <button
                            onClick={handleAddSource}
                            disabled={!addUrl.trim() || isAdding}
                            className="flex items-center gap-2 px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-md shadow-brand-600/20"
                        >
                            {isAdding ? (
                                <>
                                    <Loader2 size={16} className="animate-spin" />
                                    Adding...
                                </>
                            ) : (
                                <>
                                    <Plus size={16} />
                                    Add
                                </>
                            )}
                        </button>
                    </div>
                    <p className="text-xs text-surface-500 mt-2">
                        Paste a YouTube playlist or channel URL. All videos will be queued for download automatically.
                    </p>
                </div>
            )}

            {error && (
                <div className="text-red-500 text-sm bg-red-500/10 border border-red-500/20 px-3 py-2 rounded-lg break-words mb-4">
                    {error}
                </div>
            )}

            {sources.length === 0 && !showAddInput ? (
                <div className="flex flex-col items-center justify-center h-64 text-surface-400 space-y-4">
                    <div className="p-4 bg-surface-800 rounded-full border border-surface-700">
                        <Layers size={32} />
                    </div>
                    <p>No sources yet.</p>
                    <button
                        onClick={() => setShowAddInput(true)}
                        className="flex items-center gap-2 px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors shadow-md shadow-brand-600/20"
                    >
                        <Plus size={16} />
                        Add Your First Source
                    </button>
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {sources.map((source) => (
                        <div
                            key={source.id}
                            className={`p-4 bg-surface-800 border rounded-xl transition-colors flex items-start gap-4 ${source.is_active
                                ? 'border-surface-700 hover:border-surface-600'
                                : 'border-surface-700/50 opacity-60'
                                }`}
                        >
                            <div className="p-3 bg-surface-900 rounded-lg text-brand-400 flex-shrink-0">
                                <ListVideo size={24} />
                            </div>
                            <div className="flex-1 min-w-0">
                                <h3 className="text-base font-medium text-surface-100 truncate" title={source.name}>
                                    {source.name}
                                </h3>
                                <p className="text-xs text-surface-400 mt-1 truncate" title={source.url}>
                                    {source.url}
                                </p>
                                <div className="flex items-center gap-2 mt-3 flex-wrap">
                                    <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-brand-500/10 text-brand-400 uppercase tracking-wide">
                                        {source.source_type}
                                    </span>
                                    <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-surface-700 text-surface-300 capitalize tracking-wide">
                                        {source.platform_id}
                                    </span>
                                    <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-surface-700 text-surface-300">
                                        {source.post_count} {source.post_count === 1 ? 'video' : 'videos'}
                                    </span>
                                    {source.last_checked && (
                                        <span className="text-[10px] text-surface-500" title={source.last_checked}>
                                            Checked {formatRelativeTime(source.last_checked)}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <div className="flex flex-col items-center gap-2 flex-shrink-0">
                                <button
                                    onClick={() => handleToggleActive(source)}
                                    className={`p-1 rounded-lg transition-colors ${source.is_active
                                        ? 'text-brand-400 hover:text-brand-300'
                                        : 'text-surface-500 hover:text-surface-300'
                                        }`}
                                    title={source.is_active ? 'Disable source' : 'Enable source'}
                                >
                                    {source.is_active ? <ToggleRight size={20} /> : <ToggleLeft size={20} />}
                                </button>
                                <button
                                    onClick={() => handleDeleteClick(source.id, source.name)}
                                    className="p-2 text-surface-500 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                                    title="Delete source"
                                >
                                    <Trash2 size={16} />
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            <ConfirmModal
                isOpen={isConfirmOpen}
                title="Delete Source"
                message={`Are you sure you want to delete the source "${sourceToDelete?.name}"?\nThis won't delete already downloaded media.`}
                onConfirm={handleConfirmDelete}
                onCancel={() => {
                    setIsConfirmOpen(false);
                    setSourceToDelete(null);
                }}
                confirmText="Delete"
                isDanger={true}
            />
        </div>
    );
};
