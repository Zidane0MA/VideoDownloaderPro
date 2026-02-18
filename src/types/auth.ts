export interface PlatformSession {
  platform_id: string;
  status: 'ACTIVE' | 'EXPIRED' | 'NONE';
  username?: string;
  cookie_method: string;
  expires_at?: string; // ISO string
  last_verified?: string; // ISO string
  created_at: string;
  updated_at: string;
}

export const PLATFORMS = [
  { id: 'youtube', name: 'YouTube', icon: 'youtube' },
  { id: 'tiktok', name: 'TikTok', icon: 'music-2' }, // using music-2 as lucide icon for tiktok alternative
  { id: 'instagram', name: 'Instagram', icon: 'instagram' },
  { id: 'x', name: 'X (Twitter)', icon: 'twitter' },
];
