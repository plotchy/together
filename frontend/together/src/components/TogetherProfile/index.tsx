'use client';
import { useState, useEffect, useRef } from 'react';
import { Session } from 'next-auth';
import { Marble } from '@worldcoin/mini-apps-ui-kit-react';
import { apiClient } from '@/lib/api';
import { UserProfile as UserProfileType, ConnectionInfo } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';

interface TogetherProfileProps {
  session: Session | null;
}

export const TogetherProfile = ({ session }: TogetherProfileProps) => {
  const { profile, setProfile, user, setUser } = useProfile();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  const fetchData = async (isInitial = false) => {
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
  };

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
  }, [session?.user?.walletAddress, session?.user?.username, session?.user?.profilePictureUrl]);

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
      {/* User Info and Together Stats */}
      <div className="p-4 bg-white rounded-xl border-2 border-gray-200">
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-lg font-semibold">Together Stats</h2>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
            <span className="text-xs text-gray-500">Live</span>
          </div>
        </div>
        <div className="flex items-center justify-between">
          <div>
            {user && (
              <p className="text-sm font-semibold text-blue-600 mb-1">
                User ID: {user.id}
              </p>
            )}
            <p className="text-sm text-gray-600 font-mono">
              {profile.address.slice(0, 6)}...{profile.address.slice(-4)}
            </p>
          </div>
          <div className="text-right">
            <p className="text-2xl font-bold text-blue-600">
              {profile.total_connections}
            </p>
            <p className="text-sm text-gray-600">connections</p>
          </div>
        </div>
      </div>

      {/* Recent Connections */}
      {profile.recent_connections.length > 0 && (
        <div className="p-4 bg-white rounded-xl border-2 border-gray-200">
          <h3 className="text-lg font-semibold mb-3">Recent Connections</h3>
          <div className="space-y-3">
            {profile.recent_connections.map((connection: ConnectionInfo, index: number) => (
              <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                <div className="flex items-center space-x-3">
                  <Marble src="" className="w-8" />
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium text-sm">
                        {connection.partner_username || 'Anonymous'}
                      </p>
                      {connection.connection_strength && connection.connection_strength > 1 && (
                        <span className="bg-blue-100 text-blue-800 text-xs font-semibold px-2 py-1 rounded-full">
                          {connection.connection_strength}x
                        </span>
                      )}
                      {connection.has_optimistic && (
                        <span className="bg-green-100 text-green-800 text-xs font-semibold px-2 py-1 rounded-full">
                          Live
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-gray-600 font-mono">
                      {connection.partner_address.slice(0, 6)}...{connection.partner_address.slice(-4)}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-xs text-gray-500">
                    {new Date(connection.attestation_timestamp * 1000).toLocaleDateString()}
                  </p>
                  {connection.connection_strength && connection.connection_strength > 1 && (
                    <p className="text-xs text-blue-600">
                      {connection.connection_strength} connection{connection.connection_strength > 1 ? 's' : ''}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Connections State */}
      {profile.recent_connections.length === 0 && (
        <div className="p-6 bg-gray-50 rounded-xl border-2 border-gray-200 text-center">
          <p className="text-gray-600 mb-2">No connections yet</p>
          <p className="text-sm text-gray-500">
            Start connecting with others to see your history here!
          </p>
        </div>
      )}
    </div>
  );
};
