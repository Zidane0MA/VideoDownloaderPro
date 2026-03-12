import React, { useState, useEffect } from 'react';
import { X, Plus, Loader2, Download } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { PLATFORM_CONTEXTS, PlatformConfig } from '../config/platformContexts';
import { listen } from '@tauri-apps/api/event';
import type { PlatformSession } from '../../../types/auth';

interface AddSourceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export const AddSourceModal: React.FC<AddSourceModalProps> = ({ isOpen, onClose, onSuccess }) => {
  const [url, setUrl] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Account-Centric Hub State
  const [authStatus, setAuthStatus] = useState<PlatformSession[]>([]);
  const [dbSources, setDbSources] = useState<any[]>([]); // Using any for SourceResponse
  const [selectedHubPlatform, setSelectedHubPlatform] = useState<string | null>(null);
  const [hubSelectedFeeds, setHubSelectedFeeds] = useState<Set<string>>(new Set());

  // Context Selection (Smart Normalization)
  const [detectedPlatformConfig, setDetectedPlatformConfig] = useState<PlatformConfig | null>(null);
  const [selectedContexts, setSelectedContexts] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (isOpen) {
      fetchHubData();
      const unlisten = listen('session-status-changed', fetchHubData);
      return () => {
        unlisten.then(f => f());
      };
    }
  }, [isOpen]);

  const fetchHubData = async () => {
    try {
      const [auth, sources] = await Promise.all([
        invoke<PlatformSession[]>('get_auth_status'),
        invoke<any[]>('get_sources_command')
      ]);
      setAuthStatus(auth);
      setDbSources(sources);
    } catch (err) {
      console.error("Failed to fetch hub data:", err);
    }
  };

  if (!isOpen) return null;

  const handleClose = () => {
    setUrl('');
    setError(null);
    setSelectedHubPlatform(null);
    setHubSelectedFeeds(new Set());
    setDetectedPlatformConfig(null);
    setSelectedContexts(new Set());
    onClose();
  };

  const handleUrlChange = (newUrl: string) => {
    setUrl(newUrl);
    setSelectedContexts(new Set());
    setDetectedPlatformConfig(null);

    const strippedUrl = newUrl.replace(/^https?:\/\//, '').replace(/^www\./, '');
    for (const config of PLATFORM_CONTEXTS) {
      if (config.targetRegex.test(strippedUrl)) {
        setDetectedPlatformConfig(config);
        if (config.options[0] && config.options[0].feedType) {
          setSelectedContexts(new Set([config.options[0].feedType]));
        } else if (config.options[0]) {
          setSelectedContexts(new Set([config.options[0].id]));
        }
        break;
      }
    }
  };

  const handleSubmit = async () => {
    if (!url.trim()) return;

    setIsSubmitting(true);
    setError(null);

    try {
      let finalUrl = url.trim();
      let feedTypes: string[] | null = null;
      
      // If a standard URL with selected contexts
      if (detectedPlatformConfig) {
        if (selectedContexts.size === 1) {
            const selectedId = Array.from(selectedContexts)[0];
            const opt = detectedPlatformConfig.options.find((o: any) => o.id === selectedId || o.feedType === selectedId);
            if (opt) {
                finalUrl = opt.urlMutator(finalUrl);
            }
        }
        feedTypes = Array.from(selectedContexts).filter(id => {
            const opt = detectedPlatformConfig?.options.find((o: any) => o.id === id || o.feedType === id);
            return opt?.feedType !== undefined;
        });
        if (feedTypes.length === 0) feedTypes = null;
      }

      const results = await invoke<{ source_id: number; items_queued: number }[]>('add_source_command', {
        request: {
            url: finalUrl,
            feed_types: feedTypes,
            selected_ids: null
        }
      });
      
      const totalQueued = results.reduce((acc, curr) => acc + curr.items_queued, 0);
      console.info(`${results.length} sources added, ${totalQueued} items queued`);
      
      if (onSuccess) onSuccess();
      handleClose();
    } catch (err: any) {
      setError('Failed to add source. ' + err);
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const toggleContext = (contextId: string) => {
    // For standalone actions (like default add source) that shouldn't be mutli-selected, 
    // we could enforce single select. But here we mimic AddDownloadModal's behavior.
    if (detectedPlatformConfig?.options.find((o: any) => o.id === contextId)?.feedType === undefined) {
      setSelectedContexts(new Set([contextId]));
      return;
    }

    setSelectedContexts(prev => {
      const next = new Set(prev);
      if (next.has(contextId)) next.delete(contextId);
      else next.add(contextId);
      return next;
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-lg bg-surface-800 border border-surface-700 rounded-xl shadow-xl relative animate-in fade-in zoom-in duration-200">
        
        {/* Header */}
        <div className="flex items-center justify-between p-6 pb-0 mb-4">
          <h2 className="text-xl font-semibold text-surface-100 flex items-center gap-2">
            <Plus className="text-brand-400" />
            Add New Source
          </h2>
          <button
            onClick={handleClose}
            className="text-surface-400 hover:text-surface-100 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 pt-0 space-y-4">
          
          {/* URL Input */}
          <div>
            <label htmlFor="modal-url" className="block text-sm font-medium text-surface-400 mb-1">
              Source URL
            </label>
            <div className="flex gap-2">
              <input
                id="modal-url"
                type="text"
                value={url}
                onChange={(e) => handleUrlChange(e.target.value)}
                onKeyDown={(e) => {
                    if (e.key === 'Enter' && url.trim() && !isSubmitting) handleSubmit();
                }}
                disabled={isSubmitting}
                placeholder="https://youtube.com/playlist?list=..."
                className="w-full bg-surface-900 border border-surface-700 rounded-lg px-4 py-2 text-surface-100 placeholder-surface-500 focus:outline-none focus:ring-2 focus:ring-brand-500 disabled:opacity-50"
                autoFocus
              />
              {url.trim() && (
                 <button
                   onClick={handleSubmit}
                   disabled={isSubmitting}
                   className="flex items-center gap-2 px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-md shadow-brand-600/20"
                 >
                   {isSubmitting ? <Loader2 size={18} className="animate-spin" /> : <Plus size={18} />}
                   Add
                 </button>
              )}
            </div>
          </div>

          {/* Account-Centric Hub */}
          {!url.trim() && (
            <div className="animate-in fade-in slide-in-from-top-2 pt-2">
              <div className="flex items-center justify-between mb-4">
                <label className="text-xs font-semibold text-surface-400 uppercase tracking-wider">
                  Your Connected Accounts <span className="ml-1 px-1.5 py-0.5 rounded-full bg-brand-500/10 text-brand-400 text-[9px] border border-brand-500/20">PRO</span>
                </label>
              </div>
              <div className="flex gap-3 mb-5 overflow-x-auto pb-2 custom-scrollbar">
                {PLATFORM_CONTEXTS.map((platform: any) => {
                  const session = authStatus.find(s => s.platform_id === platform.id);
                  const isActive = session?.status === 'ACTIVE';
                  const isSelected = selectedHubPlatform === platform.id;
                  
                  return (
                    <div 
                      key={platform.id}
                      onClick={() => {
                        if (isActive) {
                          setSelectedHubPlatform(isSelected ? null : platform.id);
                          setHubSelectedFeeds(new Set()); 
                        } else {
                          invoke('open_login_window', { platform_id: platform.id });
                        }
                      }}
                      className={`relative w-14 h-14 rounded-full border-2 cursor-pointer transition-all shrink-0 bg-surface-800 flex items-center justify-center ${
                        isSelected ? 'border-brand-500 shadow-md shadow-brand-500/20 scale-105' : 
                        isActive ? 'border-transparent hover:scale-105' : 'border-dashed border-surface-600 hover:border-surface-500 opacity-60 hover:opacity-100'
                      }`}
                      style={isActive && session?.avatar_url ? { backgroundImage: `url(${session.avatar_url})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}}
                    >
                      {(!isActive || !session?.avatar_url) && (
                         isActive ? <span className="text-xl font-bold">{platform.platformName[0]}</span> : <span className="text-surface-500 text-xl">+</span>
                      )}
                      
                      <div className="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-surface-900 border-2 border-surface-800 flex items-center justify-center">
                        {(() => {
                           const Icon = platform.options[0].icon;
                           return <Icon size={10} className={platform.options[0].colorClass === 'brand' ? 'text-brand-400' : platform.options[0].colorClass === 'pink' ? 'text-pink-400' : 'text-amber-400'} />;
                        })()}
                      </div>
                      
                      {!isActive && (
                        <div className="absolute -top-1 -right-1 px-1.5 py-0.5 rounded-md bg-black/80 text-[8px] border border-surface-700 backdrop-blur-sm text-white shadow-lg pointer-events-none">
                          🔒
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Selected Account Feeds */}
              {selectedHubPlatform && (
                <div className="bg-surface-900/50 rounded-xl p-4 border border-surface-700/50 animate-in fade-in slide-in-from-top-1">
                  {(() => {
                    const platform = PLATFORM_CONTEXTS.find((p: any) => p.id === selectedHubPlatform);
                    const session = authStatus.find(s => s.platform_id === selectedHubPlatform);
                    return (
                      <>
                        <div className="text-xs text-surface-400 flex items-center gap-2 mb-3">
                           <span className="w-2 h-2 rounded-full bg-brand-500 shrink-0 shadow-[0_0_8px_rgba(99,102,241,0.5)]"></span>
                           {platform?.platformName} • {session?.username || 'Connected'}
                           {session?.error_message && (
                               <span className="ml-auto text-red-400 text-[10px] bg-red-400/10 px-2 py-0.5 rounded border border-red-400/20">
                                   Error: {session.error_message}
                               </span>
                           )}
                        </div>
                        <div className="flex gap-2 flex-wrap">
                           {platform?.options.map((opt: any) => {
                             const val = opt.feedType || opt.id;
                             const isChecked = hubSelectedFeeds.has(val);
                             const isSyncing = dbSources.some(src => 
                                 src.platform_id === platform.id && 
                                 src.is_self === true && 
                                 (src.source_type === opt.id.toUpperCase() || src.feed_type?.toLowerCase() === val.toLowerCase() || src.feed_type === val)
                             );

                             return (
                               <button
                                 key={opt.id}
                                 onClick={() => {
                                    setHubSelectedFeeds(prev => {
                                        const next = new Set(prev);
                                        if (next.has(val)) next.delete(val);
                                        else next.add(val);
                                        return next;
                                    });
                                 }}
                                 className={`px-3 py-1.5 rounded-full text-xs font-medium flex items-center gap-1.5 transition-all border ${
                                    isChecked 
                                      ? 'bg-surface-700 text-surface-100 border-brand-500/50' 
                                      : 'bg-surface-800 text-surface-400 border-surface-700 hover:bg-surface-700 hover:text-surface-200'
                                 }`}
                               >
                                 <opt.icon size={14} className={isChecked ? 'text-brand-400' : ''} />
                                 {opt.label}
                                 {isSyncing && <span className="ml-1 text-[9px] px-1.5 py-0.5 rounded bg-brand-500/10 text-brand-400 border border-brand-500/20">Syncing</span>}
                               </button>
                             );
                           })}
                        </div>
                        
                        {/* Action Button for Hub */}
                        {hubSelectedFeeds.size > 0 && (
                           <div className="mt-4 pt-4 border-t border-surface-700/50 flex justify-end">
                              <button
                                onClick={async () => {
                                   setIsSubmitting(true);
                                   setError(null);
                                   try {
                                       await invoke('add_source_command', {
                                           request: {
                                               url: `vdp://${platform?.id}/me/`,
                                               feed_types: Array.from(hubSelectedFeeds),
                                               selected_ids: null
                                           }
                                       });
                                       if (onSuccess) onSuccess();
                                       handleClose();
                                   } catch (err: any) {
                                       setError('Failed to add sources: ' + err);
                                   } finally {
                                       setIsSubmitting(false);
                                   }
                                }}
                                disabled={isSubmitting}
                                className="px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2 shadow-sm shadow-brand-600/20"
                              >
                                {isSubmitting ? <Loader2 size={16} className="animate-spin" /> : <Download size={16} />}
                                Add {hubSelectedFeeds.size} Source{hubSelectedFeeds.size > 1 ? 's' : ''}
                              </button>
                           </div>
                        )}
                      </>
                    );
                  })()}
                </div>
              )}
            </div>
          )}

          {/* Context Selector UI (For smart pasted URLs) */}
          {detectedPlatformConfig && url.trim() && (
            <div className="animate-in fade-in slide-in-from-top-2 pt-2">
              <label className="block text-xs font-semibold text-brand-400 mb-2 uppercase tracking-wider">
                Select Feeds to Track
              </label>
              <div className="flex gap-2 flex-wrap">
                {detectedPlatformConfig.options.map((opt: any) => {
                  const val = opt.feedType || opt.id;
                  const isChecked = selectedContexts.has(val);
                  return (
                    <button
                      key={opt.id}
                      onClick={() => toggleContext(val)}
                      className={`px-3 py-1.5 rounded-full text-xs font-medium flex items-center gap-1.5 transition-all border ${
                        isChecked 
                          ? 'bg-surface-700 text-surface-100 border-brand-500/50' 
                          : 'bg-surface-800 text-surface-400 border-surface-700 hover:bg-surface-700 hover:text-surface-200'
                      }`}
                    >
                      <opt.icon size={14} className={isChecked ? 'text-brand-400' : ''} />
                      {opt.label}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {error && (
            <div className="p-3 mt-4 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-500 flex items-center gap-2">
                <X size={16} onClick={() => setError(null)} className="cursor-pointer shrink-0" />
                <span>{error}</span>
            </div>
          )}

        </div>
      </div>
    </div>
  );
};
