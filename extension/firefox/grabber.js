let allLinks = [];

// Load links from storage
chrome.storage.local.get(['grabbedLinks'], (result) => {
  allLinks = result.grabbedLinks || [];
  renderLinks();
});

function renderLinks() {
  const filter = document.getElementById('extensionFilter').value.toLowerCase();
  const list = document.getElementById('linkList');
  const stats = document.getElementById('stats');
  
  const filtered = allLinks.filter(l => l.url.toLowerCase().includes(filter));
  
  list.innerHTML = filtered.map((link, idx) => `
    <div class="link-item">
      <input type="checkbox" checked data-url="${link.url}">
      <div class="link-name">${link.text || 'Untitled'}</div>
      <div class="link-url">${link.url}</div>
    </div>
  `).join('');
  
  stats.innerText = `Found ${allLinks.length} links (showing ${filtered.length})`;
}

document.getElementById('extensionFilter').addEventListener('input', renderLinks);

document.getElementById('selectAll').onclick = () => {
  document.querySelectorAll('input[type="checkbox"]').forEach(cb => cb.checked = true);
};

document.getElementById('selectNone').onclick = () => {
  document.querySelectorAll('input[type="checkbox"]').forEach(cb => cb.checked = false);
};

document.getElementById('cancelBtn').onclick = () => window.close();

document.getElementById('downloadBtn').onclick = async () => {
  const selected = Array.from(document.querySelectorAll('input[type="checkbox"]:checked'))
    .map(cb => cb.dataset.url);
    
  if (selected.length === 0) {
    alert("Please select at least one file.");
    return;
  }

  const btn = document.getElementById('downloadBtn');
  btn.disabled = true;
  btn.innerText = "Sending...";

  for (const url of selected) {
    try {
      await fetch("http://localhost:6246/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ 
          url: url, 
          folder: "/home/rubait/Downloads"
        })
      });
    } catch (e) {
      console.error("Failed to send", url, e);
    }
  }
  
  window.close();
};
