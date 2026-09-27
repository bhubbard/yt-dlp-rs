document.addEventListener('DOMContentLoaded', () => {
  const videoUrl = document.getElementById('video-url');
  const formatSelect = document.getElementById('format-select');
  const outputPath = document.getElementById('output-path');
  const fragments = document.getElementById('concurrent-fragments');
  const listFormats = document.getElementById('list-formats');
  const dumpJson = document.getElementById('dump-json');
  const commandEl = document.getElementById('generated-command');
  const copyBtn = document.getElementById('copy-btn');

  function updateCommand() {
    const parts = ['yt-dlp'];
    const url = (videoUrl.value || '').trim();

    if (url) {
      parts.push(`"${url}"`);
    }

    if (listFormats.checked) {
      parts.push('-F');
    } else if (dumpJson.checked) {
      parts.push('-j');
    } else {
      const fmt = formatSelect.value;
      if (fmt && fmt !== 'best') {
        parts.push(`-f ${fmt}`);
      }

      const out = (outputPath.value || '').trim();
      if (out && out !== '.') {
        parts.push(`-P ${out}`);
      }

      const n = parseInt(fragments.value, 10);
      if (n && n !== 4) {
        parts.push(`-N ${n}`);
      }
    }

    commandEl.textContent = parts.join(' ');
  }

  [videoUrl, formatSelect, outputPath, fragments, listFormats, dumpJson].forEach(el => {
    el.addEventListener('input', updateCommand);
    el.addEventListener('change', updateCommand);
  });

  copyBtn.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(commandEl.textContent);
      copyBtn.textContent = 'Copied!';
      setTimeout(() => {
        copyBtn.textContent = 'Copy';
      }, 2000);
    } catch (e) {
      console.error(e);
    }
  });

  updateCommand();
});
