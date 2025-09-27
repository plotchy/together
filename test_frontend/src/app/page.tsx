'use client';

import { UltrasonicDetector } from '@/components/UltrasonicDetector';

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-purple-50">
      <UltrasonicDetector />
    </div>
  );
}