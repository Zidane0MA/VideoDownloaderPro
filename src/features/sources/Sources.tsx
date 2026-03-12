import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
    ListVideo, Trash2, RefreshCw, Plus, X, ToggleLeft, ToggleRight,
    PlaySquare, Smartphone, Radio, Bookmark, Heart, Box, User, Folder, Layers
} from 'lucide-react';
import { ConfirmModal } from '../../components/ui/ConfirmModal';
import { AddSourceModal } from './components/AddSourceModal';

export interface SourceResponse {
    id: number;
    platform_id: string;
    creator_id: number | null;
    name: string;
    url: string;
    source_type: string;
    feed_type: string | null;
    sync_mode: string;
    is_active: boolean;
    last_checked: string | null;
    post_count: number;
    is_self: boolean;
    avatar_url: string | null;
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

const getFeedIcon = (source: SourceResponse, size = 16) => {
    const type = source.feed_type || source.source_type;
    switch (type.toUpperCase()) {
        case 'VIDEOS': return <PlaySquare size={size} />;
        case 'SHORTS': 
        case 'REELS': return <Smartphone size={size} />;
        case 'STREAMS': return <Radio size={size} />;
        case 'SAVED': return <Bookmark size={size} />;
        case 'LIKED': return <Heart size={size} />;
        case 'PLAYLIST': return <Folder size={size} />;
        default: return <Box size={size} />;
    }
};

type Group = {
    id: string;
    creator_id: number | null;
    name: string;
    platform: string;
    is_self: boolean;
    avatar_url: string | null;
    sources: SourceResponse[];
};

export const Sources: React.FC = () => {
    const [sources, setSources] = useState<SourceResponse[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [isAddModalOpen, setIsAddModalOpen] = useState(false);

    // Confirm Modal state
    const [isConfirmOpen, setIsConfirmOpen] = useState(false);
    const [sourceToDelete, setSourceToDelete] = useState<{ id: number, name: string } | null>(null);

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

    const handleDeleteClick = (id: number, name: string) => {
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

    const { myAccounts, channels, standalone } = React.useMemo(() => {
        const myAccounts: Group[] = [];
        const channels: Group[] = [];
        const standalone: Group[] = [];

        const groupsMap: { [key: string]: Group } = {};
        
        sources.forEach(src => {
            const key = src.creator_id ? `creator_${src.creator_id}` : `standalone_${src.id}`;
            if (!groupsMap[key]) {
                 groupsMap[key] = {
                     id: key,
                     creator_id: src.creator_id,
                     name: src.creator_id ? (src.name || 'Unknown Channel') : src.name,
                     platform: src.platform_id,
                     is_self: src.is_self,
                     avatar_url: src.avatar_url,
                     sources: []
                 };
            }
            if (src.creator_id && src.name && groupsMap[key].name === 'Unknown Channel') {
                groupsMap[key].name = src.name;
            }
            if (src.avatar_url && !groupsMap[key].avatar_url) {
                groupsMap[key].avatar_url = src.avatar_url;
            }
            if (src.is_self) {
                groupsMap[key].is_self = true;
            }
            groupsMap[key].sources.push(src);
        });

        const sortedGroups = Object.values(groupsMap).sort((a, b) => {
            if (b.sources.length !== a.sources.length) return b.sources.length - a.sources.length;
            return a.name.localeCompare(b.name);
        });

        sortedGroups.forEach(g => {
            if (g.is_self) {
                myAccounts.push(g);
            } else if (g.creator_id !== null) {
                channels.push(g);
            } else {
                standalone.push(g);
            }
        });

        return { myAccounts, channels, standalone };
    }, [sources]);

    const renderAvatar = (group: Group) => {
        if (group.avatar_url) {
            return (
                <div className="relative flex-shrink-0">
                    <img src={group.avatar_url} alt={group.name} className="w-12 h-12 rounded-full object-cover border border-surface-700 bg-surface-900" crossOrigin="anonymous" />
                </div>
            );
        }
        return (
            <div className="w-12 h-12 bg-surface-900 rounded-full flex items-center justify-center text-brand-400 border border-surface-700 flex-shrink-0">
                {group.is_self ? <User size={24} /> : (group.creator_id === null ? <Folder size={24} /> : <ListVideo size={24} />)}
            </div>
        );
    };

    const renderGroupCard = (group: Group) => (
        <div
            key={group.id}
            className="p-4 bg-surface-800 border border-surface-700 rounded-xl transition-colors flex flex-col gap-3 shadow-sm hover:border-surface-600 group"
        >
            {/* Header */}
            <div className="flex items-start gap-4">
                {renderAvatar(group)}
                
                <div className="flex-1 min-w-0 pt-0.5">
                    <div className="flex items-center gap-2">
                        <h3 className="text-base font-semibold text-surface-100 truncate" title={group.name}>
                            {group.name}
                        </h3>
                        {group.is_self && (
                            <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-brand-500/10 text-brand-400 border border-brand-500/20 truncate flex items-center gap-1">
                                <User size={10} />
                                Mi Cuenta
                            </span>
                        )}
                    </div>
                    <div className="flex items-center gap-2 mt-1 flex-wrap">
                        <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-surface-700 text-surface-300 capitalize tracking-wide">
                            {group.platform}
                        </span>
                        {group.creator_id === null && group.sources.length === 1 && (
                            <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-brand-500/10 text-brand-400 uppercase tracking-wide">
                                {group.sources[0].source_type}
                            </span>
                        )}
                        {group.creator_id === null && group.sources.length === 1 && group.sources[0].last_checked && (
                            <span className="text-[10px] text-surface-500" title={group.sources[0].last_checked}>
                                Checked {formatRelativeTime(group.sources[0].last_checked)}
                            </span>
                        )}
                    </div>
                    {group.creator_id === null && group.sources.length === 1 && (
                        <p className="text-xs text-surface-400 mt-2 truncate" title={group.sources[0].url}>
                            {group.sources[0].url}
                        </p>
                    )}
                </div>
                {/* Standalone Source Actions */}
                {group.creator_id === null && group.sources.length === 1 && (
                    <div className="flex flex-col items-center gap-1 flex-shrink-0">
                        <button
                            onClick={() => handleToggleActive(group.sources[0])}
                            className={`p-1.5 rounded-lg transition-colors ${group.sources[0].is_active
                                ? 'text-brand-400 hover:text-brand-300'
                                : 'text-surface-500 hover:text-surface-300'
                                }`}
                            title={group.sources[0].is_active ? 'Disable source' : 'Enable source'}
                        >
                            {group.sources[0].is_active ? <ToggleRight size={22} /> : <ToggleLeft size={22} />}
                        </button>
                        <button
                            onClick={() => handleDeleteClick(group.sources[0].id, group.sources[0].name)}
                            className="p-1.5 text-surface-500 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                            title="Delete source"
                        >
                            <Trash2 size={18} />
                        </button>
                    </div>
                )}
            </div>

            {/* Feed Pills Area (Only for Channels & My Accounts) */}
            {group.creator_id !== null && (
                <div className="mt-2 pt-3 border-t border-surface-700/50 flex flex-col gap-2">
                    {group.sources.map(source => (
                        <div 
                            key={source.id} 
                            className={`flex items-center justify-between p-2.5 rounded-lg bg-surface-900 border transition-all ${
                                source.is_active ? 'border-surface-600 hover:border-surface-500' : 'border-surface-800 opacity-60 grayscale-[50%]'
                            }`}
                        >
                            <div className="flex items-center gap-3 min-w-0 flex-1 pr-2">
                                <div className={`p-1.5 rounded-md ${source.is_active ? 'bg-brand-500/10 text-brand-400' : 'bg-surface-800 text-surface-500'}`}>
                                    {getFeedIcon(source, 16)}
                                </div>
                                <div className="flex flex-col min-w-0">
                                    <div className="flex items-center gap-2">
                                        <span className={`text-sm font-medium ${source.is_active ? 'text-surface-100' : 'text-surface-400'}`}>
                                            {source.feed_type ? source.feed_type.charAt(0).toUpperCase() + source.feed_type.slice(1).toLowerCase() : source.source_type}
                                        </span>
                                        {source.last_checked && (
                                            <span className="text-[10px] text-surface-500" title={source.last_checked}>
                                                Checked {formatRelativeTime(source.last_checked)}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-xs text-surface-500 truncate mt-0.5" title={source.url}>
                                        {source.url}
                                    </p>
                                </div>
                            </div>
                            <div className="flex items-center gap-1 flex-shrink-0">
                                <button
                                    onClick={() => handleToggleActive(source)}
                                    className={`p-1 rounded transition-colors ${source.is_active
                                        ? 'text-brand-400 hover:text-brand-300'
                                        : 'text-surface-500 hover:text-surface-400'
                                        }`}
                                >
                                    {source.is_active ? <ToggleRight size={20} /> : <ToggleLeft size={20} />}
                                </button>
                                <button
                                    onClick={() => handleDeleteClick(source.id, `${group.name} - ${source.feed_type || source.source_type}`)}
                                    className="p-1 text-surface-500 hover:text-red-400 hover:bg-red-500/10 rounded transition-colors"
                                >
                                    <Trash2 size={16} />
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );

    if (isLoading) {
        return (
            <div className="flex justify-center items-center h-64 text-surface-400">
                <RefreshCw className="animate-spin" />
                <span className="ml-2">Loading content sources...</span>
            </div>
        );
    }

    return (
        <div className="space-y-6 animate-in fade-in duration-300 pb-10">
            {/* Header */}
            <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold text-surface-100 flex items-center gap-2">
                    <ListVideo className="text-brand-400" />
                    Content Sources
                </h2>
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => setIsAddModalOpen(true)}
                        className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-white bg-brand-600 hover:bg-brand-500 rounded-lg transition-colors shadow-md shadow-brand-600/20"
                    >
                        <Plus size={14} />
                        Add Source
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

            {error && (
                <div className="text-red-500 text-sm bg-red-500/10 border border-red-500/20 px-4 py-3 rounded-lg break-words flex items-center gap-2">
                    <X size={16} onClick={() => setError(null)} className="cursor-pointer flex-shrink-0" />
                    <span>{error}</span>
                </div>
            )}

            {sources.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-64 text-surface-400 space-y-4">
                    <div className="p-5 bg-surface-800 rounded-full border border-surface-700">
                        <Layers size={36} />
                    </div>
                    <p className="text-surface-300">No sources added yet.</p>
                    <button
                        onClick={() => setIsAddModalOpen(true)}
                        className="flex items-center gap-2 px-5 py-2.5 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors shadow-md shadow-brand-600/20"
                    >
                        <Plus size={18} />
                        Add Your First Source
                    </button>
                </div>
            ) : (
                <div className="space-y-8">
                    {/* My Accounts Section */}
                    {myAccounts.length > 0 && (
                        <section className="space-y-4">
                            <h3 className="text-sm font-semibold text-brand-400 uppercase tracking-widest pl-1 border-b border-surface-800 pb-2 flex items-center gap-2">
                                <User size={16} />
                                My Accounts
                            </h3>
                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                {myAccounts.map(renderGroupCard)}
                            </div>
                        </section>
                    )}

                    {/* Channels Section */}
                    {channels.length > 0 && (
                        <section className="space-y-4">
                            <h3 className="text-sm font-semibold text-surface-400 uppercase tracking-widest pl-1 border-b border-surface-800 pb-2 flex items-center gap-2">
                                <ListVideo size={16} />
                                Channels
                            </h3>
                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                {channels.map(renderGroupCard)}
                            </div>
                        </section>
                    )}

                    {/* Standalone Playlists Section */}
                    {standalone.length > 0 && (
                        <section className="space-y-4">
                            <h3 className="text-sm font-semibold text-surface-400 uppercase tracking-widest pl-1 border-b border-surface-800 pb-2 flex items-center gap-2">
                                <Folder size={16} />
                                Playlists & Collections
                            </h3>
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                                {standalone.map(renderGroupCard)}
                            </div>
                        </section>
                    )}
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

            <AddSourceModal 
                isOpen={isAddModalOpen} 
                onClose={() => setIsAddModalOpen(false)} 
                onSuccess={fetchSources}
            />
        </div>
    );
};
