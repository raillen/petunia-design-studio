const MANIFEST_URL = "./docs/manifest.json";
const nav = document.querySelector("#docNav");
const content = document.querySelector("#content");
const pageToc = document.querySelector("#pageToc");
const search = document.querySelector("#search");
const searchResults = document.querySelector("#searchResults");
const sidebar = document.querySelector("#sidebar");
const mobileMenu = document.querySelector("#mobileMenu");
const cache = new Map();
let manifest;

const esc = (s) => s.replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const slug = (s) => s.toLowerCase().normalize("NFD").replace(/[\u0300-\u036f]/g,"").replace(/[^a-z0-9]+/g,"-").replace(/^-|-$/g,"");

function inline(text) {
  let out = esc(text);
  out = out.replace(/\`([^\`]+)\`/g, "<code>$1</code>");
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  out = out.replace(/\[([^\]]+)\]\((https?:\/\/[^)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
  return out;
}

function parseMarkdown(md) {
  const lines = md.replace(/\r/g, "").split("\n");
  const html = [];
  let i = 0, inCode = false, code = [], list = null;
  const closeList = () => { if (list) { html.push(`</${list}>`); list = null; } };

  while (i < lines.length) {
    const line = lines[i];

    if (line.startsWith("```")) {
      closeList();
      if (!inCode) { inCode = true; code = []; }
      else { html.push(`<pre><code>${esc(code.join("\n"))}</code></pre>`); inCode = false; }
      i++; continue;
    }
    if (inCode) { code.push(line); i++; continue; }

    const heading = /^(#{1,4})\s+(.+)$/.exec(line);
    if (heading) {
      closeList();
      const level = heading[1].length;
      const title = heading[2].replace(/[*`]/g, "");
      html.push(`<h${level} id="${slug(title)}">${inline(heading[2])}</h${level}>`);
      i++; continue;
    }

    if (line.trim() && i + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[i+1])) {
      closeList();
      const headers = line.split("|").map(x=>x.trim()).filter(Boolean);
      i += 2;
      const rows = [];
      while (i < lines.length && lines[i].includes("|") && lines[i].trim()) {
        rows.push(lines[i].split("|").map(x=>x.trim()).filter(Boolean)); i++;
      }
      html.push("<table><thead><tr>" + headers.map(x=>`<th>${inline(x)}</th>`).join("") + "</tr></thead><tbody>" +
        rows.map(r=>"<tr>"+r.map(x=>`<td>${inline(x)}</td>`).join("")+"</tr>").join("") + "</tbody></table>");
      continue;
    }

    const ul = /^\s*[-*]\s+(.+)$/.exec(line);
    const ol = /^\s*\d+\.\s+(.+)$/.exec(line);
    if (ul || ol) {
      const type = ul ? "ul" : "ol";
      if (list !== type) { closeList(); list = type; html.push(`<${type}>`); }
      html.push(`<li>${inline((ul||ol)[1])}</li>`); i++; continue;
    }
    closeList();

    if (/^---+$/.test(line.trim())) { html.push("<hr>"); i++; continue; }
    if (line.startsWith("> ")) { html.push(`<blockquote><p>${inline(line.slice(2))}</p></blockquote>`); i++; continue; }
    if (!line.trim()) { i++; continue; }

    const para = [line.trim()]; i++;
    while (i < lines.length && lines[i].trim() && !/^(#{1,4})\s/.test(lines[i]) && !/^\s*[-*]\s+/.test(lines[i]) && !/^\s*\d+\.\s+/.test(lines[i]) && !lines[i].startsWith("```") && !lines[i].startsWith("> ")) {
      if (i + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[i+1])) break;
      para.push(lines[i].trim()); i++;
    }
    html.push(`<p>${inline(para.join(" "))}</p>`);
  }
  closeList();
  return html.join("\n");
}

function headings(md) {
  return md.split("\n").map(line => {
    const m = /^(#{2,3})\s+(.+)$/.exec(line);
    return m ? {level:m[1].length, title:m[2].replace(/[*`]/g,""), id:slug(m[2].replace(/[*`]/g,""))} : null;
  }).filter(Boolean);
}

async function loadText(path) {
  if (cache.has(path)) return cache.get(path);
  const res = await fetch("./docs/" + path);
  if (!res.ok) throw new Error(`Falha ao carregar ${path}`);
  const text = await res.text();
  cache.set(path, text);
  return text;
}

function fileMeta(path) {
  for (const domain of manifest.domains) {
    const file = domain.files.find(f => f.path === path);
    if (file) return {domain, file};
  }
  if (path === "about.md") return {domain:{title:"Projeto"}, file:{title:"Sobre", path}};
  return null;
}

function setPageToc(items) {
  pageToc.innerHTML = items.map(h => `<a data-level="${h.level}" href="#${h.id}">${esc(h.title)}</a>`).join("");
}

function setActive(path) {
  document.querySelectorAll(".file-link").forEach(a => a.classList.toggle("active", a.dataset.path === path));
}

async function openDoc(path, anchor) {
  try {
    const md = await loadText(path);
    const meta = fileMeta(path);
    content.innerHTML = `<div class="doc-kicker">${esc(meta?.domain.title || "Documentação")}</div>` + parseMarkdown(md);
    setPageToc(headings(md));
    setActive(path);
    document.title = `${meta?.file.title || "Documentação"} — Petunia Design`;
    if (anchor) requestAnimationFrame(() => document.getElementById(anchor)?.scrollIntoView());
    else window.scrollTo({top:0});
    content.focus({preventScroll:true});
    sidebar.classList.remove("open");
    mobileMenu.setAttribute("aria-expanded","false");
  } catch (err) {
    content.innerHTML = `<div class="error-card"><strong>Não foi possível carregar a página.</strong><p>${esc(err.message)}</p><p>Sirva a pasta <code>website/</code> por HTTP; navegadores normalmente bloqueiam <code>fetch()</code> quando o HTML é aberto diretamente por <code>file://</code>.</p></div>`;
  }
}

async function buildNav() {
  nav.innerHTML = "";
  for (const domain of manifest.domains) {
    const domainWrap = document.createElement("div");
    domainWrap.className = "nav-domain";
    const button = document.createElement("button");
    button.className = "nav-toggle"; button.type = "button"; button.setAttribute("aria-expanded","true");
    button.innerHTML = `<span class="chevron">›</span><span>${esc(domain.title)}</span>`;
    const children = document.createElement("div"); children.className = "nav-children";
    button.addEventListener("click", () => {
      const open = button.getAttribute("aria-expanded") === "true";
      button.setAttribute("aria-expanded", String(!open)); children.hidden = open;
    });
    domainWrap.append(button, children);

    for (const file of domain.files) {
      const md = await loadText(file.path);
      const wrap = document.createElement("div"); wrap.className = "nav-file";
      const row = document.createElement("button"); row.className = "nav-toggle"; row.type = "button"; row.setAttribute("aria-expanded","false");
      const link = document.createElement("a"); link.className = "file-link"; link.dataset.path = file.path; link.href = `#/docs/${file.path}`; link.textContent = file.title;
      row.innerHTML = '<span class="chevron">›</span>';
      row.append(link);
      const topics = document.createElement("div"); topics.className = "nav-children"; topics.hidden = true;
      headings(md).filter(h=>h.level===2).forEach(h => {
        const a = document.createElement("a"); a.className = "topic-link"; a.href = `#/docs/${file.path}#${h.id}`; a.textContent = h.title; topics.append(a);
      });
      row.addEventListener("click", e => {
        if (e.target.closest("a")) return;
        const open = row.getAttribute("aria-expanded")==="true"; row.setAttribute("aria-expanded",String(!open)); topics.hidden=open;
      });
      wrap.append(row, topics); children.append(wrap);
    }
    nav.append(domainWrap);
  }
}

async function preload() {
  await Promise.all(manifest.domains.flatMap(d => d.files.map(f => loadText(f.path))));
  await loadText("about.md");
}

function doSearch(q) {
  q = q.trim().toLowerCase();
  if (q.length < 2) { searchResults.hidden = true; return; }
  const results = [];
  for (const domain of manifest.domains) for (const file of domain.files) {
    const md = cache.get(file.path) || "";
    const hay = (file.title + "\n" + md).toLowerCase();
    const pos = hay.indexOf(q);
    if (pos >= 0) {
      const plain = md.replace(/[#*\`>|]/g," ").replace(/\s+/g," ");
      const p = plain.toLowerCase().indexOf(q);
      const excerpt = p >= 0 ? plain.slice(Math.max(0,p-55), p+q.length+95) : domain.title;
      results.push({domain:domain.title,file,excerpt});
    }
  }
  searchResults.innerHTML = results.slice(0,12).map(r => `<a class="search-result" href="#/docs/${r.file.path}"><strong>${esc(r.file.title)}</strong><small>${esc(r.domain)} · …${esc(r.excerpt)}…</small></a>`).join("") || '<div class="search-result">Nenhum resultado.</div>';
  searchResults.hidden = false;
}

function route() {
  const raw = location.hash || "#/docs/00-architecture/boundaries.md";
  const m = /^#\/docs\/([^#]+)(?:#(.+))?$/.exec(raw);
  openDoc(m ? m[1] : "00-architecture/boundaries.md", m?.[2]);
}

(async function init(){
  const res = await fetch(MANIFEST_URL);
  manifest = await res.json();
  await preload();
  await buildNav();
  route();
})();

window.addEventListener("hashchange", route);
search.addEventListener("input", () => doSearch(search.value));
search.addEventListener("keydown", e => { if (e.key==="Escape") { search.value=""; searchResults.hidden=true; search.blur(); }});
document.addEventListener("keydown", e => { if (e.key==="/" && document.activeElement !== search) { e.preventDefault(); search.focus(); }});
document.addEventListener("click", e => { if (!e.target.closest(".search-wrap")) searchResults.hidden = true; });
mobileMenu.addEventListener("click", () => { const open=sidebar.classList.toggle("open"); mobileMenu.setAttribute("aria-expanded",String(open)); });
document.querySelector("#collapseAll").addEventListener("click", () => {
  document.querySelectorAll(".nav-toggle").forEach(b=>b.setAttribute("aria-expanded","false"));
  document.querySelectorAll(".nav-children").forEach(el=>el.hidden=true);
});