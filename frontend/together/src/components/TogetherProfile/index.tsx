'use client';
import { useState, useEffect } from 'react';
import { Session } from 'next-auth';
import { Marble } from '@worldcoin/mini-apps-ui-kit-react';
import { apiClient } from '@/lib/api';
import { UserProfile as UserProfileType, ConnectionInfo } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';

interface TogetherProfileProps {
  session: Session | null;
}

export const TogetherProfile = ({ session }: TogetherProfileProps) => {
  const { profile, setProfile } = useProfile();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session?.user?.walletAddress) return;

    const fetchProfile = async () => {
      setLoading(true);
      setError(null);

      const response = await apiClient.getUserProfile(
        session.user.walletAddress,
        {
          username: session.user.username,
          profile_picture_url: session.user.profilePictureUrl,
          limit: 50,
        }
      );

      if (response.error) {
        setError(response.error);
      } else if (response.data) {
        setProfile(response.data);
      }
      setLoading(false);
    };

    fetchProfile();
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
      {/* Together Stats */}
      <div className="p-4 bg-white rounded-xl border-2 border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">Together Stats</h2>
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
                    <p className="font-medium text-sm">
                      {connection.partner_username || 'Anonymous'}
                    </p>
                    <p className="text-xs text-gray-600 font-mono">
                      {connection.partner_address.slice(0, 6)}...{connection.partner_address.slice(-4)}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-xs text-gray-500">
                    {new Date(connection.attestation_timestamp * 1000).toLocaleDateString()}
                  </p>
                  {/* {connection.tx_hash ? (
                    <p className="text-xs text-blue-600 font-mono">
                      {connection.tx_hash.slice(0, 8)}...
                    </p>
                  ) : (
                    <p className="text-xs text-orange-600">
                      Pending...
                    </p>
                  )} */}
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
