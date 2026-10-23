/**
 * Sovereign Fast File Finder - Tauri v2 Silent Background Auto-Updater
 * 
 * Checks for updates on application launch against GitHub Releases (Rolling Latest).
 * If a new signed version exists:
 * 1. Downloads the signed binary in background without blocking the UI.
 * 2. In-place replaces the installation files on customer's computer.
 * 3. Prompts or cleanly relaunches the application with zero manual user effort.
 */

export interface UpdateStatus {
  isChecking: boolean;
  updateAvailable: boolean;
  version?: string;
  downloadProgress?: number;
  error?: string;
}

export async function checkAndApplyUpdates(
  onProgress?: (status: UpdateStatus) => void,
  silent: boolean = true
): Promise<boolean> {
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const { relaunch } = await import('@tauri-apps/plugin-process');

    if (onProgress) {
      onProgress({ isChecking: true, updateAvailable: false });
    }

    const update = await check();

    if (!update) {
      if (onProgress) {
        onProgress({ isChecking: false, updateAvailable: false });
      }
      return false;
    }

    if (update.available) {
      if (onProgress) {
        onProgress({
          isChecking: false,
          updateAvailable: true,
          version: update.version,
        });
      }

      let downloadedBytes = 0;
      let totalBytes = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            if (onProgress && totalBytes > 0) {
              const progress = Math.round((downloadedBytes / totalBytes) * 100);
              onProgress({
                isChecking: false,
                updateAvailable: true,
                version: update.version,
                downloadProgress: progress,
              });
            }
            break;
          case 'Finished':
            break;
        }
      });

      await relaunch();
      return true;
    }

    return false;
  } catch (err: any) {
    if (!silent) {
      console.warn('[AutoUpdater] Update check skipped or failed:', err?.message || err);
    }
    if (onProgress) {
      onProgress({
        isChecking: false,
        updateAvailable: false,
        error: err?.message || 'Update check skipped',
      });
    }
    return false;
  }
}
