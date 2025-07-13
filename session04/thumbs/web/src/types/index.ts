export interface ImageModel {
    id?: number;
    title: string;
    description?: string;
    filename: string;
    file_size: number;
    mime_type: string;
    width?: number;
    height?: number;
    alt_text?: string;
    created_at?: string;
    updated_at?: string;
}

export interface TagModel {
    id?: number;
    name: string;
    created_at?: string;
    updated_at?: string;
}

export interface ImageWithTags extends ImageModel {
    tags?: TagModel[];
}
