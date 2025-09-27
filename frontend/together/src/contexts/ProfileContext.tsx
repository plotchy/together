'use client';
import React, { createContext, useContext, useState, ReactNode } from 'react';
import { UserProfile, ConnectionInfo } from '@/types/api';

interface ProfileContextType {
  profile: UserProfile | null;
  setProfile: (profile: UserProfile | null) => void;
  addOptimisticConnection: (partnerAddress: string, partnerUsername?: string, partnerProfileUrl?: string) => void;
  clearOptimisticUpdates: () => void;
}

const ProfileContext = createContext<ProfileContextType | undefined>(undefined);

export const useProfile = () => {
  const context = useContext(ProfileContext);
  if (context === undefined) {
    throw new Error('useProfile must be used within a ProfileProvider');
  }
  return context;
};

interface ProfileProviderProps {
  children: ReactNode;
}

export const ProfileProvider = ({ children }: ProfileProviderProps) => {
  const [profile, setProfileState] = useState<UserProfile | null>(null);

  const setProfile = (newProfile: UserProfile | null) => {
    setProfileState(newProfile);
  };

  const addOptimisticConnection = (
    partnerAddress: string, 
    partnerUsername?: string, 
    partnerProfileUrl?: string
  ) => {
    if (!profile) return;

    const newConnection: ConnectionInfo = {
      partner_address: partnerAddress,
      attestation_timestamp: Math.floor(Date.now() / 1000),
      partner_username: partnerUsername,
    };

    const updatedProfile: UserProfile = {
      ...profile,
      total_connections: profile.total_connections + 1,
      recent_connections: [newConnection, ...profile.recent_connections].slice(0, 50), // Keep only latest 50
    };

    setProfileState(updatedProfile);
  };

  const clearOptimisticUpdates = () => {
    // This would be called if we want to revert optimistic updates
    // For now, we'll just refetch the profile data elsewhere
  };

  return (
    <ProfileContext.Provider
      value={{
        profile,
        setProfile,
        addOptimisticConnection,
        clearOptimisticUpdates,
      }}
    >
      {children}
    </ProfileContext.Provider>
  );
};
