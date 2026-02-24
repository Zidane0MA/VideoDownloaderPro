import { invoke } from '@tauri-apps/api/core';

export async function deletePost(postId: string): Promise<void> {
    return await invoke('delete_post', { postId });
}

export async function revealInExplorer(filePath: string): Promise<void> {
    return await invoke('reveal_in_explorer', { filePath });
}
