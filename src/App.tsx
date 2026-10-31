import { useEffect, useMemo, useState } from 'react';
import { Activity, BarChart3, ChevronDown, CircleHelp, Download, LayoutDashboard, Pause, Play, Settings2, ShieldCheck, Sparkles, Target, Trash2 } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { deleteData, exportData, getDashboard, hidePet, setCategory, setPaused, showPet } from './lib/bridge';
import type { Category, Dashboard, UsageRow } from './lib/types';
import { getCurrentWindow } from '@tauri-apps/api/window';

const fmt = (seconds: number) => `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
const pct = (n: number, total: number) => total ? Math.round(n / total * 100) : 0;

function Donut({ data }: { data: Dashboard }) { const p = pct(data.productiveSeconds, data.totalSeconds); const d = pct(data.distractingSeconds, data.totalSeconds); return <div className="donut-wrap"><div className="donut" style={{ background: `conic-gradient(#b5e36b 0 ${p}%, #f1a75b ${p}% ${p + d}%, #52627a ${p + d}% 100%)` }}><div className="donut-hole"><strong>{fmt(data.totalSeconds)}</strong><span>tracked today</span></div></div></div>; }

type PetMood = 'idle' | 'happy' | 'focused' | 'sleepy' | 'distracted';
function Pet() {
  const [mood, setMood] = useState<PetMood>('idle');
  const [message, setMessage] = useState('Ready when you are.');
  useEffect(() => {
    const refresh = async () => {
      const current = await getDashboard();
      const hour = new Date().getHours();
      if (hour >= 22 || hour < 7) { setMood('sleepy'); setMessage('A gentle landing soon?'); }
      else if (current.totalSeconds === 0) { setMood('idle'); setMessage('Ready when you are.'); }
      else if (current.productiveSeconds >= current.distractingSeconds * 1.5) { setMood('focused'); setMessage('You’re in a good rhythm.'); }
      else if (current.distractingSeconds > current.productiveSeconds) { setMood('distracted'); setMessage('A small reset might help.'); }
      else { setMood('happy'); setMessage('Nice rhythm ✦'); }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 15000);
    return () => window.clearInterval(timer);
  }, []);
  return <div className={`pet-window pet-${mood}`} data-tauri-drag-region title="Drag to reposition Daymark"><div className="pet-bubble">{message}</div><button className="pet-hide" onClick={() => void hidePet()} aria-label="Hide pet">×</button><div className="pet-face" data-tauri-drag-region><span className="pet-eyes">••</span><span className="pet-mouth">{mood === 'sleepy' ? '﹀' : mood === 'distracted' ? 'o' : '⌣'}</span></div><span className="pet-status">{mood}</span></div>;
}

function Row({ item, websites, onCategory }: { item: UsageRow; websites?: UsageRow[]; onCategory: (item: UsageRow, category: Category) => void }) { const [open, setOpen] = useState(false); return <div className="usage-row"><div className="row-main" onClick={() => websites && setOpen(!open)}><span className="app-mark">{item.label.slice(0, 1)}</span><span className="row-label">{item.label}<small>{item.category}</small></span><span className="row-time">{fmt(item.seconds)}</span>{websites && <ChevronDown size={16} className={open ? 'rotate' : ''} />}</div>{open && <div className="nested">{websites?.map(site => <div className="nested-row" key={site.key}><span>{site.label}</span><span>{fmt(site.seconds)}</span><select value={site.category} onChange={e => onCategory(site, e.target.value as Category)} aria-label={`Classify ${site.label}`}><option value="productive">Productive</option><option value="neutral">Neutral</option><option value="distracting">Distracting</option></select></div>)}</div>}<div className="category-control"><select value={item.category} onChange={e => onCategory(item, e.target.value as Category)} aria-label={`Classify ${item.label}`}><option value="productive">Productive</option><option value="neutral">Neutral</option><option value="distracting">Distracting</option></select></div></div>; }

export default function App() {
  const [data, setData] = useState<Dashboard>();
  const [paused, setPausedState] = useState(false);
  const [tab, setTab] = useState('Overview');
  const [search, setSearch] = useState('');
  const [isPet, setIsPet] = useState(false);
  const navItems: [string, LucideIcon][] = [['Overview', LayoutDashboard], ['Activity', Activity], ['Focus sessions', Target], ['Insights', BarChart3]];
  useEffect(() => { void getDashboard().then(setData); if ('__TAURI_INTERNALS__' in window) setIsPet(getCurrentWindow().label === 'pet'); const timer = window.setInterval(() => void getDashboard().then(setData), 5000); return () => window.clearInterval(timer); }, []);
  const rows = useMemo(() => data?.apps.filter(x => x.label.toLowerCase().includes(search.toLowerCase())) ?? [], [data, search]);
  const classify = async (item: UsageRow, category: Category) => { await setCategory(item.key, category, item.domain ? 'website' : 'app'); setData(await getDashboard()); };
  const downloadExport = async () => { const blob = new Blob([await exportData()], { type: 'application/json' }); const url = URL.createObjectURL(blob); const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'daymark-export.json'; anchor.click(); URL.revokeObjectURL(url); };
  const clearData = async () => { if (window.confirm('Delete all locally stored Daymark data?')) { await deleteData(); setData(await getDashboard()); } };
  // Floating pet temporarily disabled while the visual design is being refined.
  // if (isPet) return <Pet />;
  if (isPet) return null;
  if (!data) return <div className="loading">Loading your workspace…</div>;
  const max = Math.max(...data.daily.map(x => x.seconds), 1);
 return <div className="shell"><aside><div className="brand"><span className="brand-orb">✦</span><span>Daymark</span></div><div className="nav-label">WORKSPACE</div>{navItems.map(([label, Icon]) => <button key={label} className={tab === label ? 'nav active' : 'nav'} onClick={() => setTab(label)}><Icon size={17}/>{label}</button>)}<div className="nav-label space">MANAGE</div><button className="nav" onClick={() => setTab('Rules')}><Settings2 size={17}/>Productivity rules</button><button className="nav" onClick={() => setTab('Privacy')}><ShieldCheck size={17}/>Privacy & data</button><div className="sidebar-bottom"><div className="mini-pet">◒</div><div><strong>Pet paused</strong><span>Visual refresh in progress</span></div>{/* Pet launcher temporarily disabled. <button className="icon-btn" onClick={() => void showPet()} title="Show pet"><Sparkles size={16}/></button> */}</div></aside><main><header><div><p className="eyebrow">FRIDAY, OCTOBER 31, 2026</p><h1>{tab === 'Overview' ? 'Good morning, Bharat.' : tab}</h1><p className="muted">A clear view of where your attention went today.</p></div><div className="header-actions"><button className={paused ? 'pause-btn paused' : 'pause-btn'} onClick={() => { setPausedState(!paused); void setPaused(!paused); }}>{paused ? <Play size={15}/> : <Pause size={15}/>} {paused ? 'Resume tracking' : 'Pause tracking'}</button><button className="avatar">BP</button></div></header>{tab === 'Overview' ? <><section className="hero-grid"><div className="card summary-card"><div className="card-head"><div><span className="label">TODAY’S ATTENTION</span><h2>{fmt(data.totalSeconds)}</h2></div><Donut data={data}/></div><div className="legend"><span><i className="green"/>Productive <b>{fmt(data.productiveSeconds)}</b></span><span><i className="orange"/>Distracting <b>{fmt(data.distractingSeconds)}</b></span><span><i className="blue"/>Neutral <b>{fmt(data.neutralSeconds)}</b></span></div></div><div className="card focus-card"><span className="label">FOCUS SESSIONS</span><div className="focus-number">{data.focusSessions}<span> sessions</span></div><p className="muted">You’re building a steady rhythm. Keep it up.</p><div className="focus-line"><span style={{ width: '68%' }}/></div><small>2h 18m focused today</small></div></section><section className="section-head"><div><span className="label">ACTIVITY PULSE</span><h2>This week</h2></div><button className="ghost-btn" onClick={() => void downloadExport()}><Download size={15}/>Export</button></section><section className="card chart-card"><div className="chart">{data.daily.map(item => <div className="bar-col" key={item.day}><div className="bar-track"><div className="bar" style={{ height: `${item.seconds / max * 100}%` }}/></div><span>{item.day}</span></div>)}</div></section><section className="section-head usage-head"><div><span className="label">WHERE TIME WENT</span><h2>Applications & websites</h2></div><input placeholder="Filter activity" value={search} onChange={e => setSearch(e.target.value)}/></section><section className="card usage-card"><div className="usage-title"><span>APPLICATION</span><span>TIME TODAY</span><span>CLASSIFY</span></div>{rows.map(item => <Row key={item.key} item={item} websites={item.key === 'Brave Browser' ? data.websites : undefined} onCategory={classify}/>)}</section></> : tab === 'Privacy' ? <div className="card empty-state"><ShieldCheck size={28}/><h2>Your data stays on this Mac</h2><p>Daymark stores only application/domain names, timestamps, and classifications. It never stores page content, keystrokes, passwords, or form data.</p><div className="privacy-actions"><button className="ghost-btn" onClick={() => void downloadExport()}><Download size={15}/>Export JSON</button><button className="danger-btn" onClick={() => void clearData()}><Trash2 size={15}/>Delete all data</button></div></div> : <div className="card empty-state"><CircleHelp size={28}/><h2>{tab} is ready to configure</h2><p>This MVP keeps the core tracker local-first. Use the dashboard to review activity, classify items, and control privacy.</p></div>}</main></div>;
}
