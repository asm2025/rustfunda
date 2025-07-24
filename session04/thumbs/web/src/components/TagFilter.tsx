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
        return <div className="text-sm text-gray-500">Loading tags...</div>;
    }

    return (
        <div className="mb-6">
            <h3 className="text-lg font-semibold mb-3">Filter by Tags</h3>
            <div className="flex flex-wrap gap-2">
                <button onClick={() => onTagSelect(null)} className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${selectedTagId === null ? "bg-blue-500 text-white" : "bg-gray-200 text-gray-700 hover:bg-gray-300"}`}>
                    All Images
                </button>
                {tags.map((tag) => (
                    <button key={tag.id} onClick={() => onTagSelect(tag.id!)} className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${selectedTagId === tag.id ? "bg-blue-500 text-white" : "bg-gray-200 text-gray-700 hover:bg-gray-300"}`}>
                        {tag.name}
                    </button>
                ))}
            </div>
        </div>
    );
};

export default TagFilter;
