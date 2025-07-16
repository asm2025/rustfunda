import axios from "axios";
import { ResultSet, ImageModel, TagModel, ModelWithRelated } from "../types";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";

const api = axios.create({
    baseURL: API_BASE_URL,
    headers: {
        "Content-Type": "application/json",
    },
});

export const imageApi = {
    getImageUri: (name: string) => `${API_BASE_URL}/images/${name}`,
    getThumbUri: (name: string) => {
        const lastDotIndex = name.lastIndexOf(".");
        const baseName = lastDotIndex !== -1 ? name.substring(0, lastDotIndex) : name;
        const extension = lastDotIndex !== -1 ? name.substring(lastDotIndex) : "";
        return `${API_BASE_URL}/images/${baseName}_thumb${extension}`;
    },
    // Image endpoints
    getImages: () => api.get<ResultSet<ModelWithRelated<ImageModel, TagModel>>>("/"),
    getImageCount: () => api.get<number>("/count"),
    createImage: (formData: FormData) =>
        api.post("/", formData, {
            headers: {
                "Content-Type": undefined,
            },
        }),
    getImage: (id: number) => api.get<ModelWithRelated<ImageModel, TagModel>>(`/${id}`),
    updateImage: (id: number, image: Partial<ImageModel>) => api.put<ImageModel>(`/${id}`, image),
    deleteImage: (id: number) => api.delete(`/${id}`),

    // Image tags endpoints
    getImageTags: (id: number) => api.get<ResultSet<TagModel>>(`/${id}/tags/`),
    addImageTag: (id: number, tag: string) => api.post(`/${id}/tags/`, { tag }),
    removeImageTag: (id: number, tagId: number) => api.delete(`/${id}/tags/${tagId}`),
};

export const tagApi = {
    // Tag endpoints
    getTags: () => api.get<ResultSet<TagModel>>("/tags/"),
    getTagCount: () => api.get<number>("/tags/count"),
    createTag: (tag: Omit<TagModel, "id">) => api.post<TagModel>("/tags/", tag),
    getTag: (id: number) => api.get<TagModel>(`/tags/${id}`),
    updateTag: (id: number, tag: Partial<TagModel>) => api.put<TagModel>(`/tags/${id}`, tag),
    deleteTag: (id: number) => api.delete(`/tags/${id}`),

    // Tag images endpoints
    getTagImages: (id: number) => api.get<ResultSet<ImageModel>>(`/tags/${id}/images/`),
    addTagImage: (id: number, imageId: number) => api.post(`/tags/${id}/images/`, { imageId }),
    removeTagImage: (id: number, imageId: number) => api.delete(`/tags/${id}/images/${imageId}`),
};
