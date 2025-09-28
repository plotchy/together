'use client';
import { CircularIcon, Marble } from '@worldcoin/mini-apps-ui-kit-react';
import { CheckCircleSolid } from 'iconoir-react';
import { Session } from 'next-auth';

/**
 * UserInfo component displays user information including profile picture, username, and verification status.
 * It uses the Marble component from the mini-apps-ui-kit-react library to display the profile picture.
 */
interface UserInfoProps {
  session: Session | null;
}

export const UserInfo = ({ session }: UserInfoProps) => {

  return (
    <div className="flex flex-row items-center justify-start gap-4 rounded-xl w-full border-2 border-gray-200 p-4">
      <Marble src={session?.user?.profilePictureUrl} className="w-14" />
      <div className="flex flex-col items-start justify-center flex-1">
        <div className="flex flex-row items-center justify-center">
          <span className="text-lg font-semibold capitalize">
            {session?.user?.username}
          </span>
          <span className="text-sm text-gray-500 ml-2">
            Together ID: {session?.user?.id || 'N/A'}
          </span>
          {session?.user?.profilePictureUrl && (
            <CircularIcon size="sm" className="ml-2">
              <CheckCircleSolid className="text-blue-600" />
            </CircularIcon>
          )}
        </div>
      </div>
    </div>
  );
};
