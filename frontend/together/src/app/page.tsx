'use client';
import { Page } from '@/components/PageLayout';
import { AuthButton } from '../components/AuthButton';
import Image from 'next/image';
import React from 'react';

const InlineLanguageCarousel = () => {
  const languages = [
    "together",
    "juntos", 
    "ด้วยกัน",
    "一緒に",
    "함께",
    "一起",
    "एक साथ",
    "معاً",
    "ensemble"
  ];

  const [currentIndex, setCurrentIndex] = React.useState(0);

  React.useEffect(() => {
    const interval = setInterval(() => {
      setCurrentIndex((prev) => (prev + 1) % languages.length);
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <span className="relative inline-block min-w-[120px] h-[1.2em] text-left">
      {languages.map((item, index) => (
        <span
          key={index}
          className={`absolute left-1/2 top-0 transform -translate-x-1/2 text-gray-900 font-medium transition-opacity duration-700 ease-in-out ${
            index === currentIndex ? 'opacity-100' : 'opacity-0'
          }`}
        >
          {item}
        </span>
      ))}
    </span>
  );
};

export default function Home() {
  return (
    <Page>
      <Page.Main className="flex flex-col items-center justify-center min-h-screen bg-white px-6 py-8">
        <div className="max-w-2xl mx-auto text-center space-y-8">
          {/* App Logo */}
          <div className="w-32 h-32 mx-auto relative">
            <Image
              src="/logo_img.webp"
              alt="Together Logo"
              fill
              className="object-contain"
              priority
            />
          </div>
          
          {/* Main Title */}
          <div className="space-y-6">
            <h1 className="text-6xl font-light text-gray-900">
              Together
            </h1>
            <p className="text-2xl text-gray-600 leading-relaxed font-light max-w-lg mx-auto">
              World proves you're human, and we prove you're <InlineLanguageCarousel />
            </p>
          </div>

          {/* Features */}
          <div className="space-y-6 max-w-lg mx-auto">
            <div className="text-center space-y-2">
              <div className="w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mx-auto">
                <span className="text-xl">👥</span>
              </div>
              <p className="text-base text-gray-700 font-light">Meet with your friends and connect with them</p>
            </div>
            <div className="text-center space-y-2">
              <div className="w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mx-auto">
                <span className="text-xl">🤝</span>
              </div>
              <p className="text-base text-gray-700 font-light">Prove authentic human connections in the real world</p>
            </div>
            <div className="text-center space-y-2">
              <div className="w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mx-auto">
                <span className="text-xl">🫶</span>
              </div>
              <p className="text-base text-gray-700 font-light">Build your verified network of real relationships</p>
            </div>
          </div>

          {/* CTA */}
          <div className="pt-4">
            <AuthButton />
          </div>
        </div>
      </Page.Main>
    </Page>
  );
}
