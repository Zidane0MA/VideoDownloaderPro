import { Youtube, Bookmark, Heart } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

export interface QuickAction {
    id: string;
    label: string;
    icon: LucideIcon;
    iconColorClass: string;
    actionUrl: string;
}

export const QUICK_ACTIONS: QuickAction[] = [
    {
        id: 'youtube-likes',
        label: 'My YouTube Likes',
        icon: Youtube,
        iconColorClass: 'text-red-500',
        actionUrl: 'https://www.youtube.com/playlist?list=LL',
    },
    {
        id: 'tiktok-saved',
        label: 'My TikTok Saved',
        icon: Bookmark,
        iconColorClass: 'text-amber-500',
        actionUrl: 'vdp://tiktok/me/saved',
    },
    {
        id: 'tiktok-likes',
        label: 'My TikTok Likes',
        icon: Heart,
        iconColorClass: 'text-pink-500',
        actionUrl: 'vdp://tiktok/me/liked',
    }
];
