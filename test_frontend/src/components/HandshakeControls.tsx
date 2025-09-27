'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { 
  Radio, 
  Mic, 
  MicOff, 
  RadioIcon as Transmit, 
  Pause, 
  Play,
  Users,
  Settings
} from 'lucide-react';
import { HandshakeMode } from '@/lib/audio-handshake';

interface HandshakeControlsProps {
  isInitialized: boolean;
  isTransmitting: boolean;
  isListening: boolean;
  mode: HandshakeMode;
  error: string | null;
  onInitialize: () => Promise<void>;
  onTransmit: () => void;
  onStopTransmission: () => void;
  onStartListening: () => void;
  onStopListening: () => void;
  onModeChange: (mode: HandshakeMode) => void;
}

export function HandshakeControls({
  isInitialized,
  isTransmitting,
  isListening,
  mode,
  error,
  onInitialize,
  onTransmit,
  onStopTransmission,
  onStartListening,
  onStopListening,
  onModeChange
}: HandshakeControlsProps) {
  const [isInitializing, setIsInitializing] = useState(false);

  const handleInitialize = async () => {
    setIsInitializing(true);
    try {
      await onInitialize();
    } finally {
      setIsInitializing(false);
    }
  };

  const getModeIcon = (currentMode: HandshakeMode) => {
    switch (currentMode) {
      case 'broadcaster':
        return <Transmit className="h-4 w-4" />;
      case 'listener':
        return <Radio className="h-4 w-4" />;
      case 'both':
        return <Users className="h-4 w-4" />;
    }
  };

  const getModeDescription = (currentMode: HandshakeMode) => {
    switch (currentMode) {
      case 'broadcaster':
        return 'Send your profile to nearby devices';
      case 'listener':
        return 'Receive profiles from nearby devices';
      case 'both':
        return 'Send and receive profiles simultaneously';
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          Audio Handshake Controls
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Error Display */}
        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}

        {/* Initialization */}
        {!isInitialized && (
          <div className="text-center space-y-3">
            <Button 
              onClick={handleInitialize}
              disabled={isInitializing}
              className="w-full"
              size="lg"
            >
              {isInitializing ? 'Initializing...' : 'Initialize Audio System'}
            </Button>
            <div className="text-xs text-gray-500 space-y-1">
              <p>This will request microphone permissions</p>
              <p className="text-orange-600 font-medium">
                📱 Mobile users: Make sure you're using HTTPS or localhost
              </p>
              <p className="text-blue-600">
                💡 Try Chrome or Safari for best compatibility
              </p>
            </div>
          </div>
        )}

        {isInitialized && (
          <>
            {/* Mode Selection */}
            <div className="space-y-3">
              <Label className="text-sm font-medium">Handshake Mode</Label>
              <div className="grid grid-cols-1 gap-2">
                {(['broadcaster', 'listener', 'both'] as HandshakeMode[]).map((modeOption) => (
                  <div
                    key={modeOption}
                    className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                      mode === modeOption 
                        ? 'border-blue-500 bg-blue-50' 
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => onModeChange(modeOption)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        {getModeIcon(modeOption)}
                        <span className="font-medium capitalize">{modeOption}</span>
                        {mode === modeOption && (
                          <Badge variant="default" className="text-xs">Active</Badge>
                        )}
                      </div>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      {getModeDescription(modeOption)}
                    </p>
                  </div>
                ))}
              </div>
            </div>

            {/* Status Indicators */}
            <div className="flex gap-2">
              {isListening && (
                <Badge variant="outline" className="flex items-center gap-1 bg-green-50">
                  <Mic className="h-3 w-3" />
                  Listening
                </Badge>
              )}
              {isTransmitting && (
                <Badge variant="outline" className="flex items-center gap-1 bg-blue-50">
                  <Transmit className="h-3 w-3" />
                  Transmitting
                </Badge>
              )}
            </div>

            {/* Control Buttons */}
            <div className="space-y-2">
              {(mode === 'broadcaster' || mode === 'both') && (
                <div className="flex gap-2">
                  <Button
                    onClick={onTransmit}
                    disabled={isTransmitting}
                    variant={isTransmitting ? 'secondary' : 'default'}
                    className="flex-1"
                  >
                    <Transmit className="h-4 w-4 mr-2" />
                    {isTransmitting ? 'Transmitting...' : 'Send Profile'}
                  </Button>
                  {isTransmitting && (
                    <Button
                      onClick={onStopTransmission}
                      variant="destructive"
                      size="sm"
                    >
                      <Pause className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              )}

              {(mode === 'listener' || mode === 'both') && (
                <div className="flex gap-2">
                  <Button
                    onClick={isListening ? onStopListening : onStartListening}
                    variant={isListening ? 'secondary' : 'default'}
                    className="flex-1"
                  >
                    {isListening ? (
                      <>
                        <MicOff className="h-4 w-4 mr-2" />
                        Stop Listening
                      </>
                    ) : (
                      <>
                        <Mic className="h-4 w-4 mr-2" />
                        Start Listening
                      </>
                    )}
                  </Button>
                </div>
              )}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
