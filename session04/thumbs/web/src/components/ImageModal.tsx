import React, { useState, useEffect, useTransition } from "react";
import { useForm } from "react-hook-form";
import toast from "react-hot-toast";
import humanize from "humanize-plus";
import { format as formatDate } from "date-fns";
import { ImageModel, TagModel } from "../types";
import { imageApi } from "../services/api";
import ImageWithFallback from "./ImageWithFallback";

interface ImageModalProps {
    image: ImageModel;
    tags: TagModel[];
    onClose: () => void;
    onUpdate: (image: ImageModel) => void;
    onDelete: (imageId: number) => void;
}

interface FormData {
    title: string;
    description?: string;
}

const ImageModal: React.FC<ImageModalProps> = ({ image, tags, onClose, onUpdate, onDelete }) => {
    const [isEditing, setIsEditing] = useState(false);
    const [isPending, startTransition] = useTransition();

    const { register, handleSubmit, setValue } = useForm<FormData>();

    useEffect(() => {
        setValue("title", image.title);
        setValue("description", image.description);
    }, [image, setValue]);

    const handleUpdate = async (data: FormData) => {
        startTransition(async () => {
            try {
                const response = await imageApi.updateImage(image.id!, {
                    title: data.title,
                    description: data.description,
                });
                onUpdate(response.data);
                setIsEditing(false);
                toast.success("Image updated successfully!");
            } catch (error) {
                toast.error("Failed to update image");
            }
        });
    };

    const handleDelete = async () => {
        if (window.confirm("Are you sure you want to delete this image?")) {
            startTransition(async () => {
                try {
                    await imageApi.deleteImage(image.id!);
                    onDelete(image.id!);
                    toast.success("Image deleted successfully!");
                    onClose();
                } catch (error) {
                    toast.error("Failed to delete image");
                }
            });
        }
    };

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-fade-in">
            <div className="bg-white rounded-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-2xl animate-scale-in">
                <div className="p-6">
                    <div className="flex justify-between items-center mb-6">
                        <h2 className="text-2xl font-bold text-gray-800">Details</h2>
                        <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-2xl w-8 h-8 flex items-center justify-center rounded-full hover:bg-gray-100 transition-colors">
                            ×
                        </button>
                    </div>

                    {/* Image Display */}
                    <div className="aspect-video bg-gradient-to-br from-gray-100 to-gray-200 rounded-lg mb-6 flex items-center justify-center">
                        <div className="text-gray-400 text-center">
                            <ImageWithFallback src={imageApi.getThumbUri(image.filename)} alt={image.alt_text} className="max-w-full max-h-screen" phClassName="w-24 h-24 mx-auto mb-4" />
                            <p className="font-medium">{image.filename}</p>
                        </div>
                    </div>

                    {/* Image Info */}
                    {isEditing ? (
                        <form onSubmit={handleSubmit(handleUpdate)} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-2">Title</label>
                                <input type="text" {...register("title", { required: true })} className="input-field" />
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700 mb-2">Description</label>
                                <textarea {...register("description")} className="input-field" placeholder="Enter image description" rows={3} />
                            </div>
                            <div className="flex space-x-3 pt-2">
                                <button type="submit" disabled={isPending} className="btn-primary">
                                    {isPending ? "Saving..." : "Save"}
                                </button>
                                <button type="button" onClick={() => setIsEditing(false)} className="btn-secondary">
                                    Cancel
                                </button>
                            </div>
                        </form>
                    ) : (
                        <div className="space-y-6">
                            <div className="bg-gray-50 rounded-lg p-4">
                                <h3 title={image.alt_text} className="font-semibold text-xl text-gray-800 mb-3">
                                    {image.title}
                                </h3>
                                {image.description && <div className="text-gray-600 mb-4">{image.description}</div>}
                                <div className="grid grid-cols-2 gap-4 text-sm">
                                    <div className="col-span-2">
                                        <span className="font-medium text-gray-600">Filename:</span>
                                        <p className="text-gray-800">{image.filename}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Size:</span>
                                        <p className="text-gray-800">{humanize.fileSize(image.file_size)}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Type:</span>
                                        <p className="text-gray-800">{image.mime_type}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Width:</span>
                                        <p className="text-gray-800">{image.width}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Height:</span>
                                        <p className="text-gray-800">{image.height}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Created:</span>
                                        <p className="text-gray-800">{image.created_at && formatDate(image.created_at, "yyyy-MM-dd HH:mm")}</p>
                                    </div>
                                    <div>
                                        <span className="font-medium text-gray-600">Last updated:</span>
                                        <p className="text-gray-800">{image.updated_at && formatDate(image.updated_at, "yyyy-MM-dd HH:mm")}</p>
                                    </div>
                                </div>
                            </div>

                            {tags.length > 0 && (
                                <div>
                                    <div className="flex flex-wrap gap-2">
                                        {tags.map((tag) => (
                                            <span key={tag.id} className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full text-sm font-medium">
                                                {tag.name}
                                            </span>
                                        ))}
                                    </div>
                                </div>
                            )}

                            <div className="flex space-x-3 pt-4 border-t border-gray-200">
                                <button onClick={() => setIsEditing(true)} className="btn-primary">
                                    Edit
                                </button>
                                <button onClick={handleDelete} disabled={isPending} className="btn-danger">
                                    {isPending ? "Deleting..." : "Delete"}
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default ImageModal;
