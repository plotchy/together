import { auth } from '@/auth';
import { Page } from '@/components/PageLayout';
import { ProfileProvider } from '@/contexts/ProfileContext';
import { HomeContent } from '@/components/HomeContent';

export default async function Home() {
  const session = await auth();

  return (
    <>
      <Page.Main className="flex flex-col items-center min-h-screen p-4 bg-white">
        <div className="text-center max-w-md w-full pt-4">
          <ProfileProvider>
            <HomeContent session={session} />
          </ProfileProvider>
        </div>
      </Page.Main>
    </>
  );
}
