// Audio Handshake System
// Uses Web Audio API to transmit and receive data through ultrasonic frequencies

export interface UserProfile {
  id: string;
  name: string;
  avatar?: string;
  data: Record<string, any>;
}

export interface HandshakeConfig {
  baseFrequency: number; // Base frequency for transmission (e.g., 18000 Hz)
  symbolDuration: number; // Duration of each symbol in ms
  sampleRate: number;
  bitRate: number; // Bits per second
}

export type HandshakeMode = 'broadcaster' | 'listener' | 'both';

export class AudioHandshake {
  private audioContext: AudioContext | null = null;
  private analyserNode: AnalyserNode | null = null;
  private oscillator: OscillatorNode | null = null;
  private gainNode: GainNode | null = null;
  private mediaStream: MediaStream | null = null;
  private config: HandshakeConfig;
  private isTransmitting = false;
  private isListening = false;
  private logger?: (type: 'info' | 'success' | 'warning' | 'error', message: string) => void;

  constructor(config: Partial<HandshakeConfig> = {}, logger?: (type: 'info' | 'success' | 'warning' | 'error', message: string) => void) {
    this.config = {
      baseFrequency: 18000, // 18kHz - above human hearing for most adults
      symbolDuration: 15, // 15ms per symbol - maximum speed while maintaining reliability
      sampleRate: 44100,
      bitRate: 66, // 66 bits per second
      ...config
    };
    this.logger = logger;
  }

  async initialize(): Promise<void> {
    try {
      // Check for HTTPS requirement
      if (location.protocol !== 'https:' && location.hostname !== 'localhost' && location.hostname !== '127.0.0.1') {
        throw new Error('Audio features require HTTPS. Please use HTTPS or localhost for testing.');
      }

      // Check for AudioContext support
      const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
      if (!AudioContextClass) {
        throw new Error('Web Audio API is not supported in this browser.');
      }

      this.audioContext = new AudioContextClass();

      // Check for getUserMedia support with fallbacks
      let getUserMedia: ((constraints: MediaStreamConstraints) => Promise<MediaStream>) | null = null;
      
      if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        getUserMedia = (constraints) => navigator.mediaDevices.getUserMedia(constraints);
      } else if ((navigator as any).getUserMedia) {
        getUserMedia = (constraints) => new Promise((resolve, reject) => {
          (navigator as any).getUserMedia(constraints, resolve, reject);
        });
      } else if ((navigator as any).webkitGetUserMedia) {
        getUserMedia = (constraints) => new Promise((resolve, reject) => {
          (navigator as any).webkitGetUserMedia(constraints, resolve, reject);
        });
      } else if ((navigator as any).mozGetUserMedia) {
        getUserMedia = (constraints) => new Promise((resolve, reject) => {
          (navigator as any).mozGetUserMedia(constraints, resolve, reject);
        });
      } else {
        throw new Error('Microphone access is not supported in this browser. Please try Chrome, Firefox, or Safari.');
      }

      // Request microphone permission with mobile-friendly constraints
      const constraints: MediaStreamConstraints = {
        audio: {
          sampleRate: { ideal: this.config.sampleRate, min: 8000 },
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          channelCount: { ideal: 1 }
        }
      };

      this.mediaStream = await getUserMedia(constraints);

      // Setup analyzer for listening
      const source = this.audioContext.createMediaStreamSource(this.mediaStream);
      this.analyserNode = this.audioContext.createAnalyser();
      this.analyserNode.fftSize = 8192;
      this.analyserNode.smoothingTimeConstant = 0.1;
      source.connect(this.analyserNode);

      // Setup gain node for transmission
      this.gainNode = this.audioContext.createGain();
      this.gainNode.gain.value = 0.3; // Increase volume for better detection
      this.gainNode.connect(this.audioContext.destination);

      // Resume audio context if suspended (required on mobile)
      if (this.audioContext.state === 'suspended') {
        await this.audioContext.resume();
      }

    } catch (error) {
      console.error('Failed to initialize audio:', error);
      
      // Provide more helpful error messages
      if (error instanceof Error) {
        if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
          throw new Error('Microphone permission denied. Please allow microphone access and try again.');
        } else if (error.name === 'NotFoundError' || error.name === 'DevicesNotFoundError') {
          throw new Error('No microphone found. Please connect a microphone and try again.');
        } else if (error.name === 'NotSupportedError') {
          throw new Error('Microphone access is not supported in this browser.');
        } else if (error.message.includes('https')) {
          throw error; // Re-throw HTTPS error as-is
        }
      }
      
      throw new Error('Failed to initialize audio system. Please try refreshing the page or use a different browser.');
    }
  }

  // Encode user profile to simple repeated pattern: preamble + digit + suffix (looped)
  private encodeProfile(profile: UserProfile): string {
    // Extract just the single digit from ID (1-9)
    const digit = profile.id;
    const digitBits = parseInt(digit).toString(2).padStart(4, '0'); // 4-bit representation
    
    // Simple pattern: 1111 (preamble) + digit (4 bits) + 0000 (suffix)
    const pattern = '1111' + digitBits + '0000';
    
    // Repeat pattern 10 times for reliability
    return pattern.repeat(10);
  }

  // Decode simple pattern back to user profile
  private decodeProfile(binaryString: string): UserProfile | null {
    try {
      // Look for our pattern: 1111 + 4 digit bits + 0000
      const pattern = /1111([01]{4})0000/;
      const match = binaryString.match(pattern);
      
      if (!match) {
        return null;
      }
      
      // Extract the 4-bit digit and convert to number
      const digitBits = match[1];
      const digit = parseInt(digitBits, 2);
      
      this.logger?.('info', `🔍 Found digit bits: ${digitBits} = ${digit}`);
      
      // Validate digit is 1-9
      if (digit < 1 || digit > 9) {
        this.logger?.('error', `❌ Invalid digit: ${digit} (must be 1-9)`);
        return null;
      }
      
      this.logger?.('success', `✅ Valid digit found: ${digit}`);
      
      // Create simple profile
      return {
        id: digit.toString(),
        name: `User${digit}`,
        data: {}
      };
      
    } catch (error) {
      this.logger?.('error', `❌ Decode error: ${error}`);
      console.error('Failed to decode simple pattern:', error);
      return null;
    }
  }

  // Transmit user profile via audio
  async transmitProfile(profile: UserProfile): Promise<void> {
    if (!this.audioContext || !this.gainNode) {
      throw new Error('Audio not initialized');
    }

    if (this.isTransmitting) {
      this.stopTransmission();
    }

    const binaryData = this.encodeProfile(profile);
    this.isTransmitting = true;
    
    console.log(`🎵 Starting simple transmission - sending digit "${profile.id}" as pattern:`, binaryData.slice(0, 12));

    let bitIndex = 0;
    const transmitBit = () => {
      if (!this.isTransmitting) {
        console.log('✅ Transmission stopped');
        this.stopTransmission();
        return;
      }

      // Loop the transmission - restart when we reach the end
      if (bitIndex >= binaryData.length) {
        bitIndex = 0;
        this.logger?.('info', '🔄 Looping transmission...');
      }

      const bit = binaryData[bitIndex];
      const frequency = bit === '1' ? this.config.baseFrequency + 1000 : this.config.baseFrequency; // 18kHz for 0, 19kHz for 1
      
      if (bitIndex % 10 === 0) {
        console.log(`📡 Bit ${bitIndex}/${binaryData.length}: ${bit} @ ${frequency}Hz`);
      }
      
      // Create oscillator for this symbol
      if (this.oscillator) {
        this.oscillator.stop();
        this.oscillator.disconnect();
      }
      
      this.oscillator = this.audioContext!.createOscillator();
      this.oscillator.frequency.value = frequency;
      this.oscillator.type = 'sine';
      this.oscillator.connect(this.gainNode!);
      this.oscillator.start();
      
      bitIndex++;
      setTimeout(transmitBit, this.config.symbolDuration);
    };

    transmitBit();
  }

  stopTransmission(): void {
    this.isTransmitting = false;
    if (this.oscillator) {
      this.oscillator.stop();
      this.oscillator.disconnect();
      this.oscillator = null;
    }
  }

  // Listen for incoming profiles
  startListening(onProfileReceived: (profile: UserProfile) => void): void {
    if (!this.analyserNode) {
      throw new Error('Audio not initialized');
    }

    this.isListening = true;
    const bufferLength = this.analyserNode.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);
    
    let binaryBuffer = '';
    let lastPatternCheck = '';
    const detectionThreshold = 30; // Much lower threshold for better detection
    
    this.logger?.('info', 'Started listening for audio patterns...');
    
    const analyze = () => {
      if (!this.isListening) return;
      
      this.analyserNode!.getByteFrequencyData(dataArray);
      
      // Check for our frequency ranges - need to be more precise
      const freqBinSize = this.config.sampleRate / (2 * bufferLength);
      const baseBin = Math.floor(this.config.baseFrequency / freqBinSize);
      const highBin = Math.floor((this.config.baseFrequency + 1000) / freqBinSize);
      
      // Debug frequency bin calculation once
      if (binaryBuffer.length === 0) {
        this.logger?.('info', `🔧 Freq bins - Base: ${baseBin} (${this.config.baseFrequency}Hz), High: ${highBin} (${this.config.baseFrequency + 1000}Hz), Bin size: ${freqBinSize.toFixed(1)}Hz`);
      }
      
      // Average a few bins around each frequency for better detection
      const getAverageLevel = (centerBin: number, range: number = 2) => {
        let sum = 0;
        let count = 0;
        for (let i = Math.max(0, centerBin - range); i <= Math.min(bufferLength - 1, centerBin + range); i++) {
          sum += dataArray[i];
          count++;
        }
        return count > 0 ? sum / count : 0;
      };
      
      const baseLevel = getAverageLevel(baseBin);
      const highLevel = getAverageLevel(highBin);
      
      // Much more sensitive detection - log all activity and relax constraints
      const maxLevel = Math.max(baseLevel, highLevel);
      
      // Log frequency levels for debugging
      if (maxLevel > 10) { // Log even very weak signals
        this.logger?.('info', `📡 Signal detected - Base: ${baseLevel.toFixed(1)}, High: ${highLevel.toFixed(1)}, Max: ${maxLevel.toFixed(1)}`);
      }
      
      if (maxLevel > detectionThreshold) {
        const bit = highLevel > baseLevel ? '1' : '0';
        binaryBuffer += bit;
        this.logger?.('info', `🎧 Bit received: ${bit} (base: ${baseLevel.toFixed(1)}, high: ${highLevel.toFixed(1)})`);
        
        // Check for pattern components as we build the buffer
        const recent = binaryBuffer.slice(-12); // Last 12 bits
        if (recent.length >= 4) {
          // Check for preamble
          if (recent.endsWith('1111') && !lastPatternCheck.includes('preamble')) {
            this.logger?.('warning', '🔍 Spotted preamble: 1111');
            lastPatternCheck = 'preamble';
          }
          
          // Check for suffix
          if (recent.endsWith('0000') && lastPatternCheck.includes('preamble')) {
            this.logger?.('info', '🎯 Spotted suffix: 0000');
            lastPatternCheck += '+suffix';
          }
        }
        
        // Try to decode more frequently with shorter buffer
        if (binaryBuffer.length > 12) { // Try decode as soon as we have enough for one pattern
          const profile = this.decodeProfile(binaryBuffer);
          if (profile) {
            this.logger?.('success', `🎉 Successfully decoded profile: User${profile.id}`);
            console.log('🎉 Successfully decoded profile:', profile);
            onProfileReceived(profile);
            binaryBuffer = ''; // Reset buffer after successful decode
            lastPatternCheck = ''; // Reset pattern tracking
          }
          
          // Prevent buffer from growing too large
          if (binaryBuffer.length > 1000) {
            this.logger?.('warning', '🗑️ Buffer too large, trimming...');
            binaryBuffer = binaryBuffer.slice(-500);
            lastPatternCheck = ''; // Reset pattern tracking
          }
        }
      }
      
      requestAnimationFrame(analyze);
    };
    
    analyze();
  }

  stopListening(): void {
    this.isListening = false;
  }

  getFrequencyData(): Uint8Array | null {
    if (!this.analyserNode) return null;
    
    const bufferLength = this.analyserNode.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);
    this.analyserNode.getByteFrequencyData(dataArray);
    return dataArray;
  }

  isInitialized(): boolean {
    return this.audioContext !== null && this.analyserNode !== null;
  }

  cleanup(): void {
    this.stopTransmission();
    this.stopListening();
    
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach(track => track.stop());
    }
    
    if (this.audioContext) {
      this.audioContext.close();
    }
    
    this.audioContext = null;
    this.analyserNode = null;
    this.mediaStream = null;
  }
}

// Generate a random user ID
export function generateUserId(): string {
  return Math.random().toString(36).substr(2, 9);
}

// Create a user profile with just a single random digit (1-9)
export function createUserProfile(name?: string): UserProfile {
  const digit = Math.floor(Math.random() * 9) + 1; // Random digit 1-9
  return {
    id: digit.toString(),
    name: name || `User${digit}`,
    data: {}
  };
}
