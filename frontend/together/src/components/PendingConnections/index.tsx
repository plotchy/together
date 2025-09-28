'use client';
import { useState, useEffect, useRef } from 'react';
import { Session } from 'next-auth';
import { apiClient } from '@/lib/api';
import { UserPendingConnectionsResponse, UserOptimisticConnectionsResponse } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';

interface PendingConnectionsProps {
  session: Session | null;
}

export const PendingConnections = ({ session }: PendingConnectionsProps) => {
  const { user } = useProfile();
  const [pendingConnections, setPendingConnections] = useState<UserPendingConnectionsResponse | null>(null);
  const [optimisticConnections, setOptimisticConnections] = useState<UserOptimisticConnectionsResponse | null>(null);
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
        <div className="animate-pulse">
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
  const hasOptimistic = optimisticConnections?.connections.length > 0;
  const unprocessedOptimistic = optimisticConnections?.connections.filter(c => !c.processed) || [];

  return (
    <div className="w-full space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Live Connections</h3>
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
          <span className="text-xs text-gray-500">Live</span>
        </div>
      </div>

      {/* Optimistic Connections (Active Connections) */}
      {unprocessedOptimistic.length > 0 && (
        <div className="p-4 bg-white rounded-xl border-2 border-green-200">
          <h4 className="font-semibold text-green-800 mb-3">
            Active Connections ({unprocessedOptimistic.length})
          </h4>
          <div className="space-y-2">
            {unprocessedOptimistic.map((connection) => (
              <div key={connection.id} className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                <div>
                  <p className="font-medium text-sm">
                    🎯 User #{connection.user_id_1 === user?.id ? connection.user_id_2 : connection.user_id_1}
                  </p>
                  <p className="text-xs text-gray-600">
                    {connection.processed ? 'On-chain confirmed' : 'Waiting for blockchain confirmation'}
                  </p>
                </div>
                <div className="text-right">
                  <div className="flex items-center gap-2">
                    <span className="bg-green-100 text-green-800 text-xs font-semibold px-2 py-1 rounded-full">
                      Connected
                    </span>
                  </div>
                  <p className="text-xs text-gray-500 mt-1">
                    {new Date(connection.created_at).toLocaleDateString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {!hasAnyPending && unprocessedOptimistic.length === 0 && (
        <div className="p-6 bg-gray-50 rounded-xl border-2 border-gray-200 text-center">
          <p className="text-gray-600 mb-2">No active or pending connections</p>
          <p className="text-sm text-gray-500">
            Send a connection request to someone to get started!
          </p>
        </div>
      )}

      {/* Outgoing Connections */}
      {pendingConnections.outgoing.length > 0 && (
        <div className="p-4 bg-white rounded-xl border-2 border-blue-200">
          <h4 className="font-semibold text-blue-800 mb-3">
            Sent Requests ({pendingConnections.outgoing.length})
          </h4>
          <div className="space-y-2">
            {pendingConnections.outgoing.map((connection) => (
              <div key={connection.id} className="flex items-center justify-between p-3 bg-blue-50 rounded-lg">
                <div>
                  <p className="font-medium text-sm">
                    → User #{connection.to_user_id}
                  </p>
                  <p className="text-xs text-gray-600">
                    Waiting for them to send you one back
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-xs font-mono text-blue-600">
                    {formatTimeRemaining(connection.expires_at)}
                  </p>
                  <p className="text-xs text-gray-500">remaining</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Incoming Connections */}
      {pendingConnections.incoming.length > 0 && (
        <div className="p-4 bg-white rounded-xl border-2 border-green-200">
          <h4 className="font-semibold text-green-800 mb-3">
            Received Requests ({pendingConnections.incoming.length})
          </h4>
          <div className="space-y-2">
            {pendingConnections.incoming.map((connection) => (
              <div key={connection.id} className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                <div>
                  <p className="font-medium text-sm">
                    ← User #{connection.from_user_id}
                  </p>
                  <p className="text-xs text-gray-600">
                    Send them one back to connect!
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-xs font-mono text-green-600">
                    {formatTimeRemaining(connection.expires_at)}
                  </p>
                  <p className="text-xs text-gray-500">remaining</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
