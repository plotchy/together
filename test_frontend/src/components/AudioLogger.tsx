'use client';

import { useEffect, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { LogEntry } from '@/hooks/useAudioLogger';
import { Terminal, Trash2 } from 'lucide-react';

interface AudioLoggerProps {
  logs: LogEntry[];
  onClearLogs: () => void;
  className?: string;
}

export function AudioLogger({ logs, onClearLogs, className = '' }: AudioLoggerProps) {
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to latest log
  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = 0; // Scroll to top since newest is at top
    }
  }, [logs]);

  const getLogColor = (type: LogEntry['type']) => {
    switch (type) {
      case 'success':
        return 'text-green-600 bg-green-50 border-green-200';
      case 'warning':
        return 'text-orange-600 bg-orange-50 border-orange-200';
      case 'error':
        return 'text-red-600 bg-red-50 border-red-200';
      default:
        return 'text-blue-600 bg-blue-50 border-blue-200';
    }
  };

  const getBadgeVariant = (type: LogEntry['type']) => {
    switch (type) {
      case 'success':
        return 'default' as const;
      case 'warning':
        return 'secondary' as const;
      case 'error':
        return 'destructive' as const;
      default:
        return 'outline' as const;
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  };

  return (
    <Card className={className}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            Audio Decoder Log
            {logs.length > 0 && (
              <Badge variant="secondary" className="ml-2">
                {logs.length}
              </Badge>
            )}
          </CardTitle>
          {logs.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={onClearLogs}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Clear
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div
          ref={logContainerRef}
          className="max-h-60 overflow-y-auto space-y-2 font-mono text-sm"
        >
          {logs.length === 0 ? (
            <div className="text-center text-gray-500 py-8">
              <Terminal className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No logs yet</p>
              <p className="text-xs">Start transmitting to see decoder activity</p>
            </div>
          ) : (
            logs.map((log) => (
              <div
                key={log.id}
                className={`p-2 rounded border text-xs ${getLogColor(log.type)}`}
              >
                <div className="flex items-center justify-between mb-1">
                  <Badge variant={getBadgeVariant(log.type)} className="text-xs">
                    {log.type.toUpperCase()}
                  </Badge>
                  <span className="text-xs opacity-75">
                    {formatTime(log.timestamp)}
                  </span>
                </div>
                <div className="font-medium">
                  {log.message}
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
