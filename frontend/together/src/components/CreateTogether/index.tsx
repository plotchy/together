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
  const { addOptimisticConnection } = useProfile();
  const [partnerAddress, setPartnerAddress] = useState('');
  const [buttonState, setButtonState] = useState<
    'pending' | 'success' | 'failed' | undefined
  >(undefined);
  const [error, setError] = useState<string | null>(null);

  const generateRandomAddress = () => {
    // Generate a random Ethereum address for testing
    const randomHex = Array.from({ length: 40 }, () => 
      Math.floor(Math.random() * 16).toString(16)
    ).join('');
    return `0x${randomHex}`;
  };

  const handleRandomAddress = () => {
    setPartnerAddress(generateRandomAddress());
  };

  const handleCreateAttestation = async () => {
    if (!session?.user?.walletAddress || !partnerAddress) {
      setError('Missing required information');
      return;
    }

    // Validate address format
    if (!/^0x[a-fA-F0-9]{40}$/.test(partnerAddress)) {
      setError('Invalid address format');
      return;
    }

    if (partnerAddress.toLowerCase() === session.user.walletAddress.toLowerCase()) {
      setError('Cannot create attestation with yourself');
      return;
    }

    setButtonState('pending');
    setError(null);

    try {
      // For now, we'll use a simple password. In production, this would be more secure
      const password = 'debug-password';
      const timestamp = Math.floor(Date.now() / 1000);

      const request: AttestTogetherRequest = {
        my_address: session.user.walletAddress,
        partner_address: partnerAddress,
        timestamp,
        password,
        my_username: session.user.username,
        my_profile_picture_url: session.user.profilePictureUrl,
      };

      const response = await apiClient.attestTogether(request);

      if (response.error) {
        setError(response.error);
        setButtonState('failed');
        return;
      }

      if (response.data) {
        // TODO: Use the signature to submit a transaction
        // For now, we'll just show success
        console.log('Attestation signature received:', response.data);
        
        // Optimistically update the profile with the new connection
        addOptimisticConnection(
          partnerAddress,
          undefined, // We don't have partner username for random addresses
          undefined  // We don't have partner profile picture
        );
        
        setButtonState('success');
        setPartnerAddress(''); // Clear the form
        
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

  if (!session?.user?.walletAddress) {
    return (
      <div className="w-full p-4 bg-yellow-50 rounded-xl border-2 border-yellow-200">
        <p className="text-yellow-700 text-sm">Please sign in to create together attestations</p>
      </div>
    );
  }

  return (
    <div className="grid w-full gap-4">
      <p className="text-lg font-semibold">Create Together</p>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Partner Address
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={partnerAddress}
              onChange={(e) => setPartnerAddress(e.target.value)}
              placeholder="0x..."
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <Button
              onClick={handleRandomAddress}
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

        <LiveFeedback
          label={{
            failed: 'Failed to create together attestation',
            pending: 'Creating together attestation...',
            success: 'Together attestation created!',
          }}
          state={buttonState}
          className="w-full"
        >
          <Button
            onClick={handleCreateAttestation}
            disabled={buttonState === 'pending' || !partnerAddress.trim()}
            size="lg"
            variant="primary"
            className="w-full"
          >
            Create Together
          </Button>
        </LiveFeedback>
      </div>

      {/* Debug Info */}
      <div className="p-3 bg-gray-50 rounded-lg">
        <p className="text-xs text-gray-600 mb-1">Your Address:</p>
        <p className="text-xs font-mono text-gray-800">{session.user.walletAddress}</p>
      </div>
    </div>
  );
};
