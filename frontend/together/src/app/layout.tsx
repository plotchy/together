import { auth } from '@/auth';
import ClientProviders from '@/providers';
import '@worldcoin/mini-apps-ui-kit-react/styles.css';
import type { Metadata } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import './globals.css';

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin'],
});

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
});

export const metadata: Metadata = {
  title: 'TogetherApp',
  description: 'TogetherApp lets two World-verified humans prove that they were physically together. Build your social graph of real-world connections and show which ties grow stronger over time.',
  keywords: ['World verification', 'proof of presence', 'real connections', 'social graph', 'meetups', 'IRL'],
  authors: [{ name: 'TogetherApp' }],
  openGraph: {
    title: 'TogetherApp: Proof of Presence with Real Humans',
    description: 'Unlike LinkedIn or Instagram, where every connection looks the same, TogetherApp shows which ties grow stronger over time-highlighting who you\'re truly closest with.',
    type: 'website',
    images: ['/logo_img.webp'],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'TogetherApp: Proof of Presence with Real Humans',
    description: 'Prove real-world connections with World-verified humans. Build meaningful social graphs based on actual meetings.',
    images: ['/logo_img.webp'],
  },
  icons: {
    icon: '/logo_img.webp',
    shortcut: '/logo_img.webp',
    apple: '/logo_img.webp',
  },
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const session = await auth();
  return (
    <html lang="en">
      <body className={`${geistSans.variable} ${geistMono.variable} `}>
        <ClientProviders session={session}>{children}</ClientProviders>
      </body>
    </html>
  );
}
