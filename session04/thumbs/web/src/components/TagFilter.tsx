// components/TagFilter.tsx
import React, { useState, useEffect } from "react";
import { TagModel } from "../types";
import { thumbsApi } from "../services/api";
import toast from "react-hot-toast";

interface TagFilterProps {
    selectedTagId: number | null;
    onTagSelect: (tagId: number | null) => void;
}

const TagFilter: React.FC<TagFilterProps> = ({ selectedTagId, onTagSelect }) => {
    const [tags, setTags] = useState<TagModel[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        const loadTags = async () => {
            try {
                const response = await thumbsApi.getTags();
                setTags(response.data.data);
            } catch (error) {
                toast.error("Failed to load tags");
            } finally {
                setIsLoading(false);
            }
        };
        loadTags();
    }, []);

    if (isLoading) {
        return (
            <div className="h-screen flex items-center justify-center">
                <div className="text-sm text-gray-500">Loading tags...</div>
            </div>
        );
    }

    return (
        <div className="h-screen flex flex-col border-r border-gray-200">
            {/* Header - Fixed */}
            <div className="p-4 border-b border-gray-200">
                <h3 className="text-lg font-semibold">Filter by Tags</h3>
            </div>
            {/* Tags - Scrollable */}
            <div className="flex flex-wrap items-center">
                <button onClick={() => onTagSelect(null)} className={`px-5 py-2 m-1 rounded-full text-sm font-medium transition-colors ${selectedTagId === null ? "bg-blue-500 text-white" : "text-gray-700 hover:bg-gray-100 border border-gray-200"}`}>
                    All Images
                </button>
                {tags?.length > 0 &&
                    tags.map((tag) => (
                        <button
                            key={tag.id}
                            onClick={() => onTagSelect(tag.id!)}
                            className={`px-5 py-2 m-1 rounded-full text-sm font-medium transition-colors ${selectedTagId === tag.id ? "bg-blue-500 text-white" : "text-gray-700 hover:bg-gray-100 border border-gray-200"}`}>
                            {tag.name}
                        </button>
                    ))}
            </div>
        </div>
    );
};

export default TagFilter;
