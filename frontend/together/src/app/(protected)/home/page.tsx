import { auth } from '@/auth';
import { Page } from '@/components/PageLayout';
import { ProfileProvider } from '@/contexts/ProfileContext';
import { HomeContent } from '@/components/HomeContent';

export default async function Home() {
  const session = await auth();

  return (
    <>
      <Page.Main className="flex flex-col items-center min-h-screen p-8 gap-8 bg-white">
        <div className="text-center max-w-md w-full pt-16">
          <h1 className="text-5xl font-bold text-gray-900">
            Welcome, {session?.user?.username}
          </h1>
          <div className="mt-20">
            <ProfileProvider>
              <HomeContent session={session} />
            </ProfileProvider>
          </div>
        </div>
      </Page.Main>
    </>
  );
}
