export interface Media {
    id: string;
    media_type: string;
    file_path: string;
    thumbnail_path: string | null;
    thumbnail_sm_path: string | null;
    order_index: number;
    width: number | null;
    height: number | null;
    duration: number | null;
    file_size: number | null;
}

export interface Post {
    id: string;
    creator_id: string;
    source_id: string | null;
    title: string | null;
    description: string | null;
    original_url: string;
    status: string;
    posted_at: string | null;
    downloaded_at: string | null;
    created_at: string;

    // Joined creator data
    creator_name: string | null;
    creator_handle: string | null;
    creator_avatar: string | null;

    media: Media[];
}

export interface PostsPage {
    posts: Post[];
    total: number;
    page: number;
    limit: number;
    total_pages: number;
}
