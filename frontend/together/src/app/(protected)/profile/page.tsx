import { auth } from '@/auth';
import { Page } from '@/components/PageLayout';
import { TogetherProfile } from '@/components/TogetherProfile';
import { PendingConnections } from '@/components/PendingConnections';
import { ProfileProvider } from '@/contexts/ProfileContext';
import { TopBar, Marble } from '@worldcoin/mini-apps-ui-kit-react';

export default async function Profile() {
  const session = await auth();

  return (
    <>
      <Page.Header className="p-0 text-gray-900">
        <TopBar
          title="Profile"
          endAdornment={
            <div className="flex items-center gap-2">
              <p className="text-sm font-semibold capitalize text-gray-900">
                {session?.user.username}
              </p>
              <Marble src={session?.user.profilePictureUrl} className="w-12" />
            </div>
          }
        />
      </Page.Header>
      <Page.Main className="flex flex-col items-center justify-start gap-4 mb-16 bg-white">
        {/* <UserInfo session={session} /> */}
        <ProfileProvider>
          <TogetherProfile session={session} />
          <PendingConnections session={session} />
        </ProfileProvider>
      </Page.Main>
    </>
  );
}
