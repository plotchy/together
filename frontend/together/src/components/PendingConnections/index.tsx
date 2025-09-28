'use client';
import { useState, useEffect, useRef } from 'react';
import { Session } from 'next-auth';
import { apiClient } from '@/lib/api';
import { UserPendingConnectionsResponse, UserOptimisticConnectionsResponse } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';
import { getUsernamesByAddresses, formatUserDisplay } from '@/utils/username';

interface PendingConnectionsProps {
  session: Session | null;
}

export const PendingConnections = ({ session }: PendingConnectionsProps) => {
  const { user } = useProfile();
  const [pendingConnections, setPendingConnections] = useState<UserPendingConnectionsResponse | null>(null);
  const [optimisticConnections, setOptimisticConnections] = useState<UserOptimisticConnectionsResponse | null>(null);
  const [usernames, setUsernames] = useState<Record<string, string | null>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  const fetchPendingConnections = async () => {
    if (!user?.id) return;

    try {
      const [pendingResponse, optimisticResponse] = await Promise.all([
        apiClient.getUserPendingConnections(user.id),
        apiClient.getUserOptimisticConnections(user.id)
      ]);
      
      if (pendingResponse.error) {
        setError(pendingResponse.error);
      } else if (pendingResponse.data) {
        setPendingConnections(pendingResponse.data);
        setError(null);
      }

      if (optimisticResponse.error) {
        console.warn('Failed to fetch optimistic connections:', optimisticResponse.error);
      } else if (optimisticResponse.data) {
        setOptimisticConnections(optimisticResponse.data);
      }

      // Collect all unique addresses for username fetching
      const addresses = new Set<string>();
      if (pendingResponse.data) {
        pendingResponse.data.outgoing.forEach(conn => {
          if (conn.from_user_address) addresses.add(conn.from_user_address);
          if (conn.to_user_address) addresses.add(conn.to_user_address);
        });
        pendingResponse.data.incoming.forEach(conn => {
          if (conn.from_user_address) addresses.add(conn.from_user_address);
          if (conn.to_user_address) addresses.add(conn.to_user_address);
        });
      }
      if (optimisticResponse.data) {
        optimisticResponse.data.connections.forEach(conn => {
          if (conn.user_1_address) addresses.add(conn.user_1_address);
          if (conn.user_2_address) addresses.add(conn.user_2_address);
        });
      }

      // Fetch usernames for all addresses
      if (addresses.size > 0) {
        try {
          const usernameMap = await getUsernamesByAddresses(Array.from(addresses));
          setUsernames(usernameMap);
        } catch (err) {
          console.warn('Failed to fetch usernames:', err);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!user?.id) return;

    // Initial fetch
    fetchPendingConnections();

    // Set up aggressive polling every 1.5 seconds
    intervalRef.current = setInterval(fetchPendingConnections, 1500);

    // Cleanup interval on unmount
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [user?.id]);

  const formatTimeRemaining = (expiresAt: string) => {
    const now = new Date();
    const expiry = new Date(expiresAt);
    const diffMs = expiry.getTime() - now.getTime();
    
    if (diffMs <= 0) return 'Expired';
    
    const diffSeconds = Math.floor(diffMs / 1000);
    const minutes = Math.floor(diffSeconds / 60);
    const seconds = diffSeconds % 60;
    
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  if (!session?.user?.walletAddress || !user) {
    return (
      <div className="w-full p-4 bg-yellow-50 rounded-xl border-2 border-yellow-200">
        <p className="text-yellow-700 text-sm">Please sign in to view pending connections</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="w-full p-4 bg-white rounded-xl border-2 border-gray-200">
        <div>
          <div className="h-4 bg-gray-300 rounded w-48 mb-3"></div>
          <div className="space-y-2">
            <div className="h-3 bg-gray-300 rounded w-full"></div>
            <div className="h-3 bg-gray-300 rounded w-3/4"></div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full p-4 bg-red-50 rounded-xl border-2 border-red-200">
        <p className="text-red-600 text-sm">Failed to load pending connections: {error}</p>
      </div>
    );
  }

  if (!pendingConnections) {
    return null;
  }

  const hasAnyPending = pendingConnections.outgoing.length > 0 || pendingConnections.incoming.length > 0;
  const allOptimistic = optimisticConnections?.connections || [];

  return (
    <div className="w-full space-y-4">
      {/* <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Live Connections</h3>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-500 rounded-full"></div>
          <span className="text-xs text-gray-500">Live</span>
        </div>
      </div> */}

      {/* All Optimistic Connections */}
      {allOptimistic.length > 0 && (
        <div className="p-6 bg-white rounded-xl border-2 border-green-400 shadow-lg relative overflow-hidden">
          <div className="absolute top-2 right-2 text-green-500">
            🎉
          </div>
          <h4 className="font-bold text-2xl text-green-700 mb-4">
            🎊 Your Connections ({allOptimistic.length}) 🎊
          </h4>
          <div className="space-y-3">
            {allOptimistic.map((connection, index) => (
              <div key={connection.id} className="flex items-center justify-between p-4 bg-gradient-to-r from-green-50 to-emerald-50 rounded-xl border border-green-200 hover:shadow-md transition-all duration-300 hover:scale-102" style={{animationDelay: `${index * 0.1}s`}}>
                <div>
                  <p className="font-semibold text-lg text-gray-800">
                    {(() => {
                      const otherUserId = connection.user_id_1 === user?.id ? connection.user_id_2 : connection.user_id_1;
                      const otherUserAddress = connection.user_id_1 === user?.id ? connection.user_2_address : connection.user_1_address;
                      const otherUsername = otherUserAddress ? usernames[otherUserAddress] : null;
                      return otherUsername || `Together ID #${otherUserId}`;
                    })()}
                  </p>
                  <p className="text-sm text-green-600 font-medium">
                    ✨ Connected together ✨
                  </p>
                </div>
                <div className="text-right">
                  <div className="flex items-center gap-2">
                    <span className="bg-green-500 text-white text-sm font-bold px-3 py-1 rounded-full">
                      🤝 Connected
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 mt-1 font-medium">
                    {new Date(connection.created_at).toLocaleDateString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {!hasAnyPending && allOptimistic.length === 0 && (
        <div className="p-8 bg-white rounded-xl border-2 border-gray-300 text-center relative">
          <div className="text-6xl mb-4">
            😴
          </div>
          <p className="text-xl font-semibold text-gray-700 mb-2">No connections yet!</p>
          <p className="text-lg text-gray-600">
            Go to Home and send a connection request to get started! 🚀
          </p>
        </div>
      )}

      {/* Outgoing Connections */}
      {pendingConnections.outgoing.length > 0 && (
        <div className="p-6 bg-white rounded-xl border-2 border-blue-400 shadow-lg relative">
          <div className="absolute top-2 right-2 text-blue-500">
            📤
          </div>
          <h4 className="font-bold text-2xl text-blue-700 mb-4">
            📨 Sent Requests ({pendingConnections.outgoing.length})
          </h4>
          <div className="space-y-3">
            {pendingConnections.outgoing.map((connection, index) => (
              <div key={connection.id} className="flex items-center justify-between p-4 bg-gradient-to-r from-blue-50 to-cyan-50 rounded-xl border border-blue-200 hover:shadow-md transition-all duration-300" style={{animationDelay: `${index * 0.1}s`}}>
                <div>
                  <p className="font-semibold text-lg text-gray-800">
                    → {connection.to_user_address ? formatUserDisplay(connection.to_user_address, usernames[connection.to_user_address]) : `Together ID #${connection.to_user_id}`}
                  </p>
                  <p className="text-sm text-blue-600 font-medium">
                    ⏳ Waiting for them to send you one back
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-lg font-mono font-bold text-blue-700">
                    {formatTimeRemaining(connection.expires_at)}
                  </p>
                  <p className="text-sm text-gray-600 font-medium">remaining</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Incoming Connections */}
      {pendingConnections.incoming.length > 0 && (
        <div className="p-6 bg-white rounded-xl border-2 border-orange-400 shadow-lg relative">
          <div className="absolute top-2 right-2 text-orange-500">
            📬
          </div>
          <h4 className="font-bold text-2xl text-orange-700 mb-4">
            📬 Received Requests ({pendingConnections.incoming.length})
          </h4>
          <div className="space-y-3">
            {pendingConnections.incoming.map((connection, index) => (
              <div key={connection.id} className="flex items-center justify-between p-4 bg-gradient-to-r from-orange-50 to-yellow-50 rounded-xl border border-orange-200 hover:shadow-md transition-all duration-300 hover:scale-102" style={{animationDelay: `${index * 0.1}s`}}>
                <div>
                  <p className="font-semibold text-lg text-gray-800">
                    ← {connection.from_user_address ? formatUserDisplay(connection.from_user_address, usernames[connection.from_user_address]) : `Together ID #${connection.from_user_id}`}
                  </p>
                  <p className="text-sm text-orange-600 font-medium">
                    🎯 Send them one back to connect!
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-lg font-mono font-bold text-orange-700">
                    {formatTimeRemaining(connection.expires_at)}
                  </p>
                  <p className="text-sm text-gray-600 font-medium">remaining</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
