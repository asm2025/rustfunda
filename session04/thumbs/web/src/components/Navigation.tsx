import React from "react";
import { Link, useLocation } from "react-router-dom";
import logo from "./assets/gallery.svg";

const Navigation: React.FC = () => {
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;

    return (
        <nav className="bg-white shadow-sm border-b border-gray-200">
            <div className="container mx-auto px-4">
                <div className="flex items-center justify-between h-16">
                    <div className="flex items-center space-x-8">
                        <Link to="/" className="flex items-center space-x-2">
                            <img src={logo} alt="Gallery" className="w-8 h-8" />
                            <span className="text-xl font-bold text-gray-900">Gallery</span>
                        </Link>

                        <div className="flex space-x-6">
                            <Link to="/images" className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${isActive("/images") ? "bg-blue-100 text-blue-700" : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"}`}>
                                Images
                            </Link>
                            <Link to="/about" className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${isActive("/about") ? "bg-blue-100 text-blue-700" : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"}`}>
                                About
                            </Link>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
};

export default Navigation;
