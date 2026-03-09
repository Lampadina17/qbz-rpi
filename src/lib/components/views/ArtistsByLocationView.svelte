<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { t } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { ArrowLeft, Loader2, Music, Search, X, LayoutGrid, PanelLeftClose, Mic2, Disc3 } from 'lucide-svelte';
  import VirtualizedFavoritesArtistGrid from '../VirtualizedFavoritesArtistGrid.svelte';
  import VirtualizedFavoritesArtistList from '../VirtualizedFavoritesArtistList.svelte';
  import AlbumCard from '../AlbumCard.svelte';
  import { categorizeAlbum } from '$lib/adapters/qobuzAdapters';
  import type { QobuzAlbum } from '$lib/types';

  interface LocationContext {
    sourceArtistMbid: string;
    sourceArtistName: string;
    sourceArtistType: 'Person' | 'Group' | 'Other';
    location: {
      city?: string;
      areaId?: string;
      country?: string;
      countryCode?: string;
      displayName: string;
      precision: 'city' | 'state' | 'country';
    };
    affinitySeeds: {
      genres: string[];
      tags: string[];
      normalizedSeeds: string[];
    };
  }

  interface LocationCandidate {
    mbid: string;
    mb_name: string;
    qobuz_id?: number;
    qobuz_name?: string;
    qobuz_image?: string;
    score: number;
    genres: string[];
  }

  interface LocationDiscoveryResponse {
    artists: LocationCandidate[];
    scene_label: string;
    genre_summary: string;
    total_candidates: number;
    has_more: boolean;
    next_offset: number;
  }

  interface FavoriteArtist {
    id: number;
    name: string;
    image?: { small?: string; thumbnail?: string; large?: string };
    albums_count?: number;
  }

  interface ArtistGroup {
    key: string;
    id: string;
    artists: FavoriteArtist[];
  }

  interface Props {
    context: LocationContext;
    onBack: () => void;
    onArtistClick: (artistId: number) => void;
    onAlbumClick?: (albumId: string) => void;
    onAlbumPlay?: (albumId: string) => void;
  }

  let { context, onBack, onArtistClick, onAlbumClick, onAlbumPlay }: Props = $props();

  // Discovery state
  let loading = $state(true);
  let error = $state<string | null>(null);
  let artists = $state<LocationCandidate[]>([]);
  let sceneLabel = $state('');
  let genreSummary = $state('');
  let totalCandidates = $state(0);
  let hasMore = $state(false);
  let nextOffset = $state(0);
  let loadingMore = $state(false);

  // View state
  type ViewMode = 'grid' | 'sidepanel';
  let viewMode = $state<ViewMode>('grid');
  let searchQuery = $state('');
  let searchExpanded = $state(false);
  let groupingEnabled = $state(false);

  // Sidepanel state
  let selectedArtist = $state<FavoriteArtist | null>(null);
  let selectedArtistAlbums = $state<QobuzAlbum[]>([]);
  let loadingAlbums = $state(false);
  let albumsError = $state<string | null>(null);

  // Dynamic loading state
  let loadingStep = $state(0);
  let loadingStepTimer: ReturnType<typeof setInterval> | null = null;

  const loadingSteps = [
    () => {
      const genre = context.affinitySeeds.genres[0] || context.affinitySeeds.tags[0] || 'music';
      return $t('artist.sceneStep1', { values: { genre } });
    },
    () => {
      const genres = context.affinitySeeds.genres.slice(0, 3);
      const count = genres.length * 50;
      return $t('artist.sceneStep2', { values: { count: String(count) } });
    },
    () => $t('artist.sceneStep3'),
    () => $t('artist.sceneStep4'),
  ];

  function startLoadingAnimation() {
    loadingStep = 0;
    loadingStepTimer = setInterval(() => {
      if (loadingStep < loadingSteps.length - 1) {
        loadingStep++;
      }
    }, 3000);
  }

  function stopLoadingAnimation() {
    if (loadingStepTimer) {
      clearInterval(loadingStepTimer);
      loadingStepTimer = null;
    }
  }

  // Flag URL from circle-flags CDN
  const flagUrl = $derived(
    context.location.countryCode
      ? `https://hatscripts.github.io/circle-flags/flags/${context.location.countryCode}.svg`
      : null
  );

  // Convert candidates to FavoriteArtist format
  function candidatesToFavoriteArtists(candidates: LocationCandidate[]): FavoriteArtist[] {
    return candidates
      .filter((candidate) => candidate.qobuz_id != null)
      .map((candidate) => ({
        id: candidate.qobuz_id!,
        name: candidate.qobuz_name || candidate.mb_name,
        image: candidate.qobuz_image ? { small: candidate.qobuz_image } : undefined,
      }));
  }

  // Client-side search filter
  let allArtists = $derived(candidatesToFavoriteArtists(artists));
  let filteredArtists = $derived.by(() => {
    if (!searchQuery.trim()) return allArtists;
    const query = searchQuery.toLowerCase();
    return allArtists.filter((artist) => artist.name.toLowerCase().includes(query));
  });

  // Grouping
  function alphaGroupKey(name: string): string {
    const first = name.charAt(0).toUpperCase();
    return /[A-Z]/.test(first) ? first : '#';
  }

  function groupArtists(items: FavoriteArtist[]): ArtistGroup[] {
    const sorted = [...items].sort((a, b) => a.name.localeCompare(b.name));
    const groups = new Map<string, FavoriteArtist[]>();
    for (const artist of sorted) {
      const key = alphaGroupKey(artist.name);
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)?.push(artist);
    }
    const keys = [...groups.keys()].sort((a, b) => {
      if (a === '#') return -1;
      if (b === '#') return 1;
      return a.localeCompare(b);
    });
    return keys.map((key) => ({
      key,
      id: `scene-alpha-${key}`,
      artists: groups.get(key) ?? [],
    }));
  }

  let groups = $derived.by(() => {
    if (groupingEnabled) return groupArtists(filteredArtists);
    return [{ key: '', id: 'scene-all', artists: filteredArtists }];
  });

  // Album categorization for sidepanel
  let groupedAlbums = $derived.by(() => {
    if (!selectedArtist) return { discography: [], epsSingles: [], liveAlbums: [] };
    const discography: QobuzAlbum[] = [];
    const epsSingles: QobuzAlbum[] = [];
    const liveAlbums: QobuzAlbum[] = [];
    for (const album of selectedArtistAlbums) {
      const category = categorizeAlbum(album, selectedArtist.id);
      switch (category) {
        case 'albums': discography.push(album); break;
        case 'eps': epsSingles.push(album); break;
        case 'live': liveAlbums.push(album); break;
      }
    }
    return { discography, epsSingles, liveAlbums };
  });

  let totalDisplayedAlbums = $derived(
    groupedAlbums.discography.length + groupedAlbums.epsSingles.length + groupedAlbums.liveAlbums.length
  );

  // Sidepanel artist select
  async function handleArtistSelect(artist: FavoriteArtist) {
    selectedArtist = artist;
    selectedArtistAlbums = [];
    loadingAlbums = true;
    albumsError = null;

    try {
      const result = await invoke<{ items: QobuzAlbum[] }>('get_artist_albums', {
        artistId: artist.id,
        limit: 500,
        offset: 0,
      });
      selectedArtistAlbums = result.items || [];
    } catch (err) {
      console.error('[SceneView] Failed to load artist albums:', err);
      albumsError = String(err);
    } finally {
      loadingAlbums = false;
    }
  }

  async function discoverArtists(offset: number = 0) {
    try {
      const response: LocationDiscoveryResponse = await invoke('v2_discover_artists_by_location', {
        sourceMbid: context.sourceArtistMbid,
        areaId: context.location.areaId,
        areaName: context.location.city || context.location.displayName,
        country: context.location.country || null,
        genres: context.affinitySeeds.genres,
        tags: context.affinitySeeds.tags,
        limit: 50,
        offset,
      });

      if (offset === 0) {
        artists = response.artists;
      } else {
        const existingIds = new Set(artists.map((a) => a.mbid));
        const newArtists = response.artists.filter((a) => !existingIds.has(a.mbid));
        artists = [...artists, ...newArtists];
      }
      sceneLabel = response.scene_label;
      genreSummary = response.genre_summary;
      totalCandidates = response.total_candidates;
      hasMore = response.has_more;
      nextOffset = response.next_offset;
    } catch (err) {
      console.error('[ArtistsByLocationView] Discovery failed:', err);
      error = String(err);
    }
  }

  async function loadMore() {
    if (loadingMore || !hasMore) return;
    loadingMore = true;
    await discoverArtists(nextOffset);
    loadingMore = false;
  }

  onMount(async () => {
    loading = true;
    error = null;
    startLoadingAnimation();
    await discoverArtists();
    stopLoadingAnimation();
    loading = false;
  });

  onDestroy(() => {
    stopLoadingAnimation();
  });
</script>

<div class="scene-view">
  <!-- Top bar with back button -->
  <div class="top-bar">
    <button class="back-btn" onclick={onBack}>
      <ArrowLeft size={16} />
      <span>{$t('actions.back')}</span>
    </button>
  </div>

  <!-- Header with flag -->
  <div class="header">
    {#if flagUrl}
      <img src={flagUrl} alt="" class="flag-icon" />
    {/if}
    <h1>{sceneLabel || context.location.country || context.location.displayName}</h1>
    {#if !loading && filteredArtists.length > 0}
      <span class="results-count">
        {filteredArtists.length}{searchQuery ? ` / ${allArtists.length}` : ''} {$t('nav.artists').toLowerCase()}
      </span>
    {/if}
    <div class="header-search">
      {#if !searchExpanded}
        <button class="search-icon-btn" onclick={() => (searchExpanded = true)} title={$t('nav.search')}>
          <Search size={16} />
        </button>
      {:else}
        <div class="search-expanded">
          <Search size={16} class="search-icon-inline" />
          <!-- svelte-ignore a11y_autofocus -->
          <input
            type="text"
            placeholder={$t('placeholders.search')}
            bind:value={searchQuery}
            class="search-input-inline"
            autofocus
          />
          <button class="search-clear-btn" onclick={() => { searchQuery = ''; searchExpanded = false; }}>
            <X size={14} />
          </button>
        </div>
      {/if}
    </div>
  </div>

  <!-- Subtitle -->
  {#if genreSummary || context.affinitySeeds.genres.length > 0}
    <div class="scene-subtitle">
      {$t('artist.sceneBased', {
        values: {
          artist: context.sourceArtistName,
          genres: genreSummary || context.affinitySeeds.genres.slice(0, 3).join(' / '),
        },
      })}
    </div>
  {/if}

  <!-- Controls bar -->
  {#if !loading && !error && allArtists.length > 0}
    <div class="controls-bar">
      <div class="nav-right">
        <button
          class="control-btn"
          class:active={groupingEnabled}
          onclick={() => (groupingEnabled = !groupingEnabled)}
        >
          <span>{groupingEnabled ? 'A-Z' : 'Group'}</span>
        </button>
        <button
          class="control-btn icon-only"
          onclick={() => {
            if (viewMode === 'grid') {
              viewMode = 'sidepanel';
            } else {
              viewMode = 'grid';
              selectedArtist = null;
            }
          }}
          title={viewMode === 'grid' ? 'Browse view' : 'Grid view'}
        >
          {#if viewMode === 'grid'}
            <PanelLeftClose size={16} />
          {:else}
            <LayoutGrid size={16} />
          {/if}
        </button>
      </div>
    </div>
  {/if}

  <!-- Content -->
  <div class="scene-content">
    {#if loading}
      <div class="scene-loading">
        <div class="loading-visual">
          <div class="loading-pulse">
            <Music size={28} />
          </div>
        </div>
        <div class="loading-status">
          {#key loadingStep}
            <span class="loading-text fade-in">
              {loadingSteps[loadingStep]()}
            </span>
          {/key}
        </div>
        <div class="loading-dots">
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>
      </div>
    {:else if error}
      <div class="scene-error">
        <p>{error}</p>
        <button class="retry-button" onclick={() => { loading = true; error = null; startLoadingAnimation(); discoverArtists().then(() => { stopLoadingAnimation(); loading = false; }); }}>
          {$t('actions.retry')}
        </button>
      </div>
    {:else if allArtists.length === 0}
      <div class="scene-empty">
        <Music size={48} />
        <p>{$t('artist.noSceneResults')}</p>
      </div>
    {:else if viewMode === 'grid'}
      <div class="scene-grid-container">
        <VirtualizedFavoritesArtistGrid
          {groups}
          showGroupHeaders={groupingEnabled}
          onArtistClick={(id) => onArtistClick(id)}
        />
      </div>

      {#if hasMore}
        <div class="load-more-container">
          <button class="load-more-button" onclick={loadMore} disabled={loadingMore}>
            {#if loadingMore}
              <Loader2 size={16} class="spin" />
            {/if}
            <span>
              {$t('actions.loadMore')}
              ({allArtists.length} / {totalCandidates})
            </span>
          </button>
        </div>
      {/if}
    {:else}
      <!-- Sidepanel mode: artist list + albums -->
      <div class="artist-two-column-layout">
        <div class="artist-column">
          <VirtualizedFavoritesArtistList
            groups={groupArtists(filteredArtists)}
            showGroupHeaders={true}
            selectedArtistId={selectedArtist?.id ?? null}
            onArtistSelect={handleArtistSelect}
          />
        </div>

        <div class="artist-albums-column">
          {#if !selectedArtist}
            <div class="artist-albums-empty">
              <Mic2 size={48} />
              <p>{$t('favorites.selectArtistHint')}</p>
            </div>
          {:else if loadingAlbums}
            <div class="artist-albums-loading">
              <Loader2 size={32} class="spinner-icon" />
              <p>{$t('favorites.loadingAlbums')}</p>
            </div>
          {:else if albumsError}
            <div class="artist-albums-error">
              <p>{$t('favorites.failedLoadAlbums')}</p>
              <p class="error-detail">{albumsError}</p>
            </div>
          {:else if totalDisplayedAlbums === 0}
            <div class="artist-albums-empty">
              <Disc3 size={48} />
              <p>{$t('artist.noAlbumsFound')}</p>
            </div>
          {:else}
            <div class="artist-albums-scroll">
              {#if groupedAlbums.discography.length > 0}
                <div class="artist-albums-section">
                  <div class="artist-albums-section-header">
                    <span class="section-title">{$t('artist.discography')}</span>
                    <span class="section-count">{groupedAlbums.discography.length}</span>
                  </div>
                  <div class="artist-albums-grid">
                    {#each groupedAlbums.discography as album (album.id)}
                      <AlbumCard
                        albumId={album.id}
                        artwork={album.image?.large || album.image?.small || ''}
                        title={album.title}
                        artist={album.artist?.name || ''}
                        genre={album.genre?.name}
                        releaseDate={album.release_date_original}
                        onclick={() => onAlbumClick?.(album.id)}
                        onPlay={() => onAlbumPlay?.(album.id)}
                      />
                    {/each}
                  </div>
                </div>
              {/if}
              {#if groupedAlbums.epsSingles.length > 0}
                <div class="artist-albums-section">
                  <div class="artist-albums-section-header">
                    <span class="section-title">{$t('artist.epsSingles')}</span>
                    <span class="section-count">{groupedAlbums.epsSingles.length}</span>
                  </div>
                  <div class="artist-albums-grid">
                    {#each groupedAlbums.epsSingles as album (album.id)}
                      <AlbumCard
                        albumId={album.id}
                        artwork={album.image?.large || album.image?.small || ''}
                        title={album.title}
                        artist={album.artist?.name || ''}
                        genre={album.genre?.name}
                        releaseDate={album.release_date_original}
                        onclick={() => onAlbumClick?.(album.id)}
                        onPlay={() => onAlbumPlay?.(album.id)}
                      />
                    {/each}
                  </div>
                </div>
              {/if}
              {#if groupedAlbums.liveAlbums.length > 0}
                <div class="artist-albums-section">
                  <div class="artist-albums-section-header">
                    <span class="section-title">{$t('artist.liveAlbums')}</span>
                    <span class="section-count">{groupedAlbums.liveAlbums.length}</span>
                  </div>
                  <div class="artist-albums-grid">
                    {#each groupedAlbums.liveAlbums as album (album.id)}
                      <AlbumCard
                        albumId={album.id}
                        artwork={album.image?.large || album.image?.small || ''}
                        title={album.title}
                        artist={album.artist?.name || ''}
                        genre={album.genre?.name}
                        releaseDate={album.release_date_original}
                        onclick={() => onAlbumClick?.(album.id)}
                        onPlay={() => onAlbumPlay?.(album.id)}
                      />
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .scene-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Top bar - matches FavoritesView */
  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--text-muted);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: color 150ms ease;
  }

  .back-btn:hover {
    color: var(--text-secondary);
  }

  /* Header with flag */
  .header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 4px;
    min-height: 40px;
  }

  .flag-icon {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    flex-shrink: 0;
    object-fit: cover;
  }

  .header h1 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .results-count {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .header-search {
    margin-left: auto;
    flex-shrink: 0;
  }

  .search-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: none;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .search-icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .search-expanded {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-tertiary);
    border-radius: 8px;
    padding: 6px 12px;
    width: 240px;
  }

  :global(.search-expanded .search-icon-inline) {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-input-inline {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 13px;
  }

  .search-input-inline::placeholder {
    color: var(--text-muted);
  }

  .search-clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }

  .search-clear-btn:hover {
    color: var(--text-primary);
  }

  /* Subtitle */
  .scene-subtitle {
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.4;
    margin-bottom: 12px;
  }

  /* Controls bar */
  .controls-bar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-bottom: 12px;
    flex-shrink: 0;
  }

  .nav-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .control-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 12px;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .control-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .control-btn.active {
    background: var(--accent-primary);
    color: var(--text-on-accent, #fff);
    border-color: var(--accent-primary);
  }

  .control-btn.icon-only {
    width: 36px;
    height: 36px;
    padding: 0;
    justify-content: center;
  }

  /* Content area */
  .scene-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Grid container */
  .scene-grid-container {
    flex: 1;
    min-height: 0;
  }

  /* Two-column sidepanel layout */
  .artist-two-column-layout {
    display: flex;
    gap: 0;
    flex: 1;
    min-height: 0;
    margin: 0 -8px 0 -18px;
    padding: 0;
  }

  .artist-column {
    width: 240px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: transparent;
    border-right: 1px solid var(--bg-tertiary);
    overflow: hidden;
    padding-left: 18px;
  }

  .artist-albums-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
    padding: 0 8px 0 24px;
  }

  .artist-albums-scroll {
    flex: 1;
    overflow-y: auto;
    padding-right: 8px;
    -webkit-overflow-scrolling: touch;
    overscroll-behavior: contain;
    contain: strict;
  }

  .artist-albums-section {
    margin-bottom: 32px;
  }

  .artist-albums-section:last-of-type {
    margin-bottom: 16px;
  }

  .artist-albums-section-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--bg-tertiary);
    margin-bottom: 16px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .section-count {
    font-size: 12px;
    color: var(--text-muted);
  }

  .artist-albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
    align-content: start;
  }

  .artist-albums-empty,
  .artist-albums-loading,
  .artist-albums-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
    color: var(--text-muted);
    font-size: 14px;
  }

  .error-detail {
    font-size: 12px;
    opacity: 0.7;
  }

  :global(.artist-albums-loading .spinner-icon) {
    animation: spin 1s linear infinite;
  }

  /* Loading state */
  .scene-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    padding: 100px 0;
  }

  .loading-visual {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .loading-pulse {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: var(--bg-secondary);
    color: var(--accent-primary);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.8; }
    50% { transform: scale(1.08); opacity: 1; }
  }

  .loading-status {
    min-height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .loading-text {
    font-size: 14px;
    color: var(--text-secondary);
    text-align: center;
    line-height: 1.4;
  }

  .fade-in {
    animation: fadeIn 400ms ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .loading-dots {
    display: flex;
    gap: 6px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-muted);
    animation: dotBounce 1.4s ease-in-out infinite;
  }

  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }

  @keyframes dotBounce {
    0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
    40% { opacity: 1; transform: scale(1); }
  }

  /* Error and empty states */
  .scene-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 80px 0;
    color: var(--text-muted);
    font-size: 14px;
  }

  .retry-button {
    padding: 8px 20px;
    border-radius: 8px;
    border: 1px solid var(--border-primary);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
    transition: background-color 150ms ease;
  }

  .retry-button:hover {
    background: var(--bg-tertiary);
  }

  .scene-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 80px 0;
    color: var(--text-muted);
    font-size: 14px;
  }

  /* Load more */
  .load-more-container {
    display: flex;
    justify-content: center;
    padding: 16px 0 8px;
    flex-shrink: 0;
  }

  .load-more-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 24px;
    border-radius: 8px;
    border: 1px solid var(--border-primary);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
    transition: background-color 150ms ease;
  }

  .load-more-button:hover:not(:disabled) {
    background: var(--bg-tertiary);
  }

  .load-more-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.load-more-button .spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
