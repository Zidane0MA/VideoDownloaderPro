export interface PlatformSession {
  platform_id: string;
  status: 'ACTIVE' | 'EXPIRED' | 'NONE';
  username?: string;
  avatar_url?: string;
  cookie_method: string;
  expires_at?: string; // ISO string
  last_verified?: string; // ISO string
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export type PlatformCategory = 'Video' | 'Social' | 'Music' | 'Other';

export interface PlatformConfig {
  id: string;
  name: string;
  icon: string;
  category: PlatformCategory;
  popularity: number;
}

export const PLATFORMS: PlatformConfig[] = [
  { id: 'youtube', name: 'YouTube', icon: 'youtube', category: 'Video', popularity: 100 },
  { id: 'tiktok', name: 'TikTok', icon: 'music-2', category: 'Video', popularity: 90 },
  { id: 'instagram', name: 'Instagram', icon: 'instagram', category: 'Social', popularity: 80 },
  { id: 'x', name: 'X (Twitter)', icon: 'twitter', category: 'Social', popularity: 70 },
];
