'use client';

import { UserProfile } from '@/lib/audio-handshake';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { User, Clock, Wifi } from 'lucide-react';

interface UserProfileCardProps {
  profile: UserProfile;
  isLocal?: boolean;
  className?: string;
}

export function UserProfileCard({ profile, isLocal = false, className = '' }: UserProfileCardProps) {
  return (
    <Card className={`${className} ${isLocal ? 'border-blue-500 bg-blue-50' : 'border-green-500 bg-green-50'}`}>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm">
          <User className="h-4 w-4" />
          {profile.name}
          {isLocal && (
            <Badge variant="outline" className="text-xs">
              You
            </Badge>
          )}
          {!isLocal && (
            <Badge variant="outline" className="text-xs bg-green-100">
              <Wifi className="h-3 w-3 mr-1" />
              Received
            </Badge>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        <div className="space-y-2">
          <div className="text-xs text-gray-600">
            <strong>ID:</strong> {profile.id}
          </div>
          
          {profile.avatar && (
            <div className="flex justify-center">
              <img 
                src={profile.avatar} 
                alt={`${profile.name}'s avatar`}
                className="w-16 h-16 rounded-full object-cover"
              />
            </div>
          )}
          
          {Object.keys(profile.data).length > 0 && (
            <div className="space-y-1">
              <div className="text-xs font-medium text-gray-700">Additional Data:</div>
              <div className="bg-white p-2 rounded text-xs">
                {Object.entries(profile.data).map(([key, value]) => (
                  <div key={key} className="flex justify-between">
                    <span className="font-medium">{key}:</span>
                    <span className="text-gray-600">
                      {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {!isLocal && (
            <div className="flex items-center gap-1 text-xs text-gray-500">
              <Clock className="h-3 w-3" />
              Received just now
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
