import { describe, expect, it } from 'vitest';
import { aggregateIntervals } from './intervals';

describe('aggregateIntervals', () => {
  it('merges overlapping intervals without double counting', () => {
    expect(aggregateIntervals([{ source: 'app', key: 'Code', start: 0, end: 100 }, { source: 'app', key: 'Code', start: 50, end: 160 }], 0, 200).get('app:Code')).toBe(160);
  });
  it('clips intervals and keeps website accounting separate', () => {
    const result = aggregateIntervals([{ source: 'app', key: 'Brave Browser', start: 0, end: 100 }, { source: 'website', key: 'github.com', start: 20, end: 80 }], 30, 60);
    expect(result.get('app:Brave Browser')).toBe(30);
    expect(result.get('website:github.com')).toBe(30);
  });
});
