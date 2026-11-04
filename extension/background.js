// ActLog Browser Extension Background Worker
let lastSentUrl = '';
let debounceTimer = null;

async function sendHeartbeat() {
  try {
    const [activeTab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
    if (!activeTab || !activeTab.url) return;

    // Ignore browser internal schemes
    if (activeTab.url.startsWith('chrome://') || activeTab.url.startsWith('edge://') || activeTab.url.startsWith('about:')) {
      return;
    }

    let domain = '';
    try {
      const parsed = new URL(activeTab.url);
      domain = parsed.hostname;
    } catch (_) {}

    const payload = {
      url: activeTab.url,
      domain: domain,
      title: activeTab.title || '',
      tab_active: true
    };

    lastSentUrl = activeTab.url;

    await fetch('http://127.0.0.1:5577/api/heartbeat/browser', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
  } catch (_) {
    // Daemon not running or sleeping - silently ignore
  }
}

function triggerHeartbeatDebounced() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(sendHeartbeat, 500);
}

// Tab switches
chrome.tabs.onActivated.addListener(() => {
  triggerHeartbeatDebounced();
});

// URL navigation / title changes
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (tab.active && (changeInfo.url || changeInfo.title)) {
    triggerHeartbeatDebounced();
  }
});

// Periodic heartbeat every 3 seconds for continuous active reading
setInterval(sendHeartbeat, 3000);

