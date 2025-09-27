import { Page } from '@/components/PageLayout';
import { AuthButton } from '../components/AuthButton';

export default function Home() {
  return (
    <Page>
      <Page.Main className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-blue-50 to-white px-6">
        <div className="max-w-md mx-auto text-center space-y-8">
          {/* App Logo/Icon */}
          <div className="w-20 h-20 mx-auto bg-blue-600 rounded-full flex items-center justify-center">
            <span className="text-white text-2xl font-bold">T</span>
          </div>
          
          {/* Hero Text */}
          <div className="space-y-4">
            <h1 className="text-3xl font-bold text-gray-900">
              Welcome to Together!
            </h1>
            <p className="text-lg text-gray-600 leading-relaxed">
              World proves you're a human, and we prove you're together
            </p>
          </div>

          {/* Features */}
          <div className="space-y-4 text-left">
            <div className="flex items-start space-x-3">
              <div className="w-6 h-6 bg-green-100 rounded-full flex items-center justify-center mt-0.5">
                <span className="text-green-600 text-sm">✓</span>
              </div>
              <p className="text-gray-700">Meet with your friends and connect with them!</p>
            </div>
            <div className="flex items-start space-x-3">
              <div className="w-6 h-6 bg-green-100 rounded-full flex items-center justify-center mt-0.5">
                <span className="text-green-600 text-sm">✓</span>
              </div>
              <p className="text-gray-700">Prove authentic human connections in the real world</p>
            </div>
            <div className="flex items-start space-x-3">
              <div className="w-6 h-6 bg-green-100 rounded-full flex items-center justify-center mt-0.5">
                <span className="text-green-600 text-sm">✓</span>
              </div>
              <p className="text-gray-700">Build your verified network of real relationships</p>
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
