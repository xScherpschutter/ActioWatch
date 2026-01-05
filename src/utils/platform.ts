import { platform } from '@tauri-apps/plugin-os';

let cachedPlatform: string | null = null;

/**
 * Get the current platform
 * @returns The platform name (e.g., 'windows', 'linux', 'macos')
 */
export async function getPlatform(): Promise<string> {
  if (!cachedPlatform) {
    cachedPlatform = await platform();
  }
  return cachedPlatform;
}

/**
 * Check if the current platform is Windows
 * @returns true if running on Windows, false otherwise
 */
export async function isWindows(): Promise<boolean> {
  const platformName = await getPlatform();
  return platformName === 'windows';
}

/**
 * Check if the current platform is Linux
 * @returns true if running on Linux, false otherwise
 */
export async function isLinux(): Promise<boolean> {
  const platformName = await getPlatform();
  return platformName === 'linux';
}
