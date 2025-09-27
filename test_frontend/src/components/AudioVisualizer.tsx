'use client';

import { useEffect, useRef } from 'react';

interface AudioVisualizerProps {
  frequencyData: Uint8Array | null;
  width?: number;
  height?: number;
  className?: string;
  minFreq?: number;
  maxFreq?: number;
  threshold?: number;
}

export function AudioVisualizer({ 
  frequencyData, 
  width = 300, 
  height = 100, 
  className = '',
  minFreq = 300,
  maxFreq = 1200,
  threshold = 50
}: AudioVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !frequencyData) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw frequency bars
    const barWidth = width / frequencyData.length;
    
    for (let i = 0; i < frequencyData.length; i++) {
      const barHeight = (frequencyData[i] / 255) * height;
      
      // Create gradient for bars
      const gradient = ctx.createLinearGradient(0, height, 0, height - barHeight);
      gradient.addColorStop(0, '#3b82f6'); // Blue
      gradient.addColorStop(0.5, '#8b5cf6'); // Purple
      gradient.addColorStop(1, '#f59e0b'); // Orange
      
      ctx.fillStyle = gradient;
      ctx.fillRect(i * barWidth, height - barHeight, barWidth - 1, barHeight);
    }

    // Highlight detection frequency range
    const sampleRate = 44100;
    const freqBinSize = sampleRate / (2 * frequencyData.length);
    
    const minBin = Math.floor(minFreq / freqBinSize);
    const maxBin = Math.floor(maxFreq / freqBinSize);
    
    // Draw red boxes around detection frequency range
    if (minBin < frequencyData.length) {
      ctx.strokeStyle = '#ef4444';
      ctx.lineWidth = 2;
      // Min frequency box
      ctx.strokeRect(minBin * barWidth, 0, barWidth * 2, height);
      // Max frequency box  
      if (maxBin < frequencyData.length) {
        ctx.strokeRect(maxBin * barWidth, 0, barWidth * 2, height);
      }
    }

    // Draw threshold line
    const thresholdY = height - (threshold / 255) * height;
    ctx.strokeStyle = '#fbbf24'; // Yellow
    ctx.lineWidth = 1;
    ctx.setLineDash([5, 5]); // Dotted line
    ctx.beginPath();
    ctx.moveTo(0, thresholdY);
    ctx.lineTo(width, thresholdY);
    ctx.stroke();
    ctx.setLineDash([]); // Reset line dash
  }, [frequencyData, width, height, minFreq, maxFreq, threshold]);

  return (
    <div className={`bg-black rounded-lg p-4 ${className}`}>
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        className="w-full h-full audio-canvas"
      />
      <div className="text-xs text-gray-400 mt-2 text-center">
        Audio Spectrum (Red boxes: Detection range | Yellow line: Threshold)
      </div>
    </div>
  );
}
