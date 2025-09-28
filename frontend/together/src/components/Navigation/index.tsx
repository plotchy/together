'use client';

import { TabItem, Tabs } from '@worldcoin/mini-apps-ui-kit-react';
import { Home, User } from 'iconoir-react';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

/**
 * This component uses the UI Kit to navigate between pages
 * Bottom navigation is the most common navigation pattern in Mini Apps
 * We require mobile first design patterns for mini apps
 * Read More: https://docs.world.org/mini-apps/design/app-guidelines#mobile-first
 */

export const Navigation = () => {
  const pathname = usePathname();
  const router = useRouter();
  const [value, setValue] = useState('home');

  // Update tab based on current path
  useEffect(() => {
    if (pathname.includes('/profile')) {
      setValue('profile');
    } else {
      setValue('home');
    }
  }, [pathname]);

  const handleValueChange = (newValue: string) => {
    setValue(newValue);
    if (newValue === 'home') {
      router.push('/home');
    } else if (newValue === 'profile') {
      router.push('/profile');
    }
  };

  return (
    <Tabs value={value} onValueChange={handleValueChange}>
      <TabItem value="home" icon={<Home />} label="Home" />
      <TabItem value="profile" icon={<User />} label="Profile" />
      {/* <TabItem value="wallet" icon={<Bank />} label="Wallet" /> */}
    </Tabs>
  );
};
