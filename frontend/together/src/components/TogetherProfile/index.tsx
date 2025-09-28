'use client';
import { useState, useEffect, useRef, useCallback } from 'react';
import { Session } from 'next-auth';
import { apiClient } from '@/lib/api';
import { useProfile } from '@/contexts/ProfileContext';

interface TogetherProfileProps {
  session: Session | null;
}

export const TogetherProfile = ({ session }: TogetherProfileProps) => {
  const { profile, setProfile, setUser } = useProfile();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  const fetchData = useCallback(async (isInitial = false) => {
    if (!session?.user?.walletAddress) return;

    if (isInitial) {
      setLoading(true);
      setError(null);
    }

    try {
      // First, get or create the user to get their ID (only on initial load)
      if (isInitial) {
        const userResponse = await apiClient.getOrCreateUser(session.user.walletAddress);
        
        if (userResponse.error) {
          setError(userResponse.error);
          setLoading(false);
          return;
        }

        if (userResponse.data) {
          setUser(userResponse.data);
        }
      }

      // Get their profile (always refresh this)
      const profileResponse = await apiClient.getUserProfile(
        session.user.walletAddress,
        {
          username: session.user.username,
          profile_picture_url: session.user.profilePictureUrl,
          limit: 50,
        }
      );

      if (profileResponse.error) {
        if (isInitial) setError(profileResponse.error);
      } else if (profileResponse.data) {
        setProfile(profileResponse.data);
        if (isInitial) setError(null);
      }
    } catch (err) {
      if (isInitial) setError(err instanceof Error ? err.message : 'Unknown error');
    }
    
    if (isInitial) setLoading(false);
  }, [session?.user?.walletAddress, session?.user?.username, session?.user?.profilePictureUrl, setUser, setProfile]);

  useEffect(() => {
    if (!session?.user?.walletAddress) return;

    // Initial fetch
    fetchData(true);

    // Set up aggressive polling every 2 seconds for profile updates
    intervalRef.current = setInterval(() => fetchData(false), 2000);

    // Cleanup interval on unmount
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [session?.user?.walletAddress, session?.user?.username, session?.user?.profilePictureUrl, fetchData]);

  if (loading) {
    return (
      <div className="w-full p-4 bg-white rounded-xl border-2 border-gray-200">
        <div className="animate-pulse">
          <div className="flex items-center space-x-4">
            <div className="w-14 h-14 bg-gray-300 rounded-full"></div>
            <div className="space-y-2">
              <div className="h-4 bg-gray-300 rounded w-32"></div>
              <div className="h-3 bg-gray-300 rounded w-24"></div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full p-4 bg-red-50 rounded-xl border-2 border-red-200">
        <p className="text-red-600 text-sm">Failed to load profile: {error}</p>
      </div>
    );
  }

  if (!profile || !session?.user) {
    return null;
  }

  return (
    <div className="w-full space-y-4">
    </div>
  );
};
