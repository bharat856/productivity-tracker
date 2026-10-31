import { invoke } from '@tauri-apps/api/core';
import type { Dashboard, Category } from './types';

const tauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
const demo: Dashboard = {
  totalSeconds: 12640, productiveSeconds: 8140, distractingSeconds: 2130, neutralSeconds: 2370,
  apps: [{ key: 'Code', label: 'Visual Studio Code', seconds: 4920, category: 'productive' }, { key: 'Brave Browser', label: 'Brave Browser', seconds: 3880, category: 'neutral' }, { key: 'Slack', label: 'Slack', seconds: 1560, category: 'neutral' }, { key: 'Figma', label: 'Figma', seconds: 1040, category: 'productive' }],
  browsers: [{ key: 'Brave Browser', label: 'Brave Browser', seconds: 3880, category: 'neutral' }],
  websites: [{ key: 'github.com', label: 'github.com', seconds: 1860, category: 'productive', domain: 'github.com' }, { key: 'chatgpt.com', label: 'chatgpt.com', seconds: 1140, category: 'neutral', domain: 'chatgpt.com' }, { key: 'youtube.com', label: 'youtube.com', seconds: 900, category: 'neutral', domain: 'youtube.com' }],
  daily: [{ day: 'Mon', seconds: 5200 }, { day: 'Tue', seconds: 6800 }, { day: 'Wed', seconds: 8140 }, { day: 'Thu', seconds: 7400 }, { day: 'Fri', seconds: 12640 }, { day: 'Sat', seconds: 0 }, { day: 'Sun', seconds: 0 }], focusSessions: 4
};

export async function getDashboard(): Promise<Dashboard> { return tauri ? invoke('get_dashboard') : demo; }
export async function setCategory(key: string, category: Category, source = 'app'): Promise<void> { if (tauri) await invoke('set_category', { key, category, source }); }
export async function setPaused(paused: boolean): Promise<void> { if (tauri) await invoke('set_tracking_paused', { paused }); }
export async function showPet(): Promise<void> { if (tauri) await invoke('show_pet'); }
export async function hidePet(): Promise<void> { if (tauri) await invoke('hide_pet'); }
export async function exportData(): Promise<string> { return tauri ? invoke('export_data') : JSON.stringify(demo, null, 2); }
export async function deleteData(): Promise<void> { if (tauri) await invoke('delete_all_data'); }
