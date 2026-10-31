export type TimedInterval = { source: string; key: string; start: number; end: number };

/** Merge overlaps within one accounting stream. Browser website spans are intentionally
 * kept separate from foreground-app spans; the dashboard uses app spans for total time. */
export function aggregateIntervals(intervals: TimedInterval[], from: number, to: number) {
  const grouped = new Map<string, [number, number][]>();
  for (const item of intervals) {
    const start = Math.max(from, item.start);
    const end = Math.min(to, item.end);
    if (end > start) grouped.set(`${item.source}:${item.key}`, [...(grouped.get(`${item.source}:${item.key}`) ?? []), [start, end]]);
  }
  const result = new Map<string, number>();
  for (const [key, spans] of grouped) {
    spans.sort((a, b) => a[0] - b[0]);
    let total = 0; let current = spans[0];
    for (const span of spans.slice(1)) { if (span[0] <= current[1]) current[1] = Math.max(current[1], span[1]); else { total += current[1] - current[0]; current = span; } }
    result.set(key, total + current[1] - current[0]);
  }
  return result;
}
