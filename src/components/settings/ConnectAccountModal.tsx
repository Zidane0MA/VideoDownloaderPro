import { useState } from 'react';
import { useUpdateSession, useOpenLoginWindow, useImportFromBrowser } from '../../hooks/useAuth';
import { X, Save, AlertCircle, Globe, Chrome, FileText, CheckCircle2 } from 'lucide-react';

interface ConnectAccountModalProps {
  platformId: string;
  platformName: string;
  onClose: () => void;
}

type Tab = 'webview' | 'browser' | 'manual';

export function ConnectAccountModal({ platformId, platformName, onClose }: ConnectAccountModalProps) {
  const [activeTab, setActiveTab] = useState<Tab>('webview');
  const [cookies, setCookies] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  
  const updateSession = useUpdateSession();
  const openLoginWindow = useOpenLoginWindow();
  const importFromBrowser = useImportFromBrowser();

  const handleManualSave = async () => {
    setError(null);
    if (!cookies.trim()) {
      setError('Please paste your cookies first.');
      return;
    }

    try {
      await updateSession.mutateAsync({
        platform_id: platformId,
        cookies_str: cookies,
        method: 'manual_import',
      });
      onClose();
    } catch (e) {
      setError('Failed to save cookies: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleWebViewLogin = async () => {
    setError(null);
    try {
      await openLoginWindow.mutateAsync(platformId);
    } catch (e) {
        setError('Failed to open window: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleWebViewCheck = async () => {
      setError(null);
      setSuccessMsg(null);
      try {
          // "webview" is the special browser keyword we handled in backend
          await importFromBrowser.mutateAsync({ platform_id: platformId, browser: 'webview' });
          setSuccessMsg('Successfully connected via WebView!');
          setTimeout(onClose, 1500);
      } catch (e) {
          setError(e instanceof Error ? e.message : String(e));
      }
  };
  
  const handleBrowserImport = async (browser: string) => {
      setError(null);
      try {
          await importFromBrowser.mutateAsync({ platform_id: platformId, browser });
          setSuccessMsg(`Successfully imported from ${browser}!`);
          setTimeout(onClose, 1500);
      } catch (e) {
          setError(e instanceof Error ? e.message : String(e));
      }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[90vh]">
        
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-800">
          <h3 className="text-lg font-semibold text-white">
            Connect {platformName}
          </h3>
          <button 
            onClick={onClose}
            className="p-1 hover:bg-zinc-800 rounded-full transition-colors"
          >
            <X size={20} className="text-zinc-400" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-zinc-800">
            <button 
                onClick={() => setActiveTab('webview')}
                className={`flex-1 p-3 text-sm font-medium transition-colors border-b-2 flex items-center justify-center gap-2 ${activeTab === 'webview' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-400 hover:text-zinc-200'}`}
            >
                <Globe size={16} /> WebView
            </button>
            <button 
                onClick={() => setActiveTab('browser')}
                className={`flex-1 p-3 text-sm font-medium transition-colors border-b-2 flex items-center justify-center gap-2 ${activeTab === 'browser' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-400 hover:text-zinc-200'}`}
            >
                <Chrome size={16} /> Browser
            </button>
            <button 
                onClick={() => setActiveTab('manual')}
                className={`flex-1 p-3 text-sm font-medium transition-colors border-b-2 flex items-center justify-center gap-2 ${activeTab === 'manual' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-400 hover:text-zinc-200'}`}
            >
                <FileText size={16} /> Manual
            </button>
        </div>

        {/* Body */}
        <div className="p-6 flex-1 overflow-y-auto min-h-[300px]">
          
          {activeTab === 'webview' && (
              <div className="space-y-6">
                  <div className="bg-blue-500/10 border border-blue-500/20 p-4 rounded-lg">
                      <h4 className="font-semibold text-blue-400 mb-2">Instructions</h4>
                      <ol className="list-decimal list-inside text-sm text-zinc-300 space-y-2">
                          <li>Click <strong>Open Login Window</strong> below.</li>
                          <li>Log in to {platformName} in the new window.</li>
                          <li>Once logged in, close the window or return here.</li>
                          <li>Click <strong>Check Status</strong> to verify.</li>
                      </ol>
                  </div>
                  
                  <div className="flex flex-col gap-3">
                      <button
                        onClick={handleWebViewLogin}
                        disabled={openLoginWindow.isPending}
                        className="w-full py-3 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg flex items-center justify-center gap-2 transition-colors font-medium border border-zinc-700"
                      >
                         {openLoginWindow.isPending ? 'Opening...' : '1. Open Login Window'}
                      </button>

                      <button
                        onClick={handleWebViewCheck}
                        disabled={importFromBrowser.isPending}
                        className="w-full py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg flex items-center justify-center gap-2 transition-colors font-medium shadow-lg shadow-blue-900/20"
                      >
                         {importFromBrowser.isPending ? (
                             <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                         ) : (
                             '2. Check Status / Import Cookies'
                         )}
                      </button>
                  </div>
              </div>
          )}

          {activeTab === 'browser' && (
              <div className="space-y-6">
                  <div className="bg-orange-500/10 border border-orange-500/20 p-4 rounded-lg mb-4">
                      <div className="flex gap-2">
                          <AlertCircle className="w-5 h-5 text-orange-400 shrink-0" />
                          <div className="text-sm text-zinc-300 space-y-2">
                              <p><strong className="text-orange-400">Important:</strong> Your browser must be <strong>completely closed</strong> (no background processes).</p>
                              <p>Modern browsers may block this due to encryption.</p>
                          </div>
                      </div>
                  </div>

                  <p className="text-sm text-zinc-400">
                      Import cookies directly from your standard browser. 
                  </p>

                  <div className="grid grid-cols-2 gap-3">
                      {['chrome', 'edge', 'firefox', 'opera'].map(b => (
                          <button
                            key={b}
                            onClick={() => handleBrowserImport(b)}
                            disabled={importFromBrowser.isPending}
                            className="p-4 bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 rounded-xl flex flex-col items-center gap-2 transition-colors disabled:opacity-50"
                          >
                              <span className="capitalize font-medium text-white">{b}</span>
                              {importFromBrowser.isPending && importFromBrowser.variables?.browser === b ? (
                                  <div className="w-4 h-4 border-2 border-zinc-500 border-t-white rounded-full animate-spin" />
                              ) : (
                                  <span className="text-xs text-zinc-500">Click to Import</span>
                              )}
                          </button>
                      ))}
                  </div>
              </div>
          )}

          {activeTab === 'manual' && (
              <div className="space-y-4">
                  <p className="text-sm text-zinc-400">
                    Paste your Netscape-formatted cookies below.
                  </p>
                  <textarea
                    className="w-full h-48 bg-zinc-950 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 resize-none"
                    placeholder={`# Netscape HTTP Cookie File\n.${platformId}.com\tTRUE\t/\tFALSE\t1767676767\t...`}
                    value={cookies}
                    onChange={(e) => setCookies(e.target.value)}
                  />
                  <div className="flex justify-end">
                     <button
                        onClick={handleManualSave}
                        disabled={updateSession.isPending}
                        className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-lg flex items-center gap-2 transition-colors"
                      >
                        {updateSession.isPending ? (
                          <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                        ) : (
                          <>
                            <Save size={16} /> Save
                          </>
                        )}
                      </button>
                  </div>
              </div>
          )}

          {error && (
            <div className="mt-4 flex items-center gap-2 text-red-400 text-sm bg-red-400/10 p-3 rounded-lg border border-red-400/20 animate-in fade-in slide-in-from-top-2">
              <AlertCircle size={16} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}
          
          {successMsg && (
            <div className="mt-4 flex items-center gap-2 text-green-400 text-sm bg-green-400/10 p-3 rounded-lg border border-green-400/20 animate-in fade-in slide-in-from-top-2">
              <CheckCircle2 size={16} className="shrink-0" />
              <span>{successMsg}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
