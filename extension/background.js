const NATIVE_HOST = 'com.daymark.productivity';
let port = null;
let active = null;

function connect() {
  try { port = chrome.runtime.connectNative(NATIVE_HOST); port.onDisconnect.addListener(() => { port = null; }); }
  catch (_) { port = null; }
}
function send(event) { if (!port) connect(); if (port) { try { port.postMessage(event); } catch (_) { port = null; } } }
function endActive(at = Date.now()) { if (active && at > active.startedAt) send({ type: 'website_interval', domain: active.domain, url: active.url, title: active.title, startedAt: active.startedAt, endedAt: at }); active = null; }
function begin(tab) { if (!tab || !tab.url || !/^https?:/.test(tab.url)) return endActive(); let url = new URL(tab.url); active = { domain: url.hostname.replace(/^www\./, ''), url: tab.url, title: tab.title || url.hostname, startedAt: Date.now() }; }
async function sync() { const tabs = await chrome.tabs.query({ active: true, lastFocusedWindow: true }); if (tabs[0]) begin(tabs[0]); }
chrome.tabs.onActivated.addListener(async () => { endActive(); await sync(); });
chrome.tabs.onUpdated.addListener((_id, change, tab) => { if (tab.active && (change.url || change.status === 'complete')) { endActive(); begin(tab); } });
chrome.windows.onFocusChanged.addListener(async id => { endActive(); if (id !== chrome.windows.WINDOW_ID_NONE) await sync(); });
chrome.idle.onStateChanged.addListener(state => { if (state === 'active') sync(); else endActive(); });
chrome.runtime.onSuspend.addListener(() => endActive());
chrome.runtime.onStartup.addListener(sync);
connect(); sync();
