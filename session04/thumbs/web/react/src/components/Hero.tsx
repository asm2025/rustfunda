import React from "react";

interface HeroProps {
    title: string;
    subtitle?: string;
    logo?: string | React.ReactNode;
    className?: string;
}

const Hero: React.FC<HeroProps> = ({ title, subtitle, logo, className = "bg-gradient-to-r from-blue-600 to-blue-800" }) => {
    const renderLogo = () => {
        if (!logo) return null;

        if (typeof logo === "string") {
            return <img src={logo} alt="Logo" className="h-16 w-16 object-contain mr-6 flex-shrink-0 brightness-0 invert" style={{ maxHeight: 80 }} />;
        }

        return (
            <div className="h-16 w-16 mr-6 flex items-center justify-center flex-shrink-0 text-white fill-white" style={{ maxHeight: 80 }}>
                {logo}
            </div>
        );
    };

    return (
        <div className={`${className} text-white py-20 animate-fade-in`}>
            <div className="container mx-auto px-6">
                <div className="flex items-center justify-center mb-6">
                    {renderLogo()}
                    <h1 className="text-5xl md:text-7xl font-bold animate-slide-up">{title}</h1>
                </div>
                {subtitle && (
                    <div className="text-center">
                        <p className="text-xl md:text-2xl opacity-90 max-w-2xl mx-auto animate-slide-up">{subtitle}</p>
                    </div>
                )}
            </div>
        </div>
    );
};

export default Hero;
