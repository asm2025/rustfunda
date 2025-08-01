import React, { useEffect, useState } from "react";
import Markdown from "react-markdown";
import { thumbsApi } from "../services/api";
import toast from "react-hot-toast";

const About: React.FC = () => {
    const [markdownContent, setMarkdownContent] = useState("");
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        const loadAbout = async () => {
            try {
                const response = await thumbsApi.getAbout();
                setMarkdownContent(response.data);
            } catch (error) {
                toast.error("Failed to load about content");
                setMarkdownContent("# About\n\nFailed to load content from server.");
            } finally {
                setIsLoading(false);
            }
        };

        loadAbout();
    }, []);

    if (isLoading) {
        return <div className="flex items-center justify-center py-12">Loading...</div>;
    }

    return (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 max-h-[70vh] overflow-y-auto">
            <Markdown
                components={{
                    h1: ({ children }) => <h1 className="text-3xl font-bold mb-4 text-gray-900">{children}</h1>,
                    h2: ({ children }) => <h2 className="text-2xl font-semibold mb-3 text-gray-800">{children}</h2>,
                    p: ({ children }) => <p className="mb-4 text-gray-700 leading-relaxed">{children}</p>,
                    code: ({ children }) => <code className="bg-gray-100 px-2 py-1 rounded text-sm">{children}</code>,
                }}>
                {markdownContent}
            </Markdown>
        </div>
    );
};

export default About;
