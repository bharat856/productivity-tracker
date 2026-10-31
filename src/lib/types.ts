export type Category = 'productive' | 'neutral' | 'distracting';
export type TrackerEvent = { source: 'app' | 'website'; key: string; label: string; domain?: string; startedAt: number; endedAt: number; idle?: boolean };
export type UsageRow = { key: string; label: string; seconds: number; category: Category; domain?: string };
export type Dashboard = { totalSeconds: number; productiveSeconds: number; distractingSeconds: number; neutralSeconds: number; apps: UsageRow[]; browsers: UsageRow[]; websites: UsageRow[]; daily: { day: string; seconds: number }[]; focusSessions: number };
