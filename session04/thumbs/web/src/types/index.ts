export interface ImageModel {
    id?: number;
    title: string;
    description?: string;
    extension: string;
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

export interface ModelWithRelated<M, R> {
    item: M;
    related: R[];
}

export interface Pagination {
    page: number;
    page_size: number;
}

export interface ResultSet<T> {
    data: T[];
    total: number;
    pagination?: Pagination;
}
