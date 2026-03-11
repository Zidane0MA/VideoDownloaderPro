import { Play, Film, Radio, LayoutGrid, Heart, Bookmark } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

export interface ContextOption {
    id: string;
    label: string;
    icon: LucideIcon;
    colorClass: 'brand' | 'pink' | 'amber'; // Used by frontend to apply specific tailwind themes
    feedType?: string; // Optional because some contexts don't use feed_type
    urlMutator: (baseUrl: string) => string;
}

export interface PlatformConfig {
    id: string;
    platformName: string;
    targetRegex: RegExp;
    options: ContextOption[];
}

export const PLATFORM_CONTEXTS: PlatformConfig[] = [
    {
        id: 'youtube',
        platformName: 'YouTube',
        // Matches youtube.com/@username with optional trailing slash
        targetRegex: /youtube\.com\/@([^/]+)\/?$/,
        options: [
            {
                id: 'default',
                label: 'Videos',
                icon: Play,
                colorClass: 'brand',
                feedType: 'VIDEOS',
                urlMutator: (url) => url
            },
            {
                id: 'shorts',
                label: 'Shorts',
                icon: Film,
                colorClass: 'brand',
                feedType: 'SHORTS',
                urlMutator: (url) => url.endsWith('/') ? `${url}shorts` : `${url}/shorts`
            },
            {
                id: 'streams',
                label: 'Streams',
                icon: Radio,
                colorClass: 'brand',
                feedType: 'STREAMS',
                urlMutator: (url) => url.endsWith('/') ? `${url}streams` : `${url}/streams`
            }
        ]
    },
    {
        id: 'tiktok',
        platformName: 'TikTok',
        // Matches tiktok.com/@username with optional trailing slash
        targetRegex: /tiktok\.com\/@([^/]+)\/?$/,
        options: [
            {
                id: 'default',
                label: 'Public',
                icon: LayoutGrid,
                colorClass: 'brand',
                feedType: 'VIDEOS',
                urlMutator: (url) => url
            },
            {
                id: 'liked',
                label: 'Liked',
                icon: Heart,
                colorClass: 'pink',
                urlMutator: (url) => url.endsWith('/') ? `${url}liked` : `${url}/liked`
            },
            {
                id: 'saved',
                label: 'Saved',
                icon: Bookmark,
                colorClass: 'amber',
                urlMutator: (url) => url.endsWith('/') ? `${url}saved` : `${url}/saved`
            }
        ]
    }
];
