import React, { useState } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { X, Download, Loader2, Search, Image as ImageIcon } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface AddDownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface YtDlpFormat {
  format_id: string;
  url?: string;
  ext?: string;
  width?: number;
  height?: number;
  vcodec?: string;
  acodec?: string;
  filesize?: number;
  filesize_approx?: number;
}

interface YtDlpThumbnail {
  url: string;
  width?: number;
  height?: number;
}

interface YtDlpVideo {
  id: string;
  title: string;
  uploader?: string;
  formats?: YtDlpFormat[];
  thumbnails?: YtDlpThumbnail[];
}

// Minimal type for what the backend returns (supporting Video and Playlist fallback)
interface YtDlpOutput {
  _type?: 'video' | 'playlist';
  // video fields
  id?: string;
  title?: string;
  uploader?: string;
  formats?: YtDlpFormat[];
  thumbnails?: YtDlpThumbnail[];
  // playlist fields
  entries?: any[];
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({ isOpen, onClose }) => {
  const [url, setUrl] = useState('');

  // Phase 1: Fetching metadata
  const [isFetchingInfo, setIsFetchingInfo] = useState(false);
  const [metadata, setMetadata] = useState<YtDlpVideo | null>(null);

  // Phase 2: Selected format string (empty means Best Auto)
  const [selectedFormatId, setSelectedFormatId] = useState<string>('');

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { createDownload } = useDownloadManager();

  // Keep old simple format state for fallback if no formats are returned
  const [fallbackMode, setFallbackMode] = useState(false);
  const [simpleFormat, setSimpleFormat] = useState('best');

  if (!isOpen) return null;

  // Reset state when strictly closing
  const handleClose = () => {
    setUrl('');
    setMetadata(null);
    setSelectedFormatId('');
    setError(null);
    setFallbackMode(false);
    onClose();
  };

  const handleFetchInfo = async () => {
    if (!url.trim()) {
      setError('Please enter a valid URL');
      return;
    }

    setIsFetchingInfo(true);
    setError(null);
    setMetadata(null);
    setFallbackMode(false);
    setSelectedFormatId('');

    try {
      const output = await invoke<YtDlpOutput>('fetch_metadata_command', { url });

      // We only support rich UI for single videos right now.
      // If it's a playlist or lacks formats, we'll use fallback mode.
      const isVideo = output._type === 'video' || (!output._type && output.id);

      if (isVideo && output.formats && output.formats.length > 0) {
        setMetadata(output as YtDlpVideo);
      } else {
        // Fallback to basic mode for playlists or weird outputs
        setFallbackMode(true);
      }
    } catch (err) {
      console.error("Fetch metadata error:", err);
      setError(`Failed to fetch info: ${err}`);
      // Allow fallback mode so user can still force download
      setFallbackMode(true);
    } finally {
      setIsFetchingInfo(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;

    setIsSubmitting(true);
    setError(null);

    let formatOption: string | undefined = undefined;

    if (fallbackMode) {
      formatOption = simpleFormat === 'best' ? undefined : simpleFormat;
    } else if (selectedFormatId) {
      // User picked a specific format ID from the dropdown
      if (selectedFormatId === 'audio-only') {
        // Special keyword we'll interpret as best audio
        formatOption = 'bestaudio';
      } else {
        formatOption = selectedFormatId;
      }
    } // else undefined -> Best Auto

    try {
      await createDownload(url, formatOption);
      handleClose();
    } catch (err) {
      setError('Failed to create download. Please try again.');
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Helper to get Best Thumbnail
  const getBestThumbnail = (video: YtDlpVideo) => {
    if (!video.thumbnails || video.thumbnails.length === 0) return null;
    return video.thumbnails.reduce((best, current) => {
      const currentArea = (current.width || 0) * (current.height || 0);
      const bestArea = (best.width || 0) * (best.height || 0);
      return currentArea > bestArea ? current : best;
    }).url;
  };

  // Helper to process formats for UI
  const getDisplayFormats = (formats?: YtDlpFormat[]) => {
    if (!formats) return [];

    // Filter to formats that have video (vcodec != 'none' and vcodec != null)
    const videoFormats = formats.filter(f => f.vcodec && f.vcodec !== 'none' && f.height);

    // Sort by height descending
    videoFormats.sort((a, b) => (b.height || 0) - (a.height || 0));

    // Deduplicate by height and ext (keep highest bitrate)
    const uniqueFormats: YtDlpFormat[] = [];
    const seen = new Set<string>();

    for (const f of videoFormats) {
      const key = `${f.height}p_${f.ext}`;
      if (!seen.has(key)) {
        uniqueFormats.push(f);
        seen.add(key);
      }
    }

    return uniqueFormats;
  };

  const displayFormats = getDisplayFormats(metadata?.formats);
  const thumbnailUrl = metadata ? getBestThumbnail(metadata) : null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md bg-surface-800 border border-surface-700 rounded-xl shadow-xl p-6 relative animate-in fade-in zoom-in duration-200">
        <button
          onClick={handleClose}
          className="absolute top-4 right-4 text-surface-400 hover:text-surface-100 transition-colors"
        >
          <X size={20} />
        </button>

        <h2 className="text-xl font-semibold text-surface-100 mb-4">Add New Download</h2>

        <div className="space-y-4">
          <div>
            <label htmlFor="url" className="block text-sm font-medium text-surface-400 mb-1">
              Video URL
            </label>
            <div className="flex gap-2">
              <input
                id="url"
                type="text"
                value={url}
                onChange={(e) => {
                  setUrl(e.target.value);
                  // Reset metadata if URL changes to force re-fetch
                  if (metadata || fallbackMode) {
                    setMetadata(null);
                    setFallbackMode(false);
                  }
                }}
                disabled={isFetchingInfo || isSubmitting}
                placeholder="https://youtube.com/watch?v=..."
                className="w-full bg-surface-900 border border-surface-700 rounded-lg px-4 py-2 text-surface-100 placeholder-surface-500 focus:outline-none focus:ring-2 focus:ring-brand-500 disabled:opacity-50"
                autoFocus
              />
              <button
                onClick={handleFetchInfo}
                disabled={!url.trim() || isFetchingInfo || isSubmitting || !!metadata}
                className="flex items-center gap-2 px-4 py-2 bg-surface-700 hover:bg-surface-600 text-surface-100 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed border border-surface-600"
              >
                {isFetchingInfo ? (
                  <Loader2 size={18} className="animate-spin" />
                ) : (
                  <Search size={18} />
                )}
                Fetch Info
              </button>
            </div>
          </div>

          {error && (
            <div className="text-red-500 text-sm bg-red-500/10 border border-red-500/20 px-3 py-2 rounded-lg break-words">
              {error}
            </div>
          )}

          {/* Fallback basic mode UI */}
          {fallbackMode && (
            <div className="animate-in fade-in slide-in-from-top-2">
              <div className="px-4 py-3 bg-surface-900 border border-surface-700 rounded-lg mb-4 text-sm text-surface-300">
                Could not fetch detailed formats. Using basic download mode.
              </div>
              <label className="block text-sm font-medium text-surface-400 mb-1">
                Format
              </label>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={() => setSimpleFormat('best')}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${simpleFormat === 'best'
                    ? 'bg-brand-600 text-white'
                    : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                    }`}
                >
                  Best Quality (Video + Audio)
                </button>
                <button
                  type="button"
                  onClick={() => setSimpleFormat('bestaudio')}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${simpleFormat === 'bestaudio'
                    ? 'bg-brand-600 text-white'
                    : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                    }`}
                >
                  Audio Only
                </button>
              </div>
            </div>
          )}

          {/* Rich Metadata UI */}
          {metadata && !fallbackMode && (
            <div className="animate-in fade-in slide-in-from-top-2 space-y-4">

              {/* Video Details Card */}
              <div className="flex gap-4 p-3 bg-surface-900 border border-surface-700 rounded-xl items-start">
                <div className="w-32 aspect-video bg-surface-800 rounded-lg overflow-hidden flex-shrink-0 relative">
                  {thumbnailUrl ? (
                    <img src={thumbnailUrl} alt="Thumbnail" className="w-full h-full object-cover" />
                  ) : (
                    <div className="absolute inset-0 flex items-center justify-center text-surface-600">
                      <ImageIcon size={24} />
                    </div>
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-medium text-surface-100 line-clamp-2" title={metadata.title}>
                    {metadata.title}
                  </h3>
                  <p className="text-xs text-surface-400 mt-1 line-clamp-1">
                    {metadata.uploader || 'Unknown Channel'}
                  </p>
                </div>
              </div>

              {/* Format Selector */}
              <div>
                <label className="block text-sm font-medium text-surface-400 mb-1">
                  Select Quality
                </label>
                <select
                  value={selectedFormatId}
                  onChange={(e) => setSelectedFormatId(e.target.value)}
                  className="w-full bg-surface-900 border border-surface-700 rounded-lg px-4 py-2 text-sm text-surface-100 focus:outline-none focus:border-brand-500"
                >
                  <option value="">✨ Best Auto (Recommended)</option>
                  <option value="audio-only">🎵 Audio Only (Best)</option>
                  <optgroup label="Video Formats">
                    {displayFormats.map(fmt => {
                      // Formatting filesize
                      let sizeStr = '';
                      const size = fmt.filesize || fmt.filesize_approx;
                      if (size) {
                        sizeStr = ` (~${(size / (1024 * 1024)).toFixed(1)} MB)`;
                      }

                      return (
                        <option key={fmt.format_id} value={`${fmt.format_id}+bestaudio/best`}>
                          {fmt.height}p • {fmt.ext?.toUpperCase()}{sizeStr}
                        </option>
                      );
                    })}
                  </optgroup>
                </select>
                <p className="text-xs text-surface-500 mt-1.5 ml-1">
                  {selectedFormatId === '' ? "Automatically select the highest quality video and audio available." : "Downloads the selected video stream specifically and merges it with the best audio available."}
                </p>
              </div>
            </div>
          )}

          <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-surface-700">
            <button
              type="button"
              onClick={handleClose}
              disabled={isSubmitting}
              className="px-4 py-2 text-surface-400 hover:text-surface-100 transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={isSubmitting || !url.trim() || (!metadata && !fallbackMode)}
              className="flex items-center gap-2 px-6 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? (
                <>
                  <Loader2 size={18} className="animate-spin" />
                  Adding...
                </>
              ) : (
                <>
                  <Download size={18} />
                  Download
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
