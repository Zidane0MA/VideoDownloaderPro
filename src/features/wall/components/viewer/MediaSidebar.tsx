import { ExternalLink, FolderOpen, Trash2, Calendar, FileType, HardDrive, RotateCcw } from 'lucide-react';
import type { Post, Media } from '../../../../types/wall';
import { revealInExplorer, deletePost } from '../../api/viewer';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { ConfirmModal } from '../../../../components/ui/ConfirmModal';

interface MediaSidebarProps {
    post: Post;
    media: Media;
    onClose: () => void;
    isTrashMode?: boolean;
}

export function MediaSidebar({ post, media, onClose, isTrashMode }: MediaSidebarProps) {
    const [isDeleting, setIsDeleting] = useState(false);
    const [isRestoring, setIsRestoring] = useState(false);
    const [isConfirmOpen, setIsConfirmOpen] = useState(false);

    const handleReveal = async () => {
        try {
            await revealInExplorer(media.file_path);
        } catch (error) {
            console.error('Failed to reveal file:', error);
        }
    };

    const handleDeleteClick = () => {
        setIsConfirmOpen(true);
    };

    const handleConfirmDelete = async () => {
        setIsConfirmOpen(false);
        setIsDeleting(true);
        try {
            await deletePost(post.id);
            onClose(); // Close viewer after deletion
        } catch (error: any) {
            console.error('Failed to delete post:', error);
            setIsDeleting(false);
        }
    };

    const handleRestore = async () => {
        setIsRestoring(true);
        try {
            await invoke('restore_post', { postId: post.id });
            onClose(); // Close viewer after restoring
        } catch (error) {
            console.error('Failed to restore post:', error);
            setIsRestoring(false);
        }
    };

    // Format bytes to human readable
    const formatBytes = (bytes: number | null) => {
        if (!bytes) return 'Unknown';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
    };

    const formatDate = (dateStr: string | null) => {
        if (!dateStr) return 'Unknown';
        return new Date(dateStr).toLocaleString();
    };

    return (
        <div className="w-80 md:w-96 flex flex-col bg-surface-900 border-l border-surface-800 h-full overflow-y-auto custom-scrollbar">
            {/* Creator Header */}
            <div className="p-4 border-b border-surface-800 flex items-center gap-3">
                {post.creator_avatar ? (
                    <img src={post.creator_avatar} alt="Creator" className="w-12 h-12 rounded-full object-cover ring-2 ring-surface-700" />
                ) : (
                    <div className="w-12 h-12 rounded-full bg-surface-700 flex items-center justify-center font-bold text-surface-400">
                        {(post.creator_name || post.creator_id || '?')[0].toUpperCase()}
                    </div>
                )}
                <div className="flex-1 overflow-hidden">
                    <h3 className="font-semibold text-surface-100 truncate">{post.creator_name || 'Unknown Creator'}</h3>
                    <p className="text-sm text-surface-400 truncate">{post.creator_handle || post.creator_id}</p>
                </div>
            </div>

            {/* Post Content */}
            <div className="p-4 flex-1">
                {post.title && (
                    <h2 className="text-lg font-semibold text-surface-100 mb-2 leading-snug">{post.title}</h2>
                )}
                {post.description && (
                    <p className="text-sm text-surface-300 whitespace-pre-wrap leading-relaxed">
                        {post.description}
                    </p>
                )}
            </div>

            {/* Metadata Section */}
            <div className="p-4 border-t border-surface-800 bg-surface-800/30">
                <h4 className="text-xs font-semibold text-surface-400 uppercase tracking-wider mb-3">File Information</h4>
                <div className="space-y-2 text-sm text-surface-300">
                    <div className="flex items-center gap-2">
                        <FileType size={16} className="text-surface-500" />
                        <span>{media.width && media.height ? `${media.width} x ${media.height}` : 'Unknown Resolution'}</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <HardDrive size={16} className="text-surface-500" />
                        <span>{formatBytes(media.file_size)}</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <Calendar size={16} className="text-surface-500" />
                        <span className="truncate">{formatDate(post.downloaded_at)}</span>
                    </div>
                </div>
            </div>

            {/* Actions */}
            <div className="p-4 space-y-2 border-t border-surface-800">
                <a
                    href={post.original_url}
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center justify-center gap-2 w-full px-4 py-2 bg-brand-500/10 hover:bg-brand-500/20 text-brand-400 rounded-lg transition-colors font-medium text-sm"
                >
                    <ExternalLink size={16} />
                    View Original Post
                </a>

                <button
                    onClick={handleReveal}
                    className="flex items-center justify-center gap-2 w-full px-4 py-2 bg-surface-700 hover:bg-surface-600 text-surface-100 rounded-lg transition-colors font-medium text-sm"
                >
                    <FolderOpen size={16} />
                    Show in Explorer
                </button>

                {isTrashMode ? (
                    <>
                        <button
                            onClick={handleRestore}
                            disabled={isRestoring || isDeleting}
                            className="flex items-center justify-center gap-2 w-full px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg transition-colors font-medium text-sm disabled:opacity-50"
                        >
                            <RotateCcw size={16} />
                            {isRestoring ? 'Restoring...' : 'Restore'}
                        </button>
                    </>
                ) : (
                    <button
                        onClick={handleDeleteClick}
                        disabled={isDeleting}
                        className="flex items-center justify-center gap-2 w-full px-4 py-2 border border-red-500/20 hover:bg-red-500/10 text-red-500 rounded-lg transition-colors font-medium text-sm disabled:opacity-50"
                    >
                        <Trash2 size={16} />
                        {isDeleting ? 'Deleting...' : 'Delete File'}
                    </button>
                )}
            </div>

            <ConfirmModal
                isOpen={isConfirmOpen}
                title="Delete File"
                message="Are you sure you want to delete this file?\nThis action cannot be undone."
                onConfirm={handleConfirmDelete}
                onCancel={() => setIsConfirmOpen(false)}
                confirmText="Delete"
                isDanger={true}
            />
        </div>
    );
}
