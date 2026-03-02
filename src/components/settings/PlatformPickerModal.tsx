import { useState, useMemo } from 'react';
import { X, Search } from 'lucide-react';
import { PLATFORMS, PlatformCategory } from '../../types/auth';

interface PlatformPickerModalProps {
  onClose: () => void;
  onSelect: (platformId: string) => void;
}

const CATEGORIES: ('All' | PlatformCategory)[] = ['All', 'Video', 'Social', 'Music', 'Other'];

export function PlatformPickerModal({ onClose, onSelect }: PlatformPickerModalProps) {
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState<'All' | PlatformCategory>('All');

  const filteredPlatforms = useMemo(() => {
    return PLATFORMS.filter(p => {
      const matchesSearch = p.name.toLowerCase().includes(search.toLowerCase());
      const matchesCategory = category === 'All' || p.category === category;
      return matchesSearch && matchesCategory;
    }).sort((a, b) => b.popularity - a.popularity);
  }, [search, category]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-surface-800 border border-surface-700 rounded-xl shadow-2xl w-full max-w-2xl flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-surface-700">
          <h3 className="text-lg font-semibold text-surface-100">
            Add Account
          </h3>
          <button
            onClick={onClose}
            className="p-1 hover:bg-surface-700 rounded-full transition-colors"
          >
            <X size={20} className="text-surface-400" />
          </button>
        </div>

        {/* Search & Filters */}
        <div className="p-4 border-b border-surface-700 space-y-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-surface-400" size={18} />
            <input
              type="text"
              placeholder="Search platforms..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full bg-surface-900 border border-surface-700 rounded-lg pl-10 pr-4 py-2 text-sm text-surface-200 focus:outline-none focus:ring-2 focus:ring-brand-500/50"
            />
          </div>

          <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-hide">
            {CATEGORIES.map(cat => (
              <button
                key={cat}
                onClick={() => setCategory(cat)}
                className={`px-3 py-1.5 text-xs font-medium rounded-full whitespace-nowrap transition-colors ${category === cat
                    ? 'bg-brand-500/20 text-brand-600 border border-brand-500/30'
                    : 'bg-surface-800 text-surface-400 border border-surface-700 hover:bg-surface-700 hover:text-surface-100'
                  }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        {/* Results */}
        <div className="p-4 flex-1 overflow-y-auto min-h-[300px]">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {filteredPlatforms.map(platform => (
              <button
                key={platform.id}
                onClick={() => onSelect(platform.id)}
                className="bg-surface-800/50 hover:bg-surface-800 border border-surface-700/50 hover:border-surface-600 rounded-xl p-4 flex items-center gap-4 text-left transition-all"
              >
                <div className="w-10 h-10 rounded-full bg-surface-700 flex items-center justify-center shrink-0">
                  {/* In a real app we would use the actual icon here instead of a letter if present */}
                  <span className="font-bold text-lg text-surface-100">{platform.name.charAt(0)}</span>
                </div>
                <div>
                  <h4 className="font-medium text-surface-100">{platform.name}</h4>
                  <p className="text-xs text-surface-500">{platform.category}</p>
                </div>
              </button>
            ))}
          </div>

          {filteredPlatforms.length === 0 && (
            <div className="text-center py-12 text-surface-500">
              No platforms found for "{search}"
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
