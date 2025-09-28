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
    <div className="flex flex-row items-center justify-start gap-6 rounded-xl w-full border-2 border-purple-400 p-6 bg-white shadow-lg relative overflow-hidden">
      <div className="absolute top-2 right-2 text-purple-500 animate-spin" style={{animationDuration: '4s'}}>
        ⭐
      </div>
      <Marble src={session?.user?.profilePictureUrl} className="w-16 hover:scale-110 transition-transform duration-300" />
      <div className="flex flex-col items-start justify-center flex-1">
        <div className="flex flex-row items-center justify-center">
          <span className="text-2xl font-bold capitalize text-gray-800">
            {session?.user?.username}
          </span>
          {session?.user?.profilePictureUrl && (
            <CircularIcon size="sm" className="ml-3 animate-pulse">
              <CheckCircleSolid className="text-green-600" />
            </CircularIcon>
          )}
        </div>
        <p className="text-sm text-purple-600 font-medium mt-1">
          ✨ Verified Together User ✨
        </p>
      </div>
    </div>
  );
};
