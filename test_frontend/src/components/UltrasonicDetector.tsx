'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Slider } from '@/components/ui/slider';
import { Label } from '@/components/ui/label';
import { AudioVisualizer } from './AudioVisualizer';
import { RadioIcon as WaveIcon, Volume2, VolumeX, Mic, Settings } from 'lucide-react';

export function UltrasonicDetector() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [isTransmitting, setIsTransmitting] = useState(false);
  const [ultrasonicLevel, setUltrasonicLevel] = useState(0);
  const [isDetecting, setIsDetecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [frequencyData, setFrequencyData] = useState<Uint8Array | null>(null);
  
  // Detection parameters with sliders
  const [minFreq, setMinFreq] = useState(300); // 300Hz - warm bass range
  const [maxFreq, setMaxFreq] = useState(1200); // 1.2kHz - comfortable upper range
  const [detectionThreshold, setDetectionThreshold] = useState(50);
  const [currentPeakFreq, setCurrentPeakFreq] = useState(0);
  const [transmitFreq, setTransmitFreq] = useState(600); // Current transmit frequency

  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const oscillatorRef = useRef<OscillatorNode | null>(null);
  const gainRef = useRef<GainNode | null>(null);
  const mediaStreamRef = useRef<MediaStream | null>(null);
  const animationFrameRef = useRef<number | undefined>(undefined);
  const isListeningRef = useRef<boolean>(false);

  const startListening = useCallback(() => {
    if (!analyserRef.current) return;

    isListeningRef.current = true;
    const bufferLength = analyserRef.current.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);

    const analyze = () => {
      if (!analyserRef.current || !isListeningRef.current) return;

      analyserRef.current.getByteFrequencyData(dataArray);

      // Calculate frequency bins using slider values
      const sampleRate = 44100;
      const freqBinSize = sampleRate / (2 * bufferLength);
      const minBin = Math.floor(minFreq / freqBinSize);
      const maxBin = Math.floor(maxFreq / freqBinSize);

      // Find the peak in the specified frequency range
      let maxLevel = 0;
      let peakBin = minBin;
      for (let i = minBin; i <= Math.min(bufferLength - 1, maxBin); i++) {
        if (dataArray[i] > maxLevel) {
          maxLevel = dataArray[i];
          peakBin = i;
        }
      }
      
      // Calculate and store the actual peak frequency
      const peakFreq = peakBin * freqBinSize;
      setCurrentPeakFreq(peakFreq);
      
      // Average around the peak
      let sum = 0;
      let count = 0;
      const peakRange = 5; // bins around peak
      for (let i = Math.max(minBin, peakBin - peakRange); i <= Math.min(Math.min(bufferLength - 1, maxBin), peakBin + peakRange); i++) {
        sum += dataArray[i];
        count++;
      }
      
      const averageLevel = count > 0 ? sum / count : 0;
      setUltrasonicLevel(averageLevel);
      
      // Update frequency data for visualizer - copy the array properly
      const frequencyDataCopy = new Uint8Array(dataArray.length);
      frequencyDataCopy.set(dataArray);
      setFrequencyData(frequencyDataCopy);
      
      // Use slider threshold value
      setIsDetecting(averageLevel > detectionThreshold);

      if (isListeningRef.current) {
        animationFrameRef.current = requestAnimationFrame(analyze);
      }
    };

    analyze();
  }, []);

  const initialize = useCallback(async () => {
    try {
      setError(null);
      
      // Create audio context
      const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      audioContextRef.current = new AudioContextClass();

      // Get microphone access
      mediaStreamRef.current = await navigator.mediaDevices.getUserMedia({ audio: true });

      // Setup analyzer for detection
      const source = audioContextRef.current.createMediaStreamSource(mediaStreamRef.current);
      analyserRef.current = audioContextRef.current.createAnalyser();
      analyserRef.current.fftSize = 8192;
      analyserRef.current.smoothingTimeConstant = 0.8; // Better for visualization
      analyserRef.current.minDecibels = -90;
      analyserRef.current.maxDecibels = -10;
      source.connect(analyserRef.current);

      // Setup gain for transmission
      gainRef.current = audioContextRef.current.createGain();
      gainRef.current.gain.value = 0.5; // Loud enough to detect
      gainRef.current.connect(audioContextRef.current.destination);

      // Resume context if needed
      if (audioContextRef.current.state === 'suspended') {
        await audioContextRef.current.resume();
      }

      setIsInitialized(true);
      startListening();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to initialize audio');
    }
  }, [startListening, minFreq, maxFreq, detectionThreshold]);

  const startTransmission = useCallback((frequency?: number) => {
    if (!audioContextRef.current || !gainRef.current) return;

    // Stop any existing transmission first
    if (oscillatorRef.current) {
      oscillatorRef.current.stop();
      oscillatorRef.current.disconnect();
      oscillatorRef.current = null;
    }

    const freq = frequency || transmitFreq;
    setTransmitFreq(freq);
    setIsTransmitting(true);
    
    // Create tone at specified frequency
    oscillatorRef.current = audioContextRef.current.createOscillator();
    oscillatorRef.current.frequency.value = freq;
    oscillatorRef.current.type = 'sine';
    oscillatorRef.current.connect(gainRef.current);
    oscillatorRef.current.start();
  }, [transmitFreq]);

  const stopTransmission = useCallback(() => {
    setIsTransmitting(false);
    
    if (oscillatorRef.current) {
      oscillatorRef.current.stop();
      oscillatorRef.current.disconnect();
      oscillatorRef.current = null;
    }
  }, []);

  const stopListening = useCallback(() => {
    isListeningRef.current = false;
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }
  }, []);

  const cleanup = useCallback(() => {
    stopTransmission();
    stopListening();
    
    if (mediaStreamRef.current) {
      mediaStreamRef.current.getTracks().forEach(track => track.stop());
    }
    
    if (audioContextRef.current) {
      audioContextRef.current.close();
    }
  }, [stopTransmission, stopListening]);

  useEffect(() => {
    return cleanup;
  }, [cleanup]);

  return (
    <div className="max-w-4xl mx-auto space-y-6 p-6">
                <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold flex items-center justify-center gap-2">
          <WaveIcon className="h-8 w-8 text-blue-600" />
          Audio Dial Tone Test
        </h1>
        <p className="text-gray-600">
          Testing comfortable, warm, bassy dial tones for audio handshake
        </p>
      </div>

      {error && (
        <Card className="border-red-200 bg-red-50">
          <CardContent className="p-4">
            <p className="text-red-700">{error}</p>
          </CardContent>
        </Card>
      )}

      {!isInitialized ? (
        <Card>
          <CardContent className="p-6 text-center">
            <Button onClick={initialize} size="lg">
              Initialize Audio
            </Button>
            <p className="text-sm text-gray-500 mt-2">
              This will request microphone permissions
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-6">
          {/* Detection Controls */}
          {isInitialized && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="h-5 w-5" />
                  Detection Settings
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="space-y-2">
                    <Label htmlFor="min-freq">Min Frequency: {minFreq}Hz</Label>
                    <Slider
                      id="min-freq"
                      min={200}
                      max={800}
                      step={50}
                      value={[minFreq]}
                      onValueChange={(value) => setMinFreq(value[0])}
                      className="w-full"
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="max-freq">Max Frequency: {maxFreq}Hz</Label>
                    <Slider
                      id="max-freq"
                      min={800}
                      max={2000}
                      step={50}
                      value={[maxFreq]}
                      onValueChange={(value) => setMaxFreq(value[0])}
                      className="w-full"
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <Label htmlFor="threshold">Threshold: {detectionThreshold}</Label>
                    <Slider
                      id="threshold"
                      min={20}
                      max={150}
                      step={5}
                      value={[detectionThreshold]}
                      onValueChange={(value) => setDetectionThreshold(value[0])}
                      className="w-full"
                    />
                  </div>
                </div>
                
                <div className="text-xs text-gray-600 bg-gray-50 p-3 rounded">
                  <div className="font-medium mb-1">Current Detection:</div>
                  <div>Peak at: <Badge variant="outline">{currentPeakFreq.toFixed(0)}Hz</Badge></div>
                  <div>Signal: <Badge variant="outline">{ultrasonicLevel.toFixed(1)}</Badge></div>
                  <div>Range: {minFreq}-{maxFreq}Hz | Threshold: {detectionThreshold}</div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Audio Spectrum Visualizer */}
          {isInitialized && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <WaveIcon className="h-5 w-5" />
                  Audio Spectrum
                </CardTitle>
              </CardHeader>
              <CardContent>
                <AudioVisualizer 
                  frequencyData={frequencyData}
                  width={600}
                  height={120}
                  minFreq={minFreq}
                  maxFreq={maxFreq}
                  threshold={detectionThreshold}
                />
                <div className="text-xs text-gray-500 mt-2 text-center">
                  Visual representation of audio frequencies
                </div>
              </CardContent>
            </Card>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Transmitter */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Volume2 className="h-5 w-5" />
                  Transmitter
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="text-center space-y-3">
                  {/* Stop button */}
                  {isTransmitting && (
                    <Button
                      onClick={stopTransmission}
                      variant="destructive"
                      size="lg"
                      className="w-full"
                    >
                      <VolumeX className="h-4 w-4 mr-2" />
                      Stop {transmitFreq}Hz Tone
                    </Button>
                  )}
                  
                  {/* Frequency buttons */}
                  <div className="grid grid-cols-2 gap-2">
                    {[350, 440, 600, 800, 941, 1000].map((freq) => (
                      <Button
                        key={freq}
                        onClick={() => startTransmission(freq)}
                        variant={isTransmitting && transmitFreq === freq ? 'destructive' : 'outline'}
                        size="sm"
                        disabled={isTransmitting && transmitFreq === freq}
                        className="relative"
                      >
                        <Volume2 className="h-3 w-3 mr-1" />
                        {freq}Hz
                        {isTransmitting && transmitFreq === freq && (
                          <div className="absolute -top-1 -right-1 w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                        )}
                      </Button>
                    ))}
                  </div>
                </div>
                
                {isTransmitting && (
                  <div className="text-center p-4 bg-blue-50 rounded-lg">
                    <div className="text-blue-700 font-medium">
                      🎵 Broadcasting {transmitFreq}Hz tone
                    </div>
                    <div className="text-xs text-blue-600 mt-1">
                      {transmitFreq <= 1000 ? '(Warm, audible tone)' : '(Audible but higher)'}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Detector */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Mic className="h-5 w-5" />
                  Detector
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="text-center space-y-4">
                  <div className={`p-6 rounded-lg border-2 ${
                    isDetecting 
                      ? 'bg-green-50 border-green-500' 
                      : 'bg-gray-50 border-gray-200'
                  }`}>
                    <div className={`text-2xl font-bold ${
                      isDetecting ? 'text-green-700' : 'text-gray-500'
                    }`}>
                      {isDetecting 
                        ? '🎉 AUDIO DETECTED!' 
                        : '🔇 No audio signal'
                      }
                    </div>
                    
                    <div className="mt-2 space-y-1">
                      <div className="text-sm font-medium">
                        Signal Level: <Badge variant="outline">{ultrasonicLevel.toFixed(1)}</Badge>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full transition-all duration-300 ${
                            isDetecting ? 'bg-green-500' : 'bg-gray-400'
                          }`}
                          style={{ width: `${Math.min(100, (ultrasonicLevel / 100) * 100)}%` }}
                        />
                      </div>
                    </div>
                  </div>

                  <div className="text-xs text-gray-500 space-y-1">
                    <p>Monitoring {minFreq}-{maxFreq}Hz range</p>
                    <p>Threshold: {detectionThreshold}+ for detection</p>
                    <p>Peak at: {currentPeakFreq.toFixed(0)}Hz</p>
                    <p>Current: {ultrasonicLevel.toFixed(1)}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      <Card className="bg-blue-50 border-blue-200">
        <CardHeader>
          <CardTitle className="text-blue-900">How to Test</CardTitle>
        </CardHeader>
        <CardContent className="text-blue-800 space-y-2">
          <ol className="list-decimal list-inside space-y-1 text-sm">
            <li>Click &quot;Initialize Audio&quot; and allow microphone access</li>
            <li>On one device/tab: Click any frequency button (350Hz, 440Hz, etc.)</li>
            <li>On same or different device: Watch the detector</li>
            <li>If it works, you&apos;ll see &quot;AUDIO DETECTED!&quot; when tone is playing</li>
            <li>Try different frequencies, devices, and distances</li>
          </ol>
        </CardContent>
      </Card>
    </div>
  );
}
