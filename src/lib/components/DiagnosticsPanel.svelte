<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writeText as copyToClipboard } from '@tauri-apps/plugin-clipboard-manager';
  import { t } from '$lib/i18n';
  import { RefreshCw, Copy, Check, ChevronDown, ChevronRight } from 'lucide-svelte';

  interface RuntimeDiagnostics {
    audioOutputDevice: string | null;
    audioBackendType: string | null;
    audioExclusiveMode: boolean;
    audioDacPassthrough: boolean;
    audioPreferredSampleRate: number | null;
    audioAlsaPlugin: string | null;
    audioAlsaHardwareVolume: boolean;
    audioNormalizationEnabled: boolean;
    audioNormalizationTargetLufs: number;
    audioGaplessEnabled: boolean;
    audioPwForceBitperfect: boolean;
    audioStreamBufferSeconds: number;
    audioStreamingOnly: boolean;
    gfxHardwareAcceleration: boolean;
    gfxForceX11: boolean;
    gfxGdkScale: string | null;
    gfxGdkDpiScale: string | null;
    gfxGskRenderer: string | null;
    runtimeUsingFallback: boolean;
    runtimeIsWayland: boolean;
    runtimeHasNvidia: boolean;
    runtimeHasAmd: boolean;
    runtimeHasIntel: boolean;
    runtimeIsVm: boolean;
    runtimeHwAccelEnabled: boolean;
    runtimeForceX11Active: boolean;
    devForceDmabuf: boolean;
    envWebkitDisableDmabuf: string | null;
    envWebkitDisableCompositing: string | null;
    envGdkBackend: string | null;
    envGskRenderer: string | null;
    envLibglAlwaysSoftware: string | null;
    envWaylandDisplay: string | null;
    envXdgSessionType: string | null;
    appVersion: string;
  }

  interface DiagRow {
    label: string;
    saved: string;
    runtime: string;
    status: 'match' | 'mismatch' | 'info';
  }

  let diagnostics = $state<RuntimeDiagnostics | null>(null);
  let loading = $state(false);
  let copied = $state(false);
  let error = $state<string | null>(null);
  let panelOpen = $state(false);

  let audioOpen = $state(true);
  let graphicsOpen = $state(true);
  let envOpen = $state(false);

  function bool(val: boolean): string {
    return val ? 'ON' : 'OFF';
  }

  function str(val: string | null | undefined): string {
    return val ?? '—';
  }

  function matchStatus(saved: string, runtime: string): 'match' | 'mismatch' | 'info' {
    if (saved === '—' || runtime === '—') return 'info';
    return saved === runtime ? 'match' : 'mismatch';
  }

  function getAudioRows(diag: RuntimeDiagnostics): DiagRow[] {
    return [
      { label: 'Output Device', saved: str(diag.audioOutputDevice), runtime: '—', status: 'info' },
      { label: 'Backend', saved: str(diag.audioBackendType), runtime: '—', status: 'info' },
      { label: 'Exclusive Mode', saved: bool(diag.audioExclusiveMode), runtime: '—', status: 'info' },
      { label: 'DAC Passthrough', saved: bool(diag.audioDacPassthrough), runtime: '—', status: 'info' },
      { label: 'Preferred Sample Rate', saved: diag.audioPreferredSampleRate ? `${diag.audioPreferredSampleRate} Hz` : 'Auto', runtime: '—', status: 'info' },
      { label: 'ALSA Plugin', saved: str(diag.audioAlsaPlugin), runtime: '—', status: 'info' },
      { label: 'ALSA HW Volume', saved: bool(diag.audioAlsaHardwareVolume), runtime: '—', status: 'info' },
      { label: 'Normalization', saved: bool(diag.audioNormalizationEnabled), runtime: '—', status: 'info' },
      { label: 'Normalization Target', saved: `${diag.audioNormalizationTargetLufs} LUFS`, runtime: '—', status: 'info' },
      { label: 'Gapless', saved: bool(diag.audioGaplessEnabled), runtime: '—', status: 'info' },
      { label: 'PW Force Bitperfect', saved: bool(diag.audioPwForceBitperfect), runtime: '—', status: 'info' },
      { label: 'Stream Buffer', saved: `${diag.audioStreamBufferSeconds}s`, runtime: '—', status: 'info' },
      { label: 'Streaming Only', saved: bool(diag.audioStreamingOnly), runtime: '—', status: 'info' },
    ];
  }

  function getGraphicsRows(diag: RuntimeDiagnostics): DiagRow[] {
    const hwSaved = bool(diag.gfxHardwareAcceleration);
    const hwRuntime = bool(diag.runtimeHwAccelEnabled);
    const x11Saved = bool(diag.gfxForceX11);
    const x11Runtime = bool(diag.runtimeForceX11Active);
    const compositing = diag.envWebkitDisableCompositing === '1' ? 'DISABLED' : 'ENABLED';
    const dmabuf = diag.envWebkitDisableDmabuf === '1' ? 'DISABLED' : 'ENABLED';

    return [
      { label: 'Hardware Acceleration', saved: hwSaved, runtime: hwRuntime, status: matchStatus(hwSaved, hwRuntime) },
      { label: 'Force DMA-BUF', saved: bool(diag.devForceDmabuf), runtime: dmabuf, status: diag.devForceDmabuf === (dmabuf === 'ENABLED') ? 'match' : 'mismatch' },
      { label: 'Force X11', saved: x11Saved, runtime: x11Runtime, status: matchStatus(x11Saved, x11Runtime) },
      { label: 'GSK Renderer', saved: str(diag.gfxGskRenderer), runtime: str(diag.envGskRenderer), status: matchStatus(str(diag.gfxGskRenderer), str(diag.envGskRenderer)) },
      { label: 'GDK Scale', saved: str(diag.gfxGdkScale), runtime: '—', status: 'info' },
      { label: 'GDK DPI Scale', saved: str(diag.gfxGdkDpiScale), runtime: '—', status: 'info' },
      { label: 'Compositing Mode', saved: '—', runtime: compositing, status: 'info' },
      { label: 'GPU: NVIDIA', saved: '—', runtime: diag.runtimeHasNvidia ? 'Detected' : 'No', status: 'info' },
      { label: 'GPU: Intel', saved: '—', runtime: diag.runtimeHasIntel ? 'Detected' : 'No', status: 'info' },
      { label: 'GPU: AMD', saved: '—', runtime: diag.runtimeHasAmd ? 'Detected' : 'No', status: 'info' },
      { label: 'Wayland', saved: '—', runtime: diag.runtimeIsWayland ? 'Yes' : 'No (X11)', status: 'info' },
      { label: 'VM', saved: '—', runtime: diag.runtimeIsVm ? 'Yes' : 'No', status: 'info' },
      { label: 'Using Fallback', saved: '—', runtime: bool(diag.runtimeUsingFallback), status: diag.runtimeUsingFallback ? 'mismatch' : 'info' },
    ];
  }

  function getEnvRows(diag: RuntimeDiagnostics): DiagRow[] {
    return [
      { label: 'WEBKIT_DISABLE_DMABUF_RENDERER', saved: '—', runtime: str(diag.envWebkitDisableDmabuf), status: 'info' },
      { label: 'WEBKIT_DISABLE_COMPOSITING_MODE', saved: '—', runtime: str(diag.envWebkitDisableCompositing), status: 'info' },
      { label: 'GDK_BACKEND', saved: '—', runtime: str(diag.envGdkBackend), status: 'info' },
      { label: 'GSK_RENDERER', saved: '—', runtime: str(diag.envGskRenderer), status: 'info' },
      { label: 'LIBGL_ALWAYS_SOFTWARE', saved: '—', runtime: str(diag.envLibglAlwaysSoftware), status: 'info' },
      { label: 'WAYLAND_DISPLAY', saved: '—', runtime: str(diag.envWaylandDisplay), status: 'info' },
      { label: 'XDG_SESSION_TYPE', saved: '—', runtime: str(diag.envXdgSessionType), status: 'info' },
    ];
  }

  async function loadDiagnostics() {
    loading = true;
    error = null;
    try {
      diagnostics = await invoke<RuntimeDiagnostics>('v2_get_runtime_diagnostics');
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function exportToClipboard() {
    if (!diagnostics) return;
    try {
      const exportData = {
        ...diagnostics,
        exportedAt: new Date().toISOString(),
      };
      await copyToClipboard(JSON.stringify(exportData, null, 2));
      copied = true;
      setTimeout(() => { copied = false; }, 1500);
    } catch {
      try {
        await navigator.clipboard.writeText(JSON.stringify(diagnostics, null, 2));
        copied = true;
        setTimeout(() => { copied = false; }, 1500);
      } catch { /* ignore */ }
    }
  }

  function togglePanel() {
    panelOpen = !panelOpen;
    if (panelOpen && !diagnostics) {
      loadDiagnostics();
    }
  }
</script>

<div class="diagnostics-panel">
  <button class="section-toggle panel-toggle" onclick={togglePanel}>
    {#if panelOpen}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
    <h4 class="subsection-title" style="margin:0">{$t('settings.developer.diagnostics.title')}</h4>
    <small class="setting-note" style="margin:0">{$t('settings.developer.diagnostics.description')}</small>
  </button>

  {#if panelOpen}
  <div class="diag-body">
  <div class="diag-actions">
    <button class="diag-btn" onclick={loadDiagnostics} disabled={loading}>
      <RefreshCw size={14} class={loading ? 'spinning' : ''} />
      {$t('settings.developer.diagnostics.refresh')}
    </button>
    <button class="diag-btn" onclick={exportToClipboard} disabled={!diagnostics}>
      {#if copied}
        <Check size={14} />
        {$t('settings.developer.diagnostics.exported')}
      {:else}
        <Copy size={14} />
        {$t('settings.developer.diagnostics.export')}
      {/if}
    </button>
  </div>

  {#if error}
    <div class="diag-error">{error}</div>
  {/if}

  {#if diagnostics}
    <div class="diag-version">QBZ v{diagnostics.appVersion}</div>

    <!-- Audio Section -->
    <button class="section-toggle" onclick={() => audioOpen = !audioOpen}>
      {#if audioOpen}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
      {$t('settings.developer.diagnostics.sectionAudio')}
    </button>
    {#if audioOpen}
      <table class="diag-table">
        <thead>
          <tr>
            <th>{$t('settings.developer.diagnostics.colSetting')}</th>
            <th>{$t('settings.developer.diagnostics.colSaved')}</th>
            <th>{$t('settings.developer.diagnostics.colRuntime')}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each getAudioRows(diagnostics) as row (row.label)}
            <tr>
              <td class="label-cell">{row.label}</td>
              <td class="value-cell">{row.saved}</td>
              <td class="value-cell">{row.runtime}</td>
              <td class="status-cell {row.status}">{row.status === 'match' ? '✓' : row.status === 'mismatch' ? '✗' : '·'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <!-- Graphics Section -->
    <button class="section-toggle" onclick={() => graphicsOpen = !graphicsOpen}>
      {#if graphicsOpen}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
      {$t('settings.developer.diagnostics.sectionGraphics')}
    </button>
    {#if graphicsOpen}
      <table class="diag-table">
        <thead>
          <tr>
            <th>{$t('settings.developer.diagnostics.colSetting')}</th>
            <th>{$t('settings.developer.diagnostics.colSaved')}</th>
            <th>{$t('settings.developer.diagnostics.colRuntime')}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each getGraphicsRows(diagnostics) as row (row.label)}
            <tr>
              <td class="label-cell">{row.label}</td>
              <td class="value-cell">{row.saved}</td>
              <td class="value-cell">{row.runtime}</td>
              <td class="status-cell {row.status}">{row.status === 'match' ? '✓' : row.status === 'mismatch' ? '✗' : '·'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <!-- Environment Section -->
    <button class="section-toggle" onclick={() => envOpen = !envOpen}>
      {#if envOpen}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
      {$t('settings.developer.diagnostics.sectionEnv')}
    </button>
    {#if envOpen}
      <table class="diag-table">
        <thead>
          <tr>
            <th>{$t('settings.developer.diagnostics.colSetting')}</th>
            <th colspan="2">{$t('settings.developer.diagnostics.colRuntime')}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each getEnvRows(diagnostics) as row (row.label)}
            <tr>
              <td class="label-cell mono">{row.label}</td>
              <td class="value-cell mono" colspan="2">{row.runtime}</td>
              <td class="status-cell info">·</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
  </div>
  {/if}
</div>

<style>
  .diagnostics-panel {
    margin-top: 16px;
  }

  .diag-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 12px;
  }

  .diag-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .diag-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    font-size: 12px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--alpha-8);
    border-radius: 6px;
    cursor: pointer;
    transition: background 150ms ease;
  }

  .diag-btn:hover {
    background: var(--bg-hover);
  }

  .diag-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .diag-error {
    padding: 8px 12px;
    background: rgba(220, 50, 50, 0.15);
    color: var(--text-error, #f44);
    border-radius: 6px;
    font-size: 12px;
    margin-bottom: 12px;
  }

  .diag-version {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .section-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .section-toggle:hover {
    color: var(--accent-primary);
  }

  .diag-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    margin-bottom: 16px;
  }

  .diag-table th {
    text-align: left;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    border-bottom: 1px solid var(--alpha-8);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .diag-table td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--alpha-4);
  }

  .label-cell {
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .value-cell {
    color: var(--text-primary);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 11px;
  }

  .mono {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 11px;
  }

  .status-cell {
    width: 24px;
    text-align: center;
    font-weight: 700;
  }

  .status-cell.match {
    color: #4caf50;
  }

  .status-cell.mismatch {
    color: #f44336;
  }

  .status-cell.info {
    color: var(--text-muted);
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
