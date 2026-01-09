use axum::response::Html;

use super::style;

const COMPARE_CSS: &str = r"
body { min-height: 100vh; padding: 20px; }
.container { max-width: 900px; margin: 0 auto; }
header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
h1 { font-size: 1.5rem; font-weight: 300; }
.progress { color: var(--muted); font-size: 0.9rem; }
.photos { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin-bottom: 2rem; }
.photo { position: relative; aspect-ratio: 1; background: #333; border-radius: 8px; overflow: hidden; cursor: pointer; border: 3px solid transparent; transition: border-color 0.2s, transform 0.2s; }
.photo img { width: 100%; height: 100%; object-fit: cover; }
.photo:hover { border-color: var(--fg); transform: scale(1.02); }
.photo.selected { border-color: var(--fg); }
.photo.rank-1 { border-color: gold; }
.photo.rank-2 { border-color: silver; }
.photo.rank-3 { border-color: #cd7f32; }
.photo .rank-badge { position: absolute; top: 8px; left: 8px; width: 28px; height: 28px; border-radius: 50%; background: rgba(0,0,0,0.7); color: white; display: flex; align-items: center; justify-content: center; font-weight: bold; font-size: 0.9rem; }
.ranking { display: flex; gap: 0.5rem; margin-bottom: 1rem; min-height: 2rem; flex-wrap: wrap; }
.rank-slot { padding: 0.5rem 1rem; background: #333; border-radius: 4px; font-size: 0.9rem; }
.actions { display: flex; gap: 1rem; }
.btn { padding: 0.75rem 1.5rem; border: none; border-radius: 6px; cursor: pointer; font-size: 1rem; }
.btn-primary { background: var(--fg); color: var(--bg); }
.btn-secondary { background: #333; color: var(--fg); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.status { color: var(--muted); text-align: center; padding: 2rem; }
.error { color: #e74c3c; }
.loading { animation: pulse 1.5s ease-in-out infinite; }
@keyframes pulse { 0%, 100% { opacity: 0.4; } 50% { opacity: 0.7; } }
";

const COMPARE_JS: &str = r#"
let matchupId = null, photoIndices = [], ranking = [];

async function loadMatchup() {
    try {
        const res = await fetch('/api/matchup', { method: 'POST' });
        if (!res.ok) { showStatus(await res.text() || 'Failed to load matchup', true); return; }
        const data = await res.json();
        matchupId = data.matchup_id;
        photoIndices = data.photo_indices;
        ranking = [];
        renderPhotos();
        loadProgress();
    } catch (e) { showStatus('Network error', true); }
}

async function loadProgress() {
    try {
        const res = await fetch('/api/progress');
        if (res.ok) {
            const data = await res.json();
            document.getElementById('progress').textContent =
                `${data.percent}% (${data.compared_pairs}/${data.total_pairs} pairs)`;
        }
    } catch (e) { }
}

function renderPhotos() {
    document.getElementById('content').innerHTML = `
        <div class="photos">${photoIndices.map(idx => `
            <div class="photo loading" data-idx="${idx}" onclick="selectPhoto(${idx})">
                <img src="/img/thumb/${idx}" alt="Photo ${idx}"
                     onload="this.parentElement.classList.remove('loading'); loadPreview(this, ${idx});">
            </div>
        `).join('')}</div>
        <div class="ranking" id="ranking"><span style="color: var(--muted);">Click photos in order: best → worst</span></div>
        <div class="actions">
            <button class="btn btn-secondary" onclick="clearRanking()">Clear</button>
            <button class="btn btn-primary" id="submitBtn" disabled onclick="submitRanking()">Submit</button>
        </div>`;
}

function loadPreview(img, idx) {
    const preview = new Image();
    preview.onload = () => { img.src = preview.src; };
    preview.src = `/img/preview/${idx}`;
}

function selectPhoto(idx) {
    if (ranking.includes(idx)) return;
    ranking.push(idx);
    updateUI();
    if (ranking.length === photoIndices.length) document.getElementById('submitBtn').disabled = false;
}

function clearRanking() { ranking = []; updateUI(); document.getElementById('submitBtn').disabled = true; }

function updateUI() {
    document.querySelectorAll('.photo').forEach(el => {
        const idx = parseInt(el.dataset.idx), rankPos = ranking.indexOf(idx);
        el.className = el.className.replace(/ selected| rank-\d/g, '') + (rankPos >= 0 ? ` selected rank-${rankPos + 1}` : '');
        const badge = el.querySelector('.rank-badge');
        if (rankPos >= 0) {
            if (!badge) el.insertAdjacentHTML('beforeend', `<div class="rank-badge">${rankPos + 1}</div>`);
            else badge.textContent = rankPos + 1;
        } else if (badge) badge.remove();
    });
    const r = document.getElementById('ranking');
    r.innerHTML = ranking.length === 0
        ? '<span style="color: var(--muted);">Click photos in order: best → worst</span>'
        : ranking.map((idx, i) => `<span class="rank-slot">${i + 1}. Photo ${idx}</span>`).join('');
}

async function submitRanking() {
    try {
        const res = await fetch('/api/compare', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ matchup_id: matchupId, ranked_photo_indices: ranking })
        });
        if (!res.ok) { showStatus('Failed to submit', true); return; }
        loadMatchup();
    } catch (e) { showStatus('Network error', true); }
}

function showStatus(msg, isError) {
    document.getElementById('content').innerHTML = `<p class="status ${isError ? 'error' : ''}">${msg}</p>`;
}

loadMatchup();
"#;

pub async fn page() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Compare - Filmorator</title>
    <style>{css_reset}{css_vars}{compare_css}</style>
</head>
<body>
    <div class="container">
        <header><h1>Compare Photos</h1><span class="progress" id="progress">Loading...</span></header>
        <div id="content"><p class="status">Loading matchup...</p></div>
        <div style="margin-top: 2rem; text-align: center;"><a href="/" style="color: var(--muted);">&larr; Back to home</a></div>
    </div>
    <script>{compare_js}</script>
</body>
</html>"#,
        css_reset = style::CSS_RESET,
        css_vars = style::CSS_VARS,
        compare_css = COMPARE_CSS,
        compare_js = COMPARE_JS,
    ))
}
