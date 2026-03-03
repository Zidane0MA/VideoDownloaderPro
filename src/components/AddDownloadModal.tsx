import React, { useState } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import {
  X, Download, Loader2, Search, Image as ImageIcon,
  ChevronDown, ChevronUp, Music, Subtitles, Film,
  Check, Clock, ListVideo
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type {
  ProcessedMetadata, DownloadOptions
} from '../types/formats';
import { formatFileSize } from '../types/formats';

interface AddDownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({ isOpen, onClose }) => {
  const [url, setUrl] = useState('');

  // Phase 1: Fetching metadata
  const [isFetchingInfo, setIsFetchingInfo] = useState(false);
  const [metadata, setMetadata] = useState<ProcessedMetadata | null>(null);

  // Phase 2: Selection state
  const [selectedVideoId, setSelectedVideoId] = useState<string>(''); // '' = Best Auto
  const [selectedAudioId, setSelectedAudioId] = useState<string>(''); // '' = Best Auto
  const [selectedSubtitles, setSelectedSubtitles] = useState<Set<string>>(new Set());
  const [embedSubs, setEmbedSubs] = useState(true);
  const [selectedContainer, setSelectedContainer] = useState<string>(''); // '' = Auto
  const [audioOnly, setAudioOnly] = useState(false);
  const [audioExtractFormat, setAudioExtractFormat] = useState('mp3');
  const [showAdvanced, setShowAdvanced] = useState(false);

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { createDownload } = useDownloadManager();

  // Fallback mode when metadata fetch fails
  const [fallbackMode, setFallbackMode] = useState(false);
  const [simpleFormat, setSimpleFormat] = useState('best');

  if (!isOpen) return null;

  const handleClose = () => {
    setUrl('');
    setMetadata(null);
    setSelectedVideoId('');
    setSelectedAudioId('');
    setSelectedSubtitles(new Set());
    setEmbedSubs(true);
    setSelectedContainer('');
    setAudioOnly(false);
    setAudioExtractFormat('mp3');
    setShowAdvanced(false);
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
    setSelectedVideoId('');
    setSelectedAudioId('');
    setSelectedSubtitles(new Set());
    setAudioOnly(false);

    try {
      const output = await invoke<ProcessedMetadata>('fetch_metadata_command', { url });

      if (output.is_playlist || output.video_qualities.length > 0 || output.audio_tracks.length > 0) {
        setMetadata(output);
      } else {
        // No formats available and not a playlist — fallback
        setFallbackMode(true);
      }
    } catch (err) {
      console.error("Fetch metadata error:", err);
      setError(`Failed to fetch info: ${err}`);
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

    // If it's a playlist, we send it to `add_source_command`
    if (metadata?.is_playlist) {
      try {
        await invoke('add_source_command', { url });
        handleClose();
      } catch (err) {
        setError('Failed to add playlist source. ' + err);
        console.error(err);
      } finally {
        setIsSubmitting(false);
      }
      return;
    }

    let formatOption: string | undefined = undefined;

    if (fallbackMode) {
      formatOption = simpleFormat === 'best' ? undefined : simpleFormat;
    } else if (metadata) {
      // Build DownloadOptions JSON
      const opts: DownloadOptions = {
        format_id: selectedVideoId || undefined,
        audio_format_id: selectedAudioId || undefined,
        audio_only: audioOnly,
        audio_extract_format: audioOnly ? audioExtractFormat : undefined,
        subtitle_langs: Array.from(selectedSubtitles),
        embed_subs: selectedSubtitles.size > 0 && embedSubs,
        container: selectedContainer || undefined,
      };
      formatOption = JSON.stringify(opts);
    }

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

  const toggleSubtitle = (langCode: string) => {
    setSelectedSubtitles(prev => {
      const next = new Set(prev);
      if (next.has(langCode)) next.delete(langCode);
      else next.add(langCode);
      return next;
    });
  };

  // Format duration to human readable
  const formatDuration = (seconds: number | null) => {
    if (!seconds) return '';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    return `${m}:${s.toString().padStart(2, '0')}`;
  };

  // Quality presets
  const presets = [
    { label: 'Best', id: '', icon: '✨' },
    { label: '1080p', id: metadata?.video_qualities.find(q => q.height === 1080)?.format_id || '', height: 1080 },
    { label: '720p', id: metadata?.video_qualities.find(q => q.height === 720)?.format_id || '', height: 720 },
    { label: '480p', id: metadata?.video_qualities.find(q => q.height === 480)?.format_id || '', height: 480 },
  ].filter(p => p.id !== undefined);

  const hasSubtitles = metadata && metadata.subtitle_tracks.length > 0;
  const hasMultipleAudio = metadata && metadata.audio_tracks.length > 1;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-lg bg-surface-800 border border-surface-700 rounded-xl shadow-xl relative animate-in fade-in zoom-in duration-200 max-h-[85vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 pb-0">
          <h2 className="text-xl font-semibold text-surface-100">Add New Download</h2>
          <button
            onClick={handleClose}
            className="text-surface-400 hover:text-surface-100 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Scrollable Content */}
        <div className="p-6 space-y-4 overflow-y-auto flex-1">
          {/* URL Input */}
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
                Fetch
              </button>
            </div>
          </div>

          {/* Error */}
          {error && (
            <div className="text-red-500 text-sm bg-red-500/10 border border-red-500/20 px-3 py-2 rounded-lg break-words">
              {error}
            </div>
          )}

          {/* Fallback Mode */}
          {fallbackMode && (
            <div className="animate-in fade-in slide-in-from-top-2">
              <div className="px-4 py-3 bg-surface-900 border border-surface-700 rounded-lg mb-4 text-sm text-surface-300">
                Could not fetch detailed formats. Using basic download mode.
              </div>
              <label className="block text-sm font-medium text-surface-400 mb-1">Format</label>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={() => setSimpleFormat('best')}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${simpleFormat === 'best'
                    ? 'bg-brand-600 text-white'
                    : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                    }`}
                >
                  Best Quality
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
                  {metadata.thumbnail_url ? (
                    <img src={metadata.thumbnail_url} alt="Thumbnail" className="w-full h-full object-cover" />
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
                  {metadata.duration && (
                    <div className="flex items-center gap-1 text-xs text-surface-500 mt-1">
                      <Clock size={12} />
                      {formatDuration(metadata.duration)}
                    </div>
                  )}
                </div>
              </div>

              {/* ── Playlist Notice OR Video Options ── */}
              {metadata.is_playlist ? (
                <div className="p-4 bg-brand-500/10 border border-brand-500/20 rounded-lg flex items-start gap-3 mt-4">
                  <ListVideo className="text-brand-400 mt-0.5" size={20} />
                  <div>
                    <h4 className="text-sm font-medium text-brand-300">Playlist Detected</h4>
                    <p className="text-xs text-brand-400/80 mt-1">
                      This will be added as a Source. All contained videos will be automatically queued for download.
                    </p>
                  </div>
                </div>
              ) : (
                <>
                  {/* ── Mode Toggle: Video / Audio Only ── */}
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => setAudioOnly(false)}
                      className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all ${!audioOnly
                        ? 'bg-brand-600 text-white shadow-lg shadow-brand-600/20'
                        : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                        }`}
                    >
                      <Film size={16} /> Video
                    </button>
                    <button
                      type="button"
                      onClick={() => setAudioOnly(true)}
                      className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all ${audioOnly
                        ? 'bg-brand-600 text-white shadow-lg shadow-brand-600/20'
                        : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                        }`}
                    >
                      <Music size={16} /> Audio Only
                    </button>
                  </div>

                  {/* ── Video Quality Section ── */}
                  {!audioOnly && metadata.video_qualities.length > 0 && (
                    <div>
                      <label className="block text-sm font-medium text-surface-400 mb-2">
                        <Film size={14} className="inline mr-1.5 -mt-0.5" />
                        Video Quality
                      </label>

                      {/* Quick Presets */}
                      <div className="flex gap-2 mb-2">
                        {presets.map(p => {
                          const isAvailable = p.label === 'Best' || metadata.video_qualities.some(q => q.height === p.height);
                          if (!isAvailable && p.label !== 'Best') return null;
                          const isActive = p.label === 'Best' ? selectedVideoId === '' : selectedVideoId === p.id;
                          return (
                            <button
                              key={p.label}
                              type="button"
                              onClick={() => setSelectedVideoId(p.label === 'Best' ? '' : p.id)}
                              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${isActive
                                ? 'bg-brand-600 text-white'
                                : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700 hover:text-surface-200'
                                }`}
                            >
                              {p.icon && <span className="mr-1">{p.icon}</span>}
                              {p.label}
                            </button>
                          );
                        })}
                      </div>

                      {/* Advanced Format List (collapsible) */}
                      <button
                        type="button"
                        onClick={() => setShowAdvanced(!showAdvanced)}
                        className="flex items-center gap-1 text-xs text-surface-500 hover:text-surface-300 transition-colors mb-1"
                      >
                        {showAdvanced ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                        {showAdvanced ? 'Hide' : 'Show'} all formats ({metadata.video_qualities.length})
                      </button>

                      {showAdvanced && (
                        <div className="max-h-48 overflow-y-auto rounded-lg border border-surface-700 bg-surface-900">
                          {metadata.video_qualities.map(q => {
                            const isActive = selectedVideoId === q.format_id;
                            return (
                              <button
                                key={q.format_id}
                                type="button"
                                onClick={() => setSelectedVideoId(q.format_id)}
                                className={`w-full flex items-center justify-between px-3 py-2 text-xs transition-colors border-b border-surface-700/50 last:border-b-0 ${isActive
                                  ? 'bg-brand-600/15 text-brand-300'
                                  : 'text-surface-300 hover:bg-surface-800'
                                  }`}
                              >
                                <div className="flex items-center gap-2">
                                  {isActive && <Check size={12} className="text-brand-400" />}
                                  <span className="font-medium">{q.height}p</span>
                                  {q.fps && q.fps > 48 && (
                                    <span className="px-1.5 py-0.5 bg-amber-500/15 text-amber-400 rounded text-[10px]">
                                      {Math.round(q.fps)}fps
                                    </span>
                                  )}
                                  {q.dynamic_range !== 'SDR' && (
                                    <span className="px-1.5 py-0.5 bg-purple-500/15 text-purple-400 rounded text-[10px]">
                                      {q.dynamic_range}
                                    </span>
                                  )}
                                  <span className="text-surface-500">{q.vcodec}</span>
                                  <span className="text-surface-600">{q.container}</span>
                                </div>
                                {q.filesize && (
                                  <span className="text-surface-500 ml-2">{formatFileSize(q.filesize)}</span>
                                )}
                              </button>
                            );
                          })}
                        </div>
                      )}

                      {/* Info text */}
                      <p className="text-[11px] text-surface-500 mt-1.5 ml-0.5">
                        {selectedVideoId === ''
                          ? 'Automatically selects the highest quality available.'
                          : `Selected format will be merged with the best audio track.`}
                      </p>
                    </div>
                  )}

                  {/* ── Audio Format (Audio Only mode) ── */}
                  {audioOnly && (
                    <div>
                      <label className="block text-sm font-medium text-surface-400 mb-2">
                        Output Format
                      </label>
                      <div className="flex gap-2">
                        {['mp3', 'opus', 'm4a', 'flac'].map(fmt => (
                          <button
                            key={fmt}
                            type="button"
                            onClick={() => setAudioExtractFormat(fmt)}
                            className={`px-3 py-1.5 rounded-lg text-xs font-medium uppercase transition-all ${audioExtractFormat === fmt
                              ? 'bg-brand-600 text-white'
                              : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                              }`}
                          >
                            {fmt}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* ── Audio Track Selector ── */}
                  {hasMultipleAudio && !audioOnly && (
                    <div>
                      <label className="block text-sm font-medium text-surface-400 mb-2">
                        <Music size={14} className="inline mr-1.5 -mt-0.5" />
                        Audio Track
                      </label>
                      <select
                        value={selectedAudioId}
                        onChange={(e) => setSelectedAudioId(e.target.value)}
                        className="w-full bg-surface-900 border border-surface-700 rounded-lg px-3 py-2 text-sm text-surface-100 focus:outline-none focus:border-brand-500"
                      >
                        <option value="">Best Auto</option>
                        {metadata.audio_tracks.map(a => (
                          <option key={a.format_id} value={a.format_id}>
                            {a.label}{a.filesize ? ` (${formatFileSize(a.filesize)})` : ''}
                          </option>
                        ))}
                      </select>
                    </div>
                  )}

                  {/* ── Subtitles ── */}
                  {hasSubtitles && !audioOnly && (
                    <div>
                      <label className="block text-sm font-medium text-surface-400 mb-2">
                        <Subtitles size={14} className="inline mr-1.5 -mt-0.5" />
                        Subtitles
                      </label>
                      <div className="max-h-32 overflow-y-auto rounded-lg border border-surface-700 bg-surface-900">
                        {metadata.subtitle_tracks.map(sub => {
                          const isChecked = selectedSubtitles.has(sub.language_code);
                          return (
                            <label
                              key={sub.language_code}
                              className={`flex items-center gap-3 px-3 py-2 text-xs cursor-pointer transition-colors border-b border-surface-700/50 last:border-b-0 ${isChecked ? 'bg-brand-600/10' : 'hover:bg-surface-800'
                                }`}
                            >
                              <input
                                type="checkbox"
                                checked={isChecked}
                                onChange={() => toggleSubtitle(sub.language_code)}
                                className="rounded border-surface-600 bg-surface-900 text-brand-500 focus:ring-brand-500 focus:ring-offset-0"
                              />
                              <span className={isChecked ? 'text-surface-100' : 'text-surface-300'}>
                                {sub.label}
                              </span>
                              {sub.is_auto_generated && (
                                <span className="px-1.5 py-0.5 bg-surface-700 text-surface-400 rounded text-[10px]">
                                  auto
                                </span>
                              )}
                            </label>
                          );
                        })}
                      </div>
                      {selectedSubtitles.size > 0 && (
                        <label className="flex items-center gap-2 mt-2 text-xs text-surface-400 cursor-pointer">
                          <input
                            type="checkbox"
                            checked={embedSubs}
                            onChange={(e) => setEmbedSubs(e.target.checked)}
                            className="rounded border-surface-600 bg-surface-900 text-brand-500 focus:ring-brand-500 focus:ring-offset-0"
                          />
                          Embed subtitles in video file
                        </label>
                      )}
                    </div>
                  )}

                  {/* ── Container Override ── */}
                  {!audioOnly && metadata.video_qualities.length > 0 && (
                    <div>
                      <label className="block text-sm font-medium text-surface-400 mb-2">
                        Container
                      </label>
                      <div className="flex gap-2">
                        {['', 'mp4', 'mkv', 'webm'].map(c => (
                          <button
                            key={c}
                            type="button"
                            onClick={() => setSelectedContainer(c)}
                            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${selectedContainer === c
                              ? 'bg-brand-600 text-white'
                              : 'bg-surface-900 border border-surface-700 text-surface-400 hover:bg-surface-700'
                              }`}
                          >
                            {c === '' ? 'Auto' : c.toUpperCase()}
                          </button>
                        ))}
                      </div>
                      <p className="text-[11px] text-surface-500 mt-1">
                        {selectedContainer === '' ? 'Automatic container based on codec compatibility.' : `Force output as .${selectedContainer} (ffmpeg will remux if needed).`}
                      </p>
                    </div>
                  )}
                </>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 p-6 pt-4 border-t border-surface-700">
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
            className="flex items-center gap-2 px-6 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-brand-600/20"
          >
            {isSubmitting ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                Adding...
              </>
            ) : (
              <>
                <Download size={18} />
                {metadata?.is_playlist ? 'Add Source & Queue' : (audioOnly ? 'Extract Audio' : 'Download')}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
