import { MiniKit } from '@worldcoin/minikit-js';

/**
 * Fetches username for a given wallet address using MiniKit
 */
export async function getUsernameByAddress(address: string): Promise<string | null> {
  try {
    const user = await MiniKit.getUserByAddress(address);
    return user?.username || null;
  } catch (error) {
    console.warn(`Failed to fetch username for address ${address}:`, error);
    return null;
  }
}

/**
 * Fetches multiple usernames for an array of addresses
 */
export async function getUsernamesByAddresses(addresses: string[]): Promise<Record<string, string | null>> {
  const results: Record<string, string | null> = {};
  
  // Fetch usernames in parallel
  const promises = addresses.map(async (address) => {
    const username = await getUsernameByAddress(address);
    return { address, username };
  });
  
  const resolvedResults = await Promise.allSettled(promises);
  
  resolvedResults.forEach((result) => {
    if (result.status === 'fulfilled' && result.value) {
      results[result.value.address] = result.value.username;
    }
  });
  
  return results;
}

/**
 * Formats display text - shows username if available, otherwise shows formatted address
 */
export function formatUserDisplay(address: string, username?: string | null): string {
  if (username) {
    return username;
  }
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
