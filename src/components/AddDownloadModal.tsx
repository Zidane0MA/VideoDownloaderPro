import React, { useState } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import {
  X, Download, Loader2, Search, Image as ImageIcon,
  ChevronDown, ChevronUp, Music, Subtitles, Film,
  Clock, ListVideo, Settings2, Repeat, LayoutGrid, Check
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type {
  ProcessedMetadata, DownloadOptions
} from '../types/formats';
import { formatFileSize } from '../types/formats';
import { PLATFORM_CONTEXTS, PlatformConfig } from '../features/sources/config/platformContexts';

interface AddDownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({ isOpen, onClose }) => {
  const [url, setUrl] = useState('');

  // Phase 1: Fetching metadata
  const [isFetchingInfo, setIsFetchingInfo] = useState(false);
  const [metadata, setMetadata] = useState<ProcessedMetadata | null>(null);

  // Context Selection (Smart Normalization)
  const [detectedPlatformConfig, setDetectedPlatformConfig] = useState<PlatformConfig | null>(null);
  const [selectedContexts, setSelectedContexts] = useState<Set<string>>(new Set());

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

  // Source Configuration state
  const [limitMode, setLimitMode] = useState<'all' | 'custom'>('all');
  const [maxItems, setMaxItems] = useState<number>(50);
  const [keepActive, setKeepActive] = useState<boolean>(false);

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
    setFallbackMode(false);
    setLimitMode('all');
    setMaxItems(50);
    setKeepActive(false);
    setKeepActive(false);
    setDetectedPlatformConfig(null);
    setSelectedContexts(new Set());
    onClose();
  };

  // Helper to dynamically analyze URL on input change
  const handleUrlChange = (newUrl: string) => {
    setUrl(newUrl);

    if (metadata || fallbackMode) {
      setMetadata(null);
      setFallbackMode(false);
    }

    // Reset context state
    setSelectedContexts(new Set());
    setDetectedPlatformConfig(null);

    const strippedUrl = newUrl.replace(/^https?:\/\//, '').replace(/^www\./, '');
    for (const config of PLATFORM_CONTEXTS) {
      if (config.targetRegex.test(strippedUrl)) {
        setDetectedPlatformConfig(config);
        if (config.options[0] && config.options[0].feedType) {
          // Auto-select the default option if it has a feedType
          setSelectedContexts(new Set([config.options[0].feedType]));
        } else if (config.options[0]) {
          // If no feedType (like standalone actions), use the ID for local state
          setSelectedContexts(new Set([config.options[0].id]));
        }
        break;
      }
    }
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

    let finalUrl = url.trim();

    // Note: for multi-select, we don't automatically mutate the URL if we select multiple
    // We only mutate it if it's a single selection that mutates the probe URL.
    // However, channels use their base URL for probing metadata, so we can just use the base url.
    // If it's TikTok saved/liked, they only have 1 option.
    if (detectedPlatformConfig && selectedContexts.size === 1) {
      const selectedId = Array.from(selectedContexts)[0];
      const opt = detectedPlatformConfig.options.find(o => o.id === selectedId || o.feedType === selectedId);
      if (opt) {
        finalUrl = opt.urlMutator(finalUrl);
      }
    }

    try {
      const output = await invoke<ProcessedMetadata>('fetch_metadata_command', { url: finalUrl });

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

    // If it's a playlist, send the new configuration to `add_source_command`
    if (metadata?.is_playlist) {
      try {
        let finalUrl = url.trim();
        // If it's a single selection that requires mutator (like tiktok/saved), apply it
        if (detectedPlatformConfig && selectedContexts.size === 1) {
          const selectedId = Array.from(selectedContexts)[0];
          const opt = detectedPlatformConfig.options.find(o => o.id === selectedId || o.feedType === selectedId);
          if (opt) {
            finalUrl = opt.urlMutator(finalUrl);
          }
        }

        const feedTypes = Array.from(selectedContexts).filter(id => {
            // Only pass actual FeedTypes to the backend, not generic options without feedType
            const opt = detectedPlatformConfig?.options.find(o => o.id === id || o.feedType === id);
            return opt?.feedType !== undefined;
        });

        await invoke('add_source_command', {
          request: {
              url: finalUrl,
              feed_types: feedTypes.length > 0 ? feedTypes : null,
              selected_ids: null
          }
        });
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
                onChange={(e) => handleUrlChange(e.target.value)}
                disabled={isFetchingInfo || isSubmitting}
                placeholder="https://youtube.com/watch?v=..."
                className="w-full bg-surface-900 border border-surface-700 rounded-lg px-4 py-2 text-surface-100 placeholder-surface-500 focus:outline-none focus:ring-2 focus:ring-brand-500 disabled:opacity-50"
                autoFocus
              />
              <button
                onClick={handleFetchInfo}
                disabled={!url.trim() || isFetchingInfo || isSubmitting || !!metadata}
                className="flex items-center gap-2 px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-md shadow-brand-600/20"
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

          {/* Context Selector UI */}
          {detectedPlatformConfig && !metadata && !fallbackMode && (
            <div className="animate-in fade-in slide-in-from-top-2 pt-2">
              <label className="block text-xs font-semibold text-brand-400 mb-2 uppercase tracking-wider">
                What do you want to download?
              </label>

              <div className="grid grid-cols-3 gap-2">
                {detectedPlatformConfig.options.map(option => {
                  const val = option.feedType || option.id;
                  const isSelected = selectedContexts.has(val);

                  let activeClasses = "";
                  if (option.colorClass === 'brand') activeClasses = "bg-brand-600/15 border-brand-500 text-brand-300 shadow-sm shadow-brand-500/10";
                  else if (option.colorClass === 'pink') activeClasses = "bg-pink-500/15 border-pink-500 text-pink-400 shadow-sm shadow-pink-500/10";
                  else if (option.colorClass === 'amber') activeClasses = "bg-amber-500/15 border-amber-500 text-amber-400 shadow-sm shadow-amber-500/10";

                  const inactiveClasses = "bg-surface-900 border-surface-700 text-surface-400 hover:border-surface-600 hover:text-surface-200 hover:bg-surface-800";

                  return (
                    <button
                      key={option.id}
                      onClick={() => {
                          setSelectedContexts(prev => {
                              const next = new Set(prev);
                              // Toggling logic
                              if (next.has(val)) next.delete(val);
                              else next.add(val);
                              return next;
                          });
                      }}
                      className={`flex flex-col items-center justify-center p-3 rounded-xl border transition-all duration-200 ${isSelected ? activeClasses : inactiveClasses} relative`}
                    >
                      {isSelected && (
                          <div className="absolute top-1.5 right-1.5">
                              <Check size={12} className="stroke-[3]" />
                          </div>
                      )}
                      <option.icon size={20} className="mb-1.5" />
                      <span className="text-[11px] font-medium">{option.label}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

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

              {/* ── Source Configuration ── */}
              {metadata.is_playlist ? (
                <div className="space-y-4">
                  {/* Notice */}
                  <div className="p-4 bg-brand-500/10 border border-brand-500/20 rounded-lg flex items-start gap-3">
                    <LayoutGrid className="text-brand-400 mt-0.5" size={20} />
                    <div className="flex-1">
                      <h4 className="text-sm font-medium text-brand-300">New Source Detected</h4>
                      <p className="text-xs text-brand-400/80 mt-1 leading-relaxed">
                        Configure how this collection should be processed. Added sources can be managed from your backend settings.
                      </p>
                    </div>
                  </div>

                  {/* Auto-detected Source Info */}
                  <div className="bg-surface-900 border border-surface-700/50 p-4 rounded-lg flex items-start gap-4">
                    <div className="p-2.5 bg-brand-500/10 rounded-lg text-brand-400">
                      <ListVideo size={24} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium text-surface-100 truncate" title={metadata.title}>
                        {metadata.title || 'Unknown Collection'}
                      </h4>
                      <div className="flex items-center gap-3 mt-1.5 text-xs text-surface-400">
                        {metadata.uploader && <span>{metadata.uploader}</span>}
                        {metadata.playlist_entries && (
                          <span className="flex items-center gap-1">
                            • {metadata.playlist_entries.length} items detected
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Settings Grid */}
                  <div className="grid grid-cols-1 gap-3">
                    {/* Keep Active Toggle */}
                    <div className="bg-surface-900 border border-surface-700/50 p-3.5 rounded-lg flex items-center justify-between cursor-pointer" onClick={() => setKeepActive(!keepActive)}>
                      <div className="flex flex-col gap-0.5">
                        <label className="text-sm font-medium text-surface-200 cursor-pointer flex items-center gap-1.5">
                          <Repeat size={14} className={keepActive ? 'text-brand-400' : 'text-surface-500'} />
                          Keep Source Active
                        </label>
                        <p className="text-[11px] text-surface-500">
                          Continues to monitor this source for new uploads in the background.
                        </p>
                      </div>

                      {/* Custom Toggle Switch */}
                      <div className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-brand-500 focus:ring-offset-2 focus:ring-offset-surface-900 ${keepActive ? 'bg-brand-500' : 'bg-surface-600'}`}>
                        <span className={`pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${keepActive ? 'translate-x-4' : 'translate-x-0'}`} />
                      </div>
                    </div>

                    {/* Fetch Limit */}
                    <div className="bg-surface-900 border border-surface-700/50 p-3 rounded-lg flex flex-col gap-2">
                      <label className="text-xs font-medium text-surface-400 flex items-center gap-1.5 mb-1">
                        <Settings2 size={14} /> Fetch Limit
                      </label>
                      <div className="flex flex-col gap-2.5 px-1">
                        <label className="flex items-center gap-2 text-sm text-surface-200 cursor-pointer">
                          <input
                            type="radio"
                            name="limitMode"
                            checked={limitMode === 'all'}
                            onChange={() => setLimitMode('all')}
                            className="text-brand-500 bg-surface-800 border-surface-600 focus:ring-brand-500 rounded-full"
                          />
                          Fetch all available items
                        </label>
                        <label className="flex items-center gap-2 text-sm text-surface-200 cursor-pointer">
                          <input
                            type="radio"
                            name="limitMode"
                            checked={limitMode === 'custom'}
                            onChange={() => setLimitMode('custom')}
                            className="text-brand-500 bg-surface-800 border-surface-600 focus:ring-brand-500 rounded-full"
                          />
                          Limit to recent items
                        </label>
                        {limitMode === 'custom' && (
                          <div className="ml-6 flex items-center gap-2 mt-0.5 animate-in fade-in slide-in-from-top-1">
                            <input
                              type="number"
                              min="1"
                              value={maxItems}
                              onChange={e => setMaxItems(Number(e.target.value))}
                              className="w-20 bg-surface-800 border border-surface-600 rounded text-sm text-surface-200 py-1 px-2 focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
                            />
                            <span className="text-xs text-surface-500">items maximum</span>
                          </div>
                        )}
                      </div>
                    </div>
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
                {metadata?.is_playlist
                  ? 'Add Source'
                  : (audioOnly ? 'Extract Audio' : 'Download')}
              </>
            )}

          </button>
        </div>
      </div>
    </div>
  );
};
