import { useState } from 'react';
import { PlatformSession } from '../../types/auth';
import { useDeleteSession, useVerifySession } from '../../hooks/useAuth';
import { ConnectAccountModal } from './ConnectAccountModal';
import { DisconnectAccountModal } from './DisconnectAccountModal';
import { 
  CheckCircle2, 
  XCircle, 
  LogOut, 
  Upload,
  AlertCircle,
  RefreshCw
} from 'lucide-react';

interface AccountCardProps {
  platformId: string;
  name: string;
  session?: PlatformSession;
}

export function AccountCard({ platformId, name, session }: AccountCardProps) {
  const [showConnectModal, setShowConnectModal] = useState(false);
  const [showDisconnectModal, setShowDisconnectModal] = useState(false);
  const deleteSession = useDeleteSession();
  const verifySession = useVerifySession();

  const isConnected = session?.status === 'ACTIVE';
  const isExpired = session?.status === 'EXPIRED';

  const handleDisconnectConfirm = async () => {
    try {
      await deleteSession.mutateAsync(platformId);
      setShowDisconnectModal(false);
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  };

  return (
    <>
      <div className="bg-zinc-800/50 border border-zinc-700/50 rounded-xl p-4 flex items-center justify-between group hover:border-zinc-600 transition-colors">
        <div className="flex items-center gap-4">
          {/* Avatar / Icon */}
          <div className={`w-10 h-10 rounded-full flex items-center justify-center overflow-hidden flex-shrink-0 ${
            isConnected ? 'bg-green-500/10 text-green-500' : 'bg-zinc-700 text-zinc-400'
          }`}>
             {session?.avatar_url ? (
               <img src={session.avatar_url} alt={name} className="w-full h-full object-cover" />
             ) : (
               <span className="font-bold text-lg">{name.charAt(0)}</span>
             )}
          </div>

          <div>
            <h4 className="font-medium text-white">{name}</h4>
            <div className="flex items-center gap-2 mt-1">
              {isConnected ? (
                <span className="text-xs flex items-center gap-1.5 text-green-400 bg-green-400/10 px-2 py-0.5 rounded-full">
                  <CheckCircle2 size={12} />
                  {session?.username ? `@${session.username}` : 'Connected'}
                </span>
              ) : isExpired ? (
                <span className="text-xs flex items-center gap-1.5 text-orange-400 bg-orange-400/10 px-2 py-0.5 rounded-full" title={session?.error_message || "Session Expired"}>
                  <AlertCircle size={12} />
                  Expired
                </span>
              ) : (
                <span className="text-xs flex items-center gap-1.5 text-zinc-500 bg-zinc-700/50 px-2 py-0.5 rounded-full">
                  <XCircle size={12} />
                  Not Connected
                </span>
              )}
              
              {(isConnected || isExpired) && session?.last_verified && (
                 <span className="text-[10px] text-zinc-500">
                    Checked {new Date(session.last_verified).toLocaleDateString()}
                 </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {isConnected ? (
            <>
              <button
                onClick={() => verifySession.mutate(platformId)}
                disabled={verifySession.isPending}
                className="p-2 text-zinc-400 hover:text-blue-400 hover:bg-zinc-700 rounded-lg transition-colors disabled:opacity-50"
                title="Verify Session"
              >
                <RefreshCw size={18} className={verifySession.isPending ? 'animate-spin' : ''} />
              </button>
              <button
                onClick={() => setShowDisconnectModal(true)}
                className="p-2 text-zinc-400 hover:text-red-400 hover:bg-zinc-700 rounded-lg transition-colors"
                title="Disconnect"
              >
                <LogOut size={18} />
              </button>
            </>
          ) : isExpired ? (
            <>
              <button
                onClick={() => setShowConnectModal(true)}
                className="px-3 py-1.5 text-xs bg-orange-600/20 hover:bg-orange-600/30 text-orange-400 rounded-lg flex items-center gap-1.5 transition-colors border border-orange-500/20"
                title="Session expired. Reconnect."
              >
                <Upload size={14} />
                Reconnect
              </button>
              <button
                onClick={() => setShowDisconnectModal(true)}
                className="p-2 text-zinc-400 hover:text-red-400 hover:bg-zinc-700 rounded-lg transition-colors"
                title="Remove"
              >
                <LogOut size={18} />
              </button>
            </>
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

      {showDisconnectModal && (
        <DisconnectAccountModal
          platformName={name}
          onClose={() => setShowDisconnectModal(false)}
          onConfirm={handleDisconnectConfirm}
          isPending={deleteSession.isPending}
        />
      )}
    </>
  );
}
