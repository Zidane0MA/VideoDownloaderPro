import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { PlatformSession } from '../types/auth';

// Fetch all sessions
export const useAuthStatus = () => {
  const queryClient = useQueryClient();

  useEffect(() => {
    console.log('[useAuthStatus] Setting up event listener');
    const unlistenPromise = listen('session-status-changed', (event) => {
      console.log('[useAuthStatus] Event received:', event);
      // Force refetch even if fresh
      queryClient.invalidateQueries({ queryKey: ['auth-status'], refetchType: 'all' });
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [queryClient]);

  return useQuery({
    queryKey: ['auth-status'],
    queryFn: async () => {
      const sessions = await invoke<PlatformSession[]>('get_auth_status');
      console.log('[useAuthStatus] Fetched sessions:', sessions);
      return sessions;
    },
  });
};

// Update Session (e.g. Save Cookies)
export const useUpdateSession = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: { platform_id: string; cookies_str: string; method: string }) => {
      await invoke('update_session', {
        platformId: payload.platform_id,
        cookiesStr: payload.cookies_str,
        method: payload.method,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['auth-status'] });
    },
  });
};

// Delete Session (Disconnect)
export const useDeleteSession = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (platform_id: string) => {
      await invoke('delete_session', { platformId: platform_id });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['auth-status'] });
    },
  });
};

// Open Login Window (L1)
export const useOpenLoginWindow = () => {
  return useMutation({
    mutationFn: async (platform_id: string) => {
      await invoke('open_login_window', { platformId: platform_id });
    },
  });
};

// Import from Browser (L2/L1)
export const useImportFromBrowser = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: { platform_id: string; browser: string }) => {
      await invoke('import_from_browser', {
        platformId: payload.platform_id,
        browser: payload.browser,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['auth-status'] });
    },
  });
};
