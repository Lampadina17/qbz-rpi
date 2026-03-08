/**
 * Svelte action for cached images.
 *
 * Usage:
 *   <img use:cachedSrc={imageUrl} alt="..." />
 *
 * Loads the image through the backend cache. Shows nothing until resolved,
 * then sets the src attribute. Falls back to original URL on error.
 */

import { getCachedImageUrl } from '$lib/services/imageCacheService';

export function cachedSrc(node: HTMLImageElement, url: string | undefined) {
  let currentUrl = url;

  async function resolve(imageUrl: string | undefined) {
    if (!imageUrl) {
      node.src = '';
      return;
    }

    try {
      const resolved = await getCachedImageUrl(imageUrl);
      // Only apply if still the current URL (avoid race conditions)
      if (imageUrl === currentUrl) {
        node.src = resolved;
      }
    } catch {
      if (imageUrl === currentUrl) {
        node.src = imageUrl; // Fallback to original
      }
    }
  }

  resolve(url);

  return {
    update(newUrl: string | undefined) {
      if (newUrl !== currentUrl) {
        currentUrl = newUrl;
        resolve(newUrl);
      }
    }
  };
}
