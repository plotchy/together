'use client';
import { useState } from 'react';
import { Session } from 'next-auth';
import { Button, LiveFeedback } from '@worldcoin/mini-apps-ui-kit-react';
import { apiClient } from '@/lib/api';
import { useProfile } from '@/contexts/ProfileContext';

interface CreateTogetherProps {
  session: Session | null;
}

export const CreateTogether = ({ session }: CreateTogetherProps) => {
  const { user } = useProfile();
  const [partnerUserId, setPartnerUserId] = useState('');
  const [buttonState, setButtonState] = useState<
    'pending' | 'success' | 'failed' | undefined
  >(undefined);
  const [error, setError] = useState<string | null>(null);



  const handleCreatePendingConnection = async () => {
    if (!user?.id || !partnerUserId) {
      setError('Missing required information');
      return;
    }

    // Validate together ID format
    const partnerUserIdNum = parseInt(partnerUserId);
    if (isNaN(partnerUserIdNum) || partnerUserIdNum <= 0) {
      setError('Invalid together ID format');
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
        setButtonState(undefined); // Don't change button state on error
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
      setButtonState(undefined); // Don't change button state on error
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
    <div className="w-full space-y-16">
      {user && (
        <div className="text-center space-y-4">
          <p className="text-3xl text-gray-700">Your Together ID is</p>
          <p className="text-6xl font-bold text-blue-600">{user.id}</p>
        </div>
      )}
      
      <div className="space-y-6">
        <div className="text-center space-y-12">
          {/* <p className="text-2xl text-gray-700 mb-20">Put your friend's ID below</p> */}
          <p className="text-2xl text-gray-700" style={{marginBottom: '5rem'}}>Type your friend&apos;s ID below</p>
          <div className="relative">
            {/* Spinning arrows around the input */}
            <div className="absolute -top-8 left-1/2 transform -translate-x-1/2 text-4xl text-red-500 animate-bounce">
              ↓
            </div>
            <div className="absolute -bottom-8 left-1/4 text-3xl text-blue-500" style={{animation: 'bounce 1s infinite 0.5s'}}>
              ↗
            </div>
            <div className="absolute -bottom-8 right-1/4 text-3xl text-indigo-500" style={{animation: 'bounce 1s infinite 1s'}}>
              ↖
            </div>
            <input
              type="text"
              value={partnerUserId}
              onChange={(e) => {
                setPartnerUserId(e.target.value);
                setError(null); // Clear error when user types
              }}
              placeholder="friend&apos;s ID here.."
              className="relative w-full px-4 py-4 border-2 border-gray-400 rounded-xl text-xl text-center text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-4 focus:ring-rainbow focus:border-transparent transition-all duration-300 hover:shadow-lg hover:scale-105 focus:shadow-xl focus:scale-105"
              style={{
                background: 'linear-gradient(45deg, #ffffff 0%, #f8fafc 50%, #ffffff 100%)',
                boxShadow: !partnerUserId ? '0 0 20px rgba(59, 130, 246, 0.3)' : '0 0 30px rgba(34, 197, 94, 0.4)'
              }}
            />
          </div>
        </div>

        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-600 text-sm text-center">{error}</p>
          </div>
        )}

        <LiveFeedback
          label={{
            failed: 'Failed to send request',
            pending: 'Sending request...',
            success: 'Request sent!',
          }}
          state={buttonState}
          className="w-full"
        >
          <Button
            onClick={handleCreatePendingConnection}
            disabled={buttonState === 'pending' || !partnerUserId.trim()}
            size="lg"
            variant="primary"
            className="w-full py-3 text-lg"
          >
            Send Request
          </Button>
        </LiveFeedback>
      </div>
    </div>
  );
};
