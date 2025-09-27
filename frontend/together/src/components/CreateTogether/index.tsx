'use client';
import { useState } from 'react';
import { Session } from 'next-auth';
import { Button, LiveFeedback } from '@worldcoin/mini-apps-ui-kit-react';
import { apiClient } from '@/lib/api';
import { AttestTogetherRequest } from '@/types/api';
import { useProfile } from '@/contexts/ProfileContext';

interface CreateTogetherProps {
  session: Session | null;
}

export const CreateTogether = ({ session }: CreateTogetherProps) => {
  const { addOptimisticConnection, user } = useProfile();
  const [partnerUserId, setPartnerUserId] = useState('');
  const [buttonState, setButtonState] = useState<
    'pending' | 'success' | 'failed' | undefined
  >(undefined);
  const [error, setError] = useState<string | null>(null);

  const generateRandomUserId = () => {
    // Generate a random user ID for testing (between 1 and 1000)
    return Math.floor(Math.random() * 1000) + 1;
  };

  const handleRandomUserId = () => {
    setPartnerUserId(generateRandomUserId().toString());
  };

  const handleCreatePendingConnection = async () => {
    if (!user?.id || !partnerUserId) {
      setError('Missing required information');
      return;
    }

    // Validate user ID format
    const partnerUserIdNum = parseInt(partnerUserId);
    if (isNaN(partnerUserIdNum) || partnerUserIdNum <= 0) {
      setError('Invalid user ID format');
      return;
    }

    if (partnerUserIdNum === user.id) {
      setError('Cannot create connection with yourself');
      return;
    }

    setButtonState('pending');
    setError(null);

    try {
      const response = await apiClient.createPendingConnection(user.id, {
        to_user_id: partnerUserIdNum
      });

      if (response.error) {
        setError(response.error);
        setButtonState('failed');
        return;
      }

      if (response.data) {
        console.log('Pending connection created:', response.data);
        setButtonState('success');
        setPartnerUserId(''); // Clear the form
        
        // Reset after a delay
        setTimeout(() => {
          setButtonState(undefined);
        }, 3000);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setButtonState('failed');
    }

    // Reset failed state after delay
    if (buttonState === 'failed') {
      setTimeout(() => {
        setButtonState(undefined);
      }, 3000);
    }
  };

  if (!session?.user?.walletAddress || !user) {
    return (
      <div className="w-full p-4 bg-yellow-50 rounded-xl border-2 border-yellow-200">
        <p className="text-yellow-700 text-sm">Please sign in to create pending connections</p>
      </div>
    );
  }

  return (
    <div className="grid w-full gap-4">
      <p className="text-lg font-semibold">Create Pending Connection</p>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Partner User ID
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={partnerUserId}
              onChange={(e) => setPartnerUserId(e.target.value)}
              placeholder="Enter user ID (e.g., 123)"
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <Button
              onClick={handleRandomUserId}
              size="sm"
              variant="tertiary"
              className="px-3"
            >
              Random
            </Button>
          </div>
        </div>

        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-600 text-sm">{error}</p>
          </div>
        )}

        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-blue-700 text-sm">
            <strong>How it works:</strong> You send a pending connection to another user. 
            They need to send one back to you within 10 minutes to complete the connection!
          </p>
        </div>

        <LiveFeedback
          label={{
            failed: 'Failed to create pending connection',
            pending: 'Creating pending connection...',
            success: 'Pending connection created!',
          }}
          state={buttonState}
          className="w-full"
        >
          <Button
            onClick={handleCreatePendingConnection}
            disabled={buttonState === 'pending' || !partnerUserId.trim()}
            size="lg"
            variant="primary"
            className="w-full"
          >
            Send Connection Request
          </Button>
        </LiveFeedback>
      </div>

      {/* Debug Info */}
      <div className="p-3 bg-gray-50 rounded-lg">
        <p className="text-xs text-gray-600 mb-1">Your User ID:</p>
        <p className="text-xs font-semibold text-gray-800">#{user.id}</p>
        <p className="text-xs text-gray-600 mb-1 mt-2">Your Address:</p>
        <p className="text-xs font-mono text-gray-800">{session.user.walletAddress}</p>
      </div>
    </div>
  );
};
