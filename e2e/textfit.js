// #182/AK4: finds cut-off and overlapping text in the visible part of the user interface.
//
// Cut off: an element that clips its content (overflow hidden/clip or text-overflow: ellipsis) and whose text is
// wider or higher than the element; a select, button or input whose text does not fit into the control; a frame that
// scrolls sideways.
// Overlapping: two visible pieces of text of different elements whose boxes intersect (each line of a text node is
// one box, clipped by its scrolling or clipping ancestors). Text inside charts (SVG) counts as well.
// Only the given root is checked (an open dialog, the help window or the page); hidden elements are ignored.

/** Runs in the page: list of findings as readable strings. */
function findTextProblems(rootSelector) {
  const root = document.querySelector(rootSelector);
  if (!root) return [`root ${rootSelector} not found`];
  const TOL = 1.5; // px: rounding of fonts and borders
  const label = el => {
    const id = el.id ? `#${el.id}` : "";
    const cls = typeof el.className === "string" && el.className ? "." + el.className.trim().split(/\s+/).join(".") : "";
    const text = (el.innerText ?? el.textContent ?? "").trim().replace(/\s+/g, " ").slice(0, 60);
    return `${el.tagName.toLowerCase()}${id}${cls} «${text}»`;
  };
  const visCache = new Map();
  const visible = el => {
    if (visCache.has(el)) return visCache.get(el);
    const v = el.checkVisibility({ opacityProperty: true, visibilityProperty: true }) && !el.closest("dialog:not([open]), [hidden]");
    visCache.set(el, v);
    return v;
  };
  /** Rectangle of the visible area of an element (viewport and clipping ancestors). */
  const clipCache = new Map();
  const clipOf = el => {
    if (!el || el === document.body || el === document.documentElement) return { left: 0, top: 0, right: innerWidth, bottom: innerHeight };
    if (clipCache.has(el)) return clipCache.get(el);
    let r = clipOf(el.parentElement);
    const s = getComputedStyle(el);
    if (/(hidden|clip|auto|scroll)/.test(s.overflowX + s.overflowY)) {
      const b = el.getBoundingClientRect();
      r = { left: Math.max(r.left, b.left), top: Math.max(r.top, b.top), right: Math.min(r.right, b.right), bottom: Math.min(r.bottom, b.bottom) };
    }
    clipCache.set(el, r);
    return r;
  };
  const inter = (a, b) => ({ left: Math.max(a.left, b.left), top: Math.max(a.top, b.top), right: Math.min(a.right, b.right), bottom: Math.min(a.bottom, b.bottom) });
  const area = r => Math.max(0, r.right - r.left) * Math.max(0, r.bottom - r.top);

  const problems = [];
  const ctx = document.createElement("canvas").getContext("2d");

  // 1. Cut off
  for (const el of root.querySelectorAll("*")) {
    if (el instanceof SVGElement) continue;
    // A frame that scrolls sideways hides part of its text (at the window size of the app nothing should need it)
    if (/(auto|scroll)/.test(getComputedStyle(el).overflowX) && el.scrollWidth > el.clientWidth + TOL && visible(el)) {
      problems.push(`scrolls sideways: ${label(el)} (${el.scrollWidth} in ${el.clientWidth})`);
    }
    const control = ["SELECT", "BUTTON", "INPUT"].includes(el.tagName);
    const own = control || [...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim());
    if (!own || !visible(el)) continue;
    const s = getComputedStyle(el);
    if (["SELECT", "BUTTON", "INPUT"].includes(el.tagName)) {
      let text = el.tagName === "SELECT" ? el.selectedOptions[0]?.textContent ?? "" : el.tagName === "INPUT" ? (["text", "search", ""].includes(el.type) ? el.value || el.placeholder : "") : "";
      if (el.tagName === "BUTTON") {
        if (el.scrollWidth > el.clientWidth + TOL && /(hidden|clip)/.test(s.overflowX)) problems.push(`cut off: ${label(el)}`);
        continue;
      }
      if (!text.trim()) continue;
      ctx.font = `${s.fontStyle} ${s.fontWeight} ${s.fontSize} ${s.fontFamily}`;
      const need = ctx.measureText(text.trim()).width + parseFloat(s.paddingLeft) + parseFloat(s.paddingRight) + (el.tagName === "SELECT" ? 20 : 0); // arrow of a select (measured in Chrome)
      if (need > el.clientWidth + TOL) problems.push(`cut off: ${label(el)} (needs ${Math.round(need)} px, has ${el.clientWidth})`);
      continue;
    }
    const clips = /(hidden|clip)/.test(s.overflowX) || /(hidden|clip)/.test(s.overflowY) || s.textOverflow === "ellipsis";
    if (!clips) continue;
    if (el.scrollWidth > el.clientWidth + TOL || el.scrollHeight > el.clientHeight + TOL) problems.push(`cut off: ${label(el)} (${el.scrollWidth}×${el.scrollHeight} in ${el.clientWidth}×${el.clientHeight})`);
  }

  // 2. Overlapping
  const boxes = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, { acceptNode: n => n.textContent.trim() ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT });
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const el = n.parentElement;
    if (!el || !visible(el) || el.closest("option, title, style, script")) continue;
    const clip = clipOf(el);
    const range = document.createRange(); range.selectNodeContents(n);
    for (const r of range.getClientRects()) {
      const v = inter(r, clip);
      if (area(v) > 2) boxes.push({ el, r: v });
    }
  }
  const seen = new Set();
  boxes.sort((a, b) => a.r.top - b.r.top);
  for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length && boxes[j].r.top < boxes[i].r.bottom - TOL; j++) {
    const a = boxes[i], b = boxes[j];
    if (a.el === b.el) continue;
    const x = inter(a.r, b.r);
    const w = x.right - x.left, h = x.bottom - x.top;
    if (w <= TOL || h <= TOL) continue;
    // Ignore the small overlap of neighbouring lines (line-height smaller than the glyph box)
    if (h < 0.35 * Math.min(a.r.bottom - a.r.top, b.r.bottom - b.r.top)) continue;
    const key = label(a.el) + " ↔ " + label(b.el);
    if (seen.has(key)) continue;
    seen.add(key);
    problems.push(`overlapping: ${key}`);
  }
  return problems;
}

/** Findings in the root of the page (default: the visible page without closed dialogs). */
async function textProblems(page, rootSelector = "body") {
  return page.evaluate(findTextProblems, rootSelector);
}

module.exports = { textProblems };
