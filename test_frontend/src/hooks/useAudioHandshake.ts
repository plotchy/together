import { useState, useEffect, useCallback, useRef } from 'react';
import { AudioHandshake, UserProfile, HandshakeMode } from '@/lib/audio-handshake';
import { useAudioLogger, LogEntry } from './useAudioLogger';

interface UseAudioHandshakeReturn {
  isInitialized: boolean;
  isTransmitting: boolean;
  isListening: boolean;
  mode: HandshakeMode;
  receivedProfiles: UserProfile[];
  frequencyData: Uint8Array | null;
  error: string | null;
  logs: LogEntry[];
  initialize: () => Promise<void>;
  transmitProfile: (profile: UserProfile) => Promise<void>;
  startListening: () => void;
  stopListening: () => void;
  stopTransmission: () => void;
  setMode: (mode: HandshakeMode) => void;
  clearReceivedProfiles: () => void;
  clearLogs: () => void;
  cleanup: () => void;
}

export function useAudioHandshake(): UseAudioHandshakeReturn {
  const [isInitialized, setIsInitialized] = useState(false);
  const [isTransmitting, setIsTransmitting] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [mode, setMode] = useState<HandshakeMode>('both');
  const [receivedProfiles, setReceivedProfiles] = useState<UserProfile[]>([]);
  const [frequencyData, setFrequencyData] = useState<Uint8Array | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { logs, addLog, clearLogs } = useAudioLogger();
  const handshakeRef = useRef<AudioHandshake | null>(null);
  const animationFrameRef = useRef<number>();

  // Initialize audio handshake
  const initialize = useCallback(async () => {
    try {
      setError(null);
      
      if (!handshakeRef.current) {
        handshakeRef.current = new AudioHandshake({}, addLog);
      }
      
      await handshakeRef.current.initialize();
      setIsInitialized(true);
      
      // Start frequency visualization
      const updateFrequencyData = () => {
        if (handshakeRef.current && handshakeRef.current.isInitialized()) {
          const data = handshakeRef.current.getFrequencyData();
          setFrequencyData(data);
        }
        animationFrameRef.current = requestAnimationFrame(updateFrequencyData);
      };
      updateFrequencyData();
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to initialize audio');
      setIsInitialized(false);
    }
  }, []);

  // Transmit profile
  const transmitProfile = useCallback(async (profile: UserProfile) => {
    if (!handshakeRef.current || !isInitialized) {
      throw new Error('Audio handshake not initialized');
    }

    try {
      setError(null);
      setIsTransmitting(true);
      await handshakeRef.current.transmitProfile(profile);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to transmit profile');
      setIsTransmitting(false);
    }
  }, [isInitialized]);

  // Start listening
  const startListening = useCallback(() => {
    if (!handshakeRef.current || !isInitialized) {
      setError('Audio handshake not initialized');
      return;
    }

    try {
      setError(null);
      setIsListening(true);
      
      handshakeRef.current.startListening((profile: UserProfile) => {
        setReceivedProfiles(prev => {
          // Avoid duplicates based on user ID
          const exists = prev.some(p => p.id === profile.id);
          if (!exists) {
            return [...prev, profile];
          }
          return prev;
        });
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start listening');
      setIsListening(false);
    }
  }, [isInitialized]);

  // Stop listening
  const stopListening = useCallback(() => {
    if (handshakeRef.current) {
      handshakeRef.current.stopListening();
    }
    setIsListening(false);
  }, []);

  // Stop transmission
  const stopTransmission = useCallback(() => {
    if (handshakeRef.current) {
      handshakeRef.current.stopTransmission();
    }
    setIsTransmitting(false);
  }, []);

  // Clear received profiles
  const clearReceivedProfiles = useCallback(() => {
    setReceivedProfiles([]);
  }, []);

  // Cleanup
  const cleanup = useCallback(() => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }
    
    if (handshakeRef.current) {
      handshakeRef.current.cleanup();
      handshakeRef.current = null;
    }
    
    setIsInitialized(false);
    setIsTransmitting(false);
    setIsListening(false);
    setFrequencyData(null);
    setError(null);
  }, []);

  // Handle mode changes
  useEffect(() => {
    if (!isInitialized) return;

    // Auto-start listening for 'listener' and 'both' modes
    if ((mode === 'listener' || mode === 'both') && !isListening) {
      startListening();
    } else if (mode === 'broadcaster' && isListening) {
      stopListening();
    }
  }, [mode, isInitialized, isListening, startListening, stopListening]);

  // Cleanup on unmount
  useEffect(() => {
    return cleanup;
  }, [cleanup]);

  return {
    isInitialized,
    isTransmitting,
    isListening,
    mode,
    receivedProfiles,
    frequencyData,
    error,
    logs,
    initialize,
    transmitProfile,
    startListening,
    stopListening,
    stopTransmission,
    setMode,
    clearReceivedProfiles,
    clearLogs,
    cleanup
  };
}
