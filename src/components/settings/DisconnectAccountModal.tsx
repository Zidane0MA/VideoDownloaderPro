import { AlertTriangle, X } from 'lucide-react';

interface DisconnectAccountModalProps {
  platformName: string;
  onClose: () => void;
  onConfirm: () => void;
  isPending: boolean;
}

export function DisconnectAccountModal({ 
  platformName, 
  onClose, 
  onConfirm, 
  isPending 
}: DisconnectAccountModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl w-full max-w-md flex flex-col">
        
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-800">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <AlertTriangle className="text-orange-500" size={20} />
            Disconnect Account
          </h3>
          <button 
            onClick={onClose}
            disabled={isPending}
            className="p-1 hover:bg-zinc-800 rounded-full transition-colors disabled:opacity-50"
          >
            <X size={20} className="text-zinc-400" />
          </button>
        </div>

        {/* Body */}
        <div className="p-6">
          <p className="text-zinc-300">
            Are you sure you want to disconnect your <strong className="text-white">{platformName}</strong> account?
          </p>
          <p className="mt-2 text-sm text-zinc-400">
            You will need to sign in again to download age-restricted content or access private videos.
          </p>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-4 border-t border-zinc-800 bg-zinc-900/50 rounded-b-xl">
          <button
            onClick={onClose}
            disabled={isPending}
            className="px-4 py-2 text-sm font-medium text-zinc-300 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isPending}
            className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-500 rounded-lg transition-colors shadow-lg shadow-red-900/20 flex items-center gap-2 disabled:opacity-50"
          >
            {isPending ? (
              <>
                <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                Disconnecting...
              </>
            ) : (
              'Disconnect'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
