'use client';
import { useState, useEffect } from 'react';
import { Session } from 'next-auth';
import { apiClient } from '@/lib/api';
import { useProfile } from '@/contexts/ProfileContext';
import { CreateTogether } from '@/components/CreateTogether';
import { AuthButton } from '@/components/AuthButton';

interface HomeContentProps {
  session: Session | null;
}

export const HomeContent = ({ session }: HomeContentProps) => {
  const { setUser } = useProfile();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchUser = async () => {
      if (!session?.user?.walletAddress) {
        setLoading(false);
        return;
      }

      try {
        const userResponse = await apiClient.getOrCreateUser(session.user.walletAddress);
        
        if (userResponse.error) {
          setError(userResponse.error);
        } else if (userResponse.data) {
          setUser(userResponse.data);
          setError(null);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchUser();
  }, [session?.user?.walletAddress]);

  // Show auth button if no session
  if (!session?.user?.walletAddress) {
    return <AuthButton />;
  }

  // Show loading state
  if (loading) {
    return (
      <div className="w-full space-y-6">
        <div className="text-center">
          <div className="h-6 bg-gray-200 rounded w-48 mx-auto mb-2 animate-pulse"></div>
          <div className="h-12 bg-gray-200 rounded w-24 mx-auto animate-pulse"></div>
        </div>
        <div className="space-y-4">
          <div className="h-4 bg-gray-200 rounded w-64 mx-auto animate-pulse"></div>
          <div className="h-12 bg-gray-200 rounded w-full animate-pulse"></div>
          <div className="h-12 bg-gray-200 rounded w-full animate-pulse"></div>
        </div>
      </div>
    );
  }

  // Show error state
  if (error) {
    return (
      <div className="w-full p-4 bg-red-50 rounded-xl border-2 border-red-200">
        <p className="text-red-600 text-sm text-center">Failed to load user data: {error}</p>
        <AuthButton />
      </div>
    );
  }

  // Show the main interface once user is loaded
  return <CreateTogether session={session} />;
};
