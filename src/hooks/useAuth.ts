import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { PlatformSession } from '../types/auth';

// Fetch all sessions
export const useAuthStatus = () => {
  return useQuery({
    queryKey: ['auth-status'],
    queryFn: async () => {
      const sessions = await invoke<PlatformSession[]>('get_auth_status');
      // Normalize: API might return missing platforms as simply not in the list.
      // We process this in the UI or here. Let's return raw list.
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
