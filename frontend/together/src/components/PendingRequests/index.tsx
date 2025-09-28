'use client';
import { useState, useEffect, useRef } from 'react';
import { Session } from 'next-auth';
import { apiClient } from '@/lib/api';
import { UserPendingConnectionsResponse, UserOptimisticConnectionsResponse } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';
import { getUsernamesByAddresses, formatUserDisplay } from '@/utils/username';

interface PendingRequestsProps {
  session: Session | null;
}

export const PendingRequests = ({ session }: PendingRequestsProps) => {
  const { user } = useProfile();
  const [pendingConnections, setPendingConnections] = useState<UserPendingConnectionsResponse | null>(null);
  const [optimisticConnections, setOptimisticConnections] = useState<UserOptimisticConnectionsResponse | null>(null);
  const [usernames, setUsernames] = useState<Record<string, string | null>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const fetchedAddressesRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    if (!user?.id) return;

    const fetch = async () => {
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

        // Only fetch usernames for new addresses we haven't seen before
        const newAddresses = Array.from(addresses).filter(addr => !fetchedAddressesRef.current.has(addr));
        if (newAddresses.length > 0) {
          try {
            const usernameMap = await getUsernamesByAddresses(newAddresses);
            setUsernames(prev => ({ ...prev, ...usernameMap }));
            // Mark these addresses as fetched
            newAddresses.forEach(addr => fetchedAddressesRef.current.add(addr));
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

    // Initial fetch
    fetch();

    // Set up aggressive polling every 1.5 seconds
    intervalRef.current = setInterval(fetch, 1500);

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
    return null;
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
        <p className="text-red-600 text-sm">Failed to load pending requests: {error}</p>
      </div>
    );
  }

  if (!pendingConnections) {
    return null;
  }

  const hasAnyPending = pendingConnections.outgoing.length > 0 || pendingConnections.incoming.length > 0;
  const latestConnection = optimisticConnections?.connections 
    ? optimisticConnections.connections
        .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
        .slice(0, 1)[0]
    : null;

  // Show component if there are pending requests OR a latest connection
  if (!hasAnyPending && !latestConnection) {
    return null;
  }

  // Limit to latest 3 for outgoing and incoming
  const limitedOutgoing = pendingConnections.outgoing.slice(0, 3);
  const limitedIncoming = pendingConnections.incoming.slice(0, 3);

  return (
    <div className="w-full space-y-4">
      {/* Outgoing Connections */}
      {limitedOutgoing.length > 0 && (
        <div className="p-6 bg-white rounded-xl border-2 border-blue-400 shadow-lg relative">
          <div className="absolute top-2 right-2 text-blue-500">
            📤
          </div>
          <h4 className="font-bold text-2xl text-blue-700 mb-4">
            📨 Sent Requests ({pendingConnections.outgoing.length > 3 ? '3+' : pendingConnections.outgoing.length})
          </h4>
          <div className="space-y-3">
            {limitedOutgoing.map((connection, index) => (
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
      {limitedIncoming.length > 0 && (
        <div className="p-6 bg-white rounded-xl border-2 border-orange-400 shadow-lg relative">
          <div className="absolute top-2 right-2 text-orange-500">
            📬
          </div>
          <h4 className="font-bold text-2xl text-orange-700 mb-4">
            📬 Received Requests ({pendingConnections.incoming.length > 3 ? '3+' : pendingConnections.incoming.length})
          </h4>
          <div className="space-y-3">
            {limitedIncoming.map((connection, index) => (
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

      {/* Latest Connection */}
      {latestConnection && (
        <div className="p-6 bg-white rounded-xl border-2 border-green-400 shadow-lg relative overflow-hidden">
          <h4 className="font-bold text-2xl text-green-700 mb-4">
            🤝 Your Latest Connection
          </h4>
          <div className="flex items-center justify-between p-4 bg-gradient-to-r from-green-50 to-emerald-50 rounded-xl border border-green-200 hover:shadow-md transition-all duration-300 hover:scale-102">
            <div>
              <p className="font-semibold text-lg text-gray-800">
                {(() => {
                  const otherUserId = latestConnection.user_id_1 === user?.id ? latestConnection.user_id_2 : latestConnection.user_id_1;
                  const otherUserAddress = latestConnection.user_id_1 === user?.id ? latestConnection.user_2_address : latestConnection.user_1_address;
                  const otherUsername = otherUserAddress ? usernames[otherUserAddress] : null;
                  return otherUsername || `Together ID #${otherUserId}`;
                })()}
              </p>
            </div>
            <div className="text-right">
              <div className="flex items-center gap-2">
                <span className="bg-green-500 text-white text-sm font-bold px-3 py-1 rounded-full">
                  Connected
                </span>
              </div>
              <p className="text-sm text-gray-600 mt-1 font-medium">
                {new Date(latestConnection.created_at).toLocaleDateString()}
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
