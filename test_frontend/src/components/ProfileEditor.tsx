'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { UserProfile, createUserProfile } from '@/lib/audio-handshake';
import { Plus, Trash2, User } from 'lucide-react';

interface ProfileEditorProps {
  profile: UserProfile;
  onProfileChange: (profile: UserProfile) => void;
}

export function ProfileEditor({ profile, onProfileChange }: ProfileEditorProps) {
  const [newDataKey, setNewDataKey] = useState('');
  const [newDataValue, setNewDataValue] = useState('');

  const handleNameChange = (name: string) => {
    onProfileChange({ ...profile, name });
  };

  const handleAddData = () => {
    if (newDataKey.trim() && newDataValue.trim()) {
      const updatedProfile = {
        ...profile,
        data: {
          ...profile.data,
          [newDataKey.trim()]: newDataValue.trim()
        }
      };
      onProfileChange(updatedProfile);
      setNewDataKey('');
      setNewDataValue('');
    }
  };

  const handleRemoveData = (key: string) => {
    const { [key]: removed, ...remainingData } = profile.data;
    onProfileChange({
      ...profile,
      data: remainingData
    });
  };

  const handleDataValueChange = (key: string, value: string) => {
    onProfileChange({
      ...profile,
      data: {
        ...profile.data,
        [key]: value
      }
    });
  };

  const generateRandomProfile = () => {
    const newProfile = createUserProfile(); // Just generates random UUID-based profile
    onProfileChange(newProfile);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <User className="h-5 w-5" />
          Your Profile
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Basic Info */}
        <div className="space-y-2">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={profile.name}
            onChange={(e) => handleNameChange(e.target.value)}
            placeholder="Enter your name"
          />
        </div>

        <div className="space-y-2">
          <Label>User ID</Label>
          <Input
            value={profile.id}
            disabled
            className="bg-gray-50 text-gray-600"
          />
          <p className="text-xs text-gray-500">
            This unique ID is automatically generated
          </p>
        </div>

        {/* Additional Data */}
        <div className="space-y-3">
          <Label className="text-sm font-medium">Additional Data</Label>
          
          {/* Existing data fields */}
          {Object.entries(profile.data).map(([key, value]) => (
            <div key={key} className="flex gap-2 items-center">
              <div className="grid grid-cols-2 gap-2 flex-1">
                <Input
                  value={key}
                  disabled
                  className="bg-gray-50 text-sm"
                />
                <Input
                  value={String(value)}
                  onChange={(e) => handleDataValueChange(key, e.target.value)}
                  className="text-sm"
                />
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleRemoveData(key)}
                className="px-2"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          ))}

          {/* Add new data field */}
          <div className="space-y-2 p-3 bg-gray-50 rounded-lg">
            <div className="grid grid-cols-2 gap-2">
              <Input
                placeholder="Key (e.g., email)"
                value={newDataKey}
                onChange={(e) => setNewDataKey(e.target.value)}
                className="text-sm"
              />
              <Input
                placeholder="Value"
                value={newDataValue}
                onChange={(e) => setNewDataValue(e.target.value)}
                className="text-sm"
                onKeyPress={(e) => e.key === 'Enter' && handleAddData()}
              />
            </div>
            <Button
              onClick={handleAddData}
              disabled={!newDataKey.trim() || !newDataValue.trim()}
              size="sm"
              className="w-full"
            >
              <Plus className="h-4 w-4 mr-2" />
              Add Field
            </Button>
          </div>
        </div>

        {/* Quick Actions */}
        <div className="pt-2 border-t">
          <Button
            onClick={generateRandomProfile}
            variant="outline"
            className="w-full"
          >
            Generate Random Profile
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
