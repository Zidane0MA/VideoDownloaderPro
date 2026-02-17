import { useState } from 'react';
import { PlatformSession } from '../../types/auth';
import { useDeleteSession } from '../../hooks/useAuth';
import { ConnectAccountModal } from './ConnectAccountModal';
import { 
  CheckCircle2, 
  XCircle, 
  LogOut, 
  Upload 
} from 'lucide-react';

interface AccountCardProps {
  platformId: string;
  name: string;
  session?: PlatformSession;
}

export function AccountCard({ platformId, name, session }: AccountCardProps) {
  const [showConnectModal, setShowConnectModal] = useState(false);
  const deleteSession = useDeleteSession();

  const isConnected = session?.status === 'ACTIVE';
  const isExpired = session?.status === 'EXPIRED';

  const handleDisconnect = () => {
    if (confirm(`Are you sure you want to disconnect ${name}?`)) {
      deleteSession.mutate(platformId);
    }
  };

  return (
    <>
      <div className="bg-zinc-800/50 border border-zinc-700/50 rounded-xl p-4 flex items-center justify-between group hover:border-zinc-600 transition-colors">
        <div className="flex items-center gap-4">
          {/* Platform Icon Placeholders - in real app use SVGs or mapped icons */}
          <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
            isConnected ? 'bg-green-500/10 text-green-500' : 'bg-zinc-700 text-zinc-400'
          }`}>
             {/* Simple initial letter fallback if icons missing */}
             <span className="font-bold text-lg">{name.charAt(0)}</span>
          </div>

          <div>
            <h4 className="font-medium text-white">{name}</h4>
            <div className="flex items-center gap-2 mt-1">
              {isConnected ? (
                <span className="text-xs flex items-center gap-1.5 text-green-400 bg-green-400/10 px-2 py-0.5 rounded-full">
                  <CheckCircle2 size={12} />
                  Connected
                </span>
              ) : isExpired ? (
                <span className="text-xs flex items-center gap-1.5 text-orange-400 bg-orange-400/10 px-2 py-0.5 rounded-full">
                  <AlertCircle size={12} />
                  Expired
                </span>
              ) : (
                <span className="text-xs flex items-center gap-1.5 text-zinc-500 bg-zinc-700/50 px-2 py-0.5 rounded-full">
                  <XCircle size={12} />
                  Not Connected
                </span>
              )}
              
              {isConnected && session?.last_verified && (
                 <span className="text-xs text-zinc-500 ml-2">
                   Checked: {new Date(session.last_verified).toLocaleDateString()}
                 </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {isConnected ? (
            <button
              onClick={handleDisconnect}
              disabled={deleteSession.isPending}
              className="p-2 text-zinc-400 hover:text-red-400 hover:bg-zinc-700 rounded-lg transition-colors"
              title="Disconnect"
            >
              <LogOut size={18} />
            </button>
          ) : (
            <button
              onClick={() => setShowConnectModal(true)}
              className="px-3 py-1.5 text-sm bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg flex items-center gap-2 transition-colors"
            >
              <Upload size={14} />
              Connect
            </button>
          )}
        </div>
      </div>

      {showConnectModal && (
        <ConnectAccountModal
          platformId={platformId}
          platformName={name}
          onClose={() => setShowConnectModal(false)}
        />
      )}
    </>
  );
}

// Helper for AlertCircle which was missing in import above if we used it in 'isExpired' logic
import { AlertCircle } from 'lucide-react'; 
