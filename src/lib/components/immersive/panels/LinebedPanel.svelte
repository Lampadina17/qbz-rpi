<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import QualityBadge from '$lib/components/QualityBadge.svelte';
  import { getPanelFrameInterval } from '$lib/immersive/fpsConfig';

  interface Props {
    enabled?: boolean;
    artwork?: string;
    trackTitle?: string;
    artist?: string;
    album?: string;
    explicit?: boolean;
    quality?: string;
    bitDepth?: number;
    samplingRate?: number;
    originalBitDepth?: number;
    originalSamplingRate?: number;
    format?: string;
  }

  let {
    enabled = true,
    artwork = '',
    trackTitle = '',
    artist = '',
    album = '',
    explicit = false,
    quality,
    bitDepth,
    samplingRate,
    originalBitDepth,
    originalSamplingRate,
    format
  }: Props = $props();

  let canvasRef: HTMLCanvasElement | null = $state(null);
  let ctx: CanvasRenderingContext2D | null = null;
  let animationFrame: number | null = null;
  let unlisten: UnlistenFn | null = null;
  let isInitialized = false;

  // Linebed parameters
  const NUM_BANDS = 190; // viz:spectral sends 190 bands
  const NUM_LINES = 35;  // Number of stacked history lines
  const SMOOTHING = 0.5;

  // Ring buffer of spectrum snapshots
  const history: Float32Array[] = [];
  for (let i = 0; i < NUM_LINES; i++) {
    history.push(new Float32Array(NUM_BANDS));
  }
  let historyIndex = 0;
  let frameCounter = 0;

  // Current smoothed spectrum
  const smoothedData = new Float32Array(NUM_BANDS);

  let lastRenderTime = 0;
  const FRAME_INTERVAL = getPanelFrameInterval('linebed');

  // Colors from artwork
  let lineColor = $state({ r: 255, g: 255, b: 255 });

  function extractColors(imgSrc: string) {
    if (!imgSrc) return;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      const sampleCanvas = document.createElement('canvas');
      const size = 10;
      sampleCanvas.width = size;
      sampleCanvas.height = size;
      const sampleCtx = sampleCanvas.getContext('2d');
      if (!sampleCtx) return;

      sampleCtx.drawImage(img, 0, 0, size, size);
      const data = sampleCtx.getImageData(0, 0, size, size).data;

      const colors: { r: number; g: number; b: number; lum: number }[] = [];
      for (let i = 0; i < data.length; i += 4) {
        const r = data[i], g = data[i + 1], b = data[i + 2];
        const lum = (r + g + b) / 3;
        if (lum > 100 && lum < 240) {
          colors.push({ r, g, b, lum });
        }
      }

      if (colors.length > 0) {
        // Pick the brightest suitable color
        colors.sort((a, b) => b.lum - a.lum);
        lineColor = { r: colors[0].r, g: colors[0].g, b: colors[0].b };
      } else {
        lineColor = { r: 255, g: 255, b: 255 };
      }
    };
    img.src = imgSrc;
  }

  $effect(() => {
    if (artwork) {
      extractColors(artwork);
    }
  });

  async function init() {
    if (!canvasRef || isInitialized) return;

    ctx = canvasRef.getContext('2d');
    if (!ctx) return;

    isInitialized = true;

    try {
      await invoke('v2_set_visualizer_enabled', { enabled: true });
    } catch (e) {
      console.error('[Linebed] Failed to enable backend:', e);
    }

    unlisten = await listen<number[]>('viz:spectral', (event) => {
      const payload = event.payload;
      if (Array.isArray(payload)) {
        const bytes = new Uint8Array(payload);
        const floats = new Float32Array(bytes.buffer);
        if (floats.length === NUM_BANDS) {
          for (let i = 0; i < NUM_BANDS; i++) {
            smoothedData[i] = smoothedData[i] * SMOOTHING + floats[i] * (1 - SMOOTHING);
          }

          // Push snapshot into history every 2nd frame for slower scroll
          frameCounter++;
          if (frameCounter >= 2) {
            frameCounter = 0;
            history[historyIndex].set(smoothedData);
            historyIndex = (historyIndex + 1) % NUM_LINES;
          }
        }
      }
    });

    render(0);
  }

  function render(timestamp: number = 0) {
    if (!ctx || !canvasRef) return;

    const delta = timestamp - lastRenderTime;
    if (delta < FRAME_INTERVAL) {
      animationFrame = requestAnimationFrame(render);
      return;
    }
    lastRenderTime = timestamp;

    const rect = canvasRef.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const width = rect.width;
    const height = rect.height;

    const targetWidth = Math.floor(width * dpr);
    const targetHeight = Math.floor(height * dpr);
    if (canvasRef.width !== targetWidth || canvasRef.height !== targetHeight) {
      canvasRef.width = targetWidth;
      canvasRef.height = targetHeight;
      ctx.scale(dpr, dpr);
    }

    // Clear
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, width, height);

    // Linebed rendering area
    const bedLeft = width * 0.08;
    const bedRight = width * 0.92;
    const bedWidth = bedRight - bedLeft;
    const bedTop = height * 0.15;
    const bedBottom = height * 0.85;
    const bedHeight = bedBottom - bedTop;

    // Perspective parameters
    const vanishY = bedTop - bedHeight * 0.3; // Vanishing point above the bed
    const frontLineWidth = bedWidth;
    const backLineWidth = bedWidth * 0.35;

    // Subsample bands for smoother lines (use every ~2nd band)
    const step = 2;
    const numPoints = Math.floor(NUM_BANDS / step);

    // Draw lines from back (oldest) to front (newest) for correct occlusion
    for (let lineIdx = 0; lineIdx < NUM_LINES; lineIdx++) {
      // Map lineIdx: 0=oldest (back), NUM_LINES-1=newest (front)
      const bufIdx = (historyIndex + lineIdx) % NUM_LINES;
      const spectrum = history[bufIdx];

      // Interpolation factor: 0=back, 1=front
      const lineFactor = lineIdx / (NUM_LINES - 1);

      // Y position: back lines near top, front lines near bottom
      const baseY = bedTop + lineFactor * bedHeight;

      // Line width: narrower at back, wider at front (perspective)
      const currentLineWidth = backLineWidth + lineFactor * (frontLineWidth - backLineWidth);
      const lineLeft = (width - currentLineWidth) / 2;

      // Amplitude scale: taller at front
      const amplitudeScale = 0.15 + lineFactor * 0.85;
      const maxAmplitude = bedHeight * 0.28 * amplitudeScale;

      // Opacity: fainter at back
      const opacity = 0.08 + lineFactor * 0.92;

      // Build the line path
      ctx.beginPath();
      ctx.moveTo(lineLeft, baseY);

      for (let p = 0; p < numPoints; p++) {
        const bandIdx = p * step;
        const x = lineLeft + (p / (numPoints - 1)) * currentLineWidth;
        const amplitude = spectrum[bandIdx];
        const y = baseY - amplitude * maxAmplitude;
        if (p === 0) {
          ctx.lineTo(x, y);
        } else {
          // Smooth with quadratic curves
          const prevBandIdx = (p - 1) * step;
          const prevX = lineLeft + ((p - 1) / (numPoints - 1)) * currentLineWidth;
          const prevAmplitude = spectrum[prevBandIdx];
          const prevY = baseY - prevAmplitude * maxAmplitude;
          const cpX = (prevX + x) / 2;
          const cpY = (prevY + y) / 2;
          ctx.quadraticCurveTo(prevX, prevY, cpX, cpY);
        }
      }

      // Close the last point
      const lastBandIdx = (numPoints - 1) * step;
      const lastX = lineLeft + currentLineWidth;
      const lastY = baseY - spectrum[lastBandIdx] * maxAmplitude;
      ctx.lineTo(lastX, lastY);

      // Close the shape below for occlusion fill
      ctx.lineTo(lineLeft + currentLineWidth, baseY + 2);
      ctx.lineTo(lineLeft, baseY + 2);
      ctx.closePath();

      // Fill with black for occlusion (hides lines behind)
      ctx.fillStyle = '#000000';
      ctx.fill();

      // Draw the line stroke on top
      // Rebuild just the top path for stroke
      ctx.beginPath();
      ctx.moveTo(lineLeft, baseY);

      for (let p = 0; p < numPoints; p++) {
        const bandIdx = p * step;
        const x = lineLeft + (p / (numPoints - 1)) * currentLineWidth;
        const amplitude = spectrum[bandIdx];
        const y = baseY - amplitude * maxAmplitude;
        if (p === 0) {
          ctx.lineTo(x, y);
        } else {
          const prevBandIdx = (p - 1) * step;
          const prevX = lineLeft + ((p - 1) / (numPoints - 1)) * currentLineWidth;
          const prevAmplitude = spectrum[prevBandIdx];
          const prevY = baseY - prevAmplitude * maxAmplitude;
          const cpX = (prevX + x) / 2;
          const cpY = (prevY + y) / 2;
          ctx.quadraticCurveTo(prevX, prevY, cpX, cpY);
        }
      }

      const lastY2 = baseY - spectrum[lastBandIdx] * maxAmplitude;
      ctx.lineTo(lastX, lastY2);

      // Stroke with line color and opacity
      const lineWeight = 0.5 + lineFactor * 1.0;
      ctx.strokeStyle = `rgba(${lineColor.r}, ${lineColor.g}, ${lineColor.b}, ${opacity})`;
      ctx.lineWidth = lineWeight;
      ctx.stroke();
    }

    animationFrame = requestAnimationFrame(render);
  }

  async function cleanup() {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
      animationFrame = null;
    }

    if (unlisten) {
      unlisten();
      unlisten = null;
    }

    try {
      await invoke('v2_set_visualizer_enabled', { enabled: false });
    } catch (e) {
      console.error('[Linebed] Failed to disable backend:', e);
    }

    isInitialized = false;
  }

  onMount(() => {
    if (enabled) {
      init();
    }
    return cleanup;
  });

  $effect(() => {
    if (enabled && !isInitialized) {
      init();
    } else if (!enabled && isInitialized) {
      cleanup();
    }
  });
</script>

<div class="linebed-panel" class:visible={enabled}>
  <canvas bind:this={canvasRef} class="linebed-canvas"></canvas>

  <div class="bottom-info">
    <div class="track-meta">
      <span class="track-title">{trackTitle}</span>
      {#if explicit}
        <span class="explicit-badge" title="Explicit"></span>
      {/if}
      {#if album}
        <span class="track-album">{album}</span>
      {/if}
      <span class="track-artist">{artist}</span>
      <QualityBadge {quality} {bitDepth} {samplingRate} {originalBitDepth} {originalSamplingRate} {format} compact />
    </div>
    {#if artwork}
      <div class="artwork-thumb">
        <img src={artwork} alt={trackTitle} />
      </div>
    {/if}
  </div>
</div>

<style>
  .linebed-panel {
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 300ms ease;
    z-index: 5;
    background: #000000;
  }

  .linebed-panel.visible {
    opacity: 1;
  }

  .linebed-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .bottom-info {
    position: absolute;
    right: 24px;
    bottom: 24px;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .track-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
  }

  .track-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary, white);
    text-shadow: 0 1px 6px rgba(0, 0, 0, 0.4);
    max-width: 400px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-album {
    font-size: 12px;
    color: var(--alpha-50, rgba(255, 255, 255, 0.5));
    font-style: italic;
    max-width: 400px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artist {
    font-size: 12px;
    color: var(--alpha-60, rgba(255, 255, 255, 0.6));
    max-width: 400px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artwork-thumb {
    width: 72px;
    height: 72px;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    flex-shrink: 0;
  }

  .artwork-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .explicit-badge {
    display: inline-block;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    opacity: 0.45;
    background-color: var(--text-primary, white);
    -webkit-mask: url('/explicit.svg') center / contain no-repeat;
    mask: url('/explicit.svg') center / contain no-repeat;
  }

  @media (max-width: 768px) {
    .bottom-info {
      right: 16px;
      bottom: 16px;
    }

    .artwork-thumb {
      width: 56px;
      height: 56px;
    }
  }
</style>
