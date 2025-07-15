import React, { useState, useTransition } from "react";
import { useForm } from "react-hook-form";
import toast from "react-hot-toast";
import humanize from "humanize-plus";
import { imageApi } from "../services/api";
import { ImageModel } from "../types";

const MAX_FILE_SIZE = Math.pow(2, 20) * 10;

interface ImageUploadProps {
    onUpload: (image: ImageModel) => void;
}

interface FormData {
    title: string;
    description: string;
    file: FileList;
    tags: string;
}

const ImageUpload: React.FC<ImageUploadProps> = ({ onUpload }) => {
    const [isPending, startTransition] = useTransition();
    const {
        register,
        handleSubmit,
        reset,
        formState: { errors },
    } = useForm<FormData>();

    const onSubmit = async (data: FormData) => {
        const file = data.file && data.file.length > 0 && data.file[0];

        if (file) {
            // Validate file size (2MB max)
            if (file.size > MAX_FILE_SIZE) {
                toast.error(`File size must not be larger than ${humanize.fileSize(MAX_FILE_SIZE)}`);
                return;
            }

            // Validate file type
            if (!file.type.startsWith("image/")) {
                toast.error("Please select an image file");
                return;
            }
        }

        startTransition(async () => {
            try {
                const formData = new window.FormData();
                formData.append("title", data.title);
                formData.append("description", data.description || "");
                formData.append("alt_text", data.title);
                formData.append("tags", data.tags || "");

                if (file) {
                    formData.append("image_file", file);
                    formData.append("file", file);
                    formData.append("filename", file.name);
                    formData.append("mime_type", file.type);
                }

                const response = await imageApi.createImage(formData);
                onUpload(response.data);
                reset();
                toast.success("Image uploaded successfully!");
            } catch (error) {
                toast.error("Failed to upload image");
                console.error("Upload error:", error);
            }
        });
    };

    return (
        <div className="card p-6 mb-8 animate-slide-up">
            <h2 className="text-2xl font-bold mb-4 text-gray-800">Upload New Image</h2>
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">Title</label>
                    <input type="text" {...register("title", { required: "Title is required" })} className="input-field" placeholder="Enter image title" />
                    {errors.title && <p className="text-red-500 text-sm mt-1">{errors.title.message}</p>}
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">Description</label>
                    <textarea {...register("description")} className="input-field" placeholder="Enter image description" rows={3} />
                    {errors.description && <p className="text-red-500 text-sm mt-1">{errors.description.message}</p>}
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">Image File (Max 2MB)</label>
                    <input
                        type="file"
                        accept="image/*"
                        {...register("file", { required: "Please select an image" })}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 
						focus:ring-blue-500 transition-all file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 
						file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100 file:cursor-pointer"
                    />
                    {errors.file && <p className="text-red-500 text-sm mt-1">{errors.file.message}</p>}
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">Tags (comma-separated)</label>
                    <input type="text" {...register("tags")} className="input-field" placeholder="nature, landscape, sunset" />
                </div>

                <button type="submit" disabled={isPending} className="w-full btn-primary">
                    {isPending ? (
                        <span className="flex items-center justify-center">
                            <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            Uploading...
                        </span>
                    ) : (
                        "Upload"
                    )}
                </button>
            </form>
        </div>
    );
};

export default ImageUpload;
