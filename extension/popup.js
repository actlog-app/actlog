async function loadStats() {
  const dot = document.getElementById('dot');
  const activeTime = document.getElementById('active-time');
  const curInfo = document.getElementById('cur-info');

  try {
    const res = await fetch('http://127.0.0.1:5566/api/stats/summary');
    if (!res.ok) throw new Error('Daemon unreachable');
    const data = await res.json();

    const hours = Math.floor(data.total_active_seconds / 3600);
    const mins = Math.floor((data.total_active_seconds % 3600) / 60);
    activeTime.innerText = `${hours}h ${mins}m`;

    const [activeTab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
    if (activeTab) {
      curInfo.innerText = `Current Tab: ${activeTab.title || activeTab.url}`;
    }
    dot.style.background = '#4CAF50';
  } catch (_) {
    dot.style.background = '#F44336';
    activeTime.innerText = 'Offline';
    curInfo.innerText = 'ActLog desktop app is not running.';
  }
}

loadStats();

