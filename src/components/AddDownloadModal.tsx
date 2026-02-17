import React, { useState } from 'react';
import { useDownloadManager } from '../hooks/useDownloadManager';
import { X, Download, Loader2 } from 'lucide-react';

interface AddDownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({ isOpen, onClose }) => {
  const [url, setUrl] = useState('');
  const [format, setFormat] = useState('best'); // 'best' (video+audio) or 'bestaudio'
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { createDownload } = useDownloadManager();

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) {
      setError('Please enter a valid URL');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      await createDownload(url, format === 'best' ? undefined : format);
      setUrl('');
      setFormat('best');
      onClose();
    } catch (err) {
      setError('Failed to create download. Please try again.');
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md bg-zinc-900 border border-zinc-800 rounded-xl shadow-xl p-6 relative animate-in fade-in zoom-in duration-200">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-zinc-400 hover:text-white transition-colors"
        >
          <X size={20} />
        </button>

        <h2 className="text-xl font-semibold text-white mb-4">Add New Download</h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="url" className="block text-sm font-medium text-zinc-400 mb-1">
              Video URL
            </label>
            <input
              id="url"
              type="text"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://youtube.com/watch?v=..."
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-400 mb-1">
              Format
            </label>
            <div className="grid grid-cols-2 gap-2">
              <button
                type="button"
                onClick={() => setFormat('best')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  format === 'best'
                    ? 'bg-blue-600 text-white'
                    : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'
                }`}
              >
                Best Quality (Video + Audio)
              </button>
              <button
                type="button"
                onClick={() => setFormat('bestaudio')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  format === 'bestaudio'
                    ? 'bg-blue-600 text-white'
                    : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'
                }`}
              >
                Audio Only
              </button>
            </div>
          </div>

          {error && (
            <div className="text-red-400 text-sm bg-red-400/10 px-3 py-2 rounded-lg">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 mt-6">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-zinc-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting || !url.trim()}
              className="flex items-center gap-2 px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
        </form>
      </div>
    </div>
  );
};
