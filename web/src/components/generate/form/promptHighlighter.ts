export type WeightHighlightConfig = {
  parenthesisBoost: number;
  maxDeltaForIntensity: number;
  upHue: number;
  downHue: number;
  saturation: number;
  saturationBoost: number;
  baseLightness: number;
  lightnessDelta: number;
  intensityExponent: number;
  neutralColor: string;
  colonColor: string;
};

export const defaultHighlightConfig: WeightHighlightConfig = {
  parenthesisBoost: 1.1,
  maxDeltaForIntensity: 3,
  upHue: 12,
  downHue: 208,
  saturation: 68,
  saturationBoost: 18,
  baseLightness: 56,
  lightnessDelta: 26,
  intensityExponent: 0.7,
  neutralColor: "#6b7280",
  colonColor: "#16a34a",
};

const numericWeightPattern = /^[-+]?(?:\d+(?:\.\d+)?|\.\d+)(?=::)/;
const snippetPattern = /^<\s*snippet:([^>\s]+)\s*>/i;
// Only treat bracketed/brace groups with :number as weighted; avoid plain colon tags (e.g., artist:foo).
const explicitGroupPattern =
  /^(\[|\{)([^()\[\]{}]+?):\s*(-?\d+(?:\.\d+)?)(\]|\})/;

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function weightToColor(weight: number, config: WeightHighlightConfig): string {
  if (!Number.isFinite(weight) || weight === 1) return "";
  const delta = Math.min(Math.abs(weight - 1), config.maxDeltaForIntensity);
  const normalized = Math.min(delta / config.maxDeltaForIntensity, 1);
  const intensity = Math.pow(normalized, config.intensityExponent);
  const hue = weight > 1 ? config.upHue : config.downHue;
  const lightness = config.baseLightness - intensity * config.lightnessDelta;
  const saturation = config.saturation + intensity * config.saturationBoost;
  return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
}

function wrap(
  text: string,
  weight: number,
  config: WeightHighlightConfig
): string {
  const color = weightToColor(weight, config);
  if (!color) return text;
  return `<span style="color:${color}">${text}</span>`;
}

function countRun(text: string, start: number, ch: string): number {
  let len = 0;
  while (text[start + len] === ch) len += 1;
  return len;
}

function consumeBalancedBlock(
  text: string,
  start: number,
  openChar: string,
  closeChar: string
): { inner: string; depth: number; end: number } | null {
  const runLen = countRun(text, start, openChar);
  if (!runLen) return null;
  const openRun = openChar.repeat(runLen);
  const closeRun = closeChar.repeat(runLen);

  let level = 1;
  let idx = start + runLen;
  while (idx < text.length) {
    if (text.startsWith(openRun, idx)) {
      level += 1;
      idx += runLen;
      continue;
    }
    if (text.startsWith(closeRun, idx)) {
      level -= 1;
      if (level === 0) {
        const end = idx + runLen;
        const inner = text.slice(start + runLen, idx);
        return { inner, depth: runLen, end };
      }
      idx += runLen;
      continue;
    }
    idx += 1;
  }
  return null;
}

function renderSegment(
  text: string,
  config: WeightHighlightConfig,
  weight: number
): string {
  let i = 0;
  const out: string[] = [];

  const applyWeight = (segment: string, factor: number) =>
    wrap(segment, weight * factor, config);

  while (i < text.length) {
    if (text[i] === "<") {
      const snippetMatch = text.slice(i).match(snippetPattern);
      if (snippetMatch) {
        const [full] = snippetMatch;
        out.push(`<span class="prompt-snippet">${escapeHtml(full)}</span>`);
        i += full.length;
        continue;
      }
    }

    if (text.startsWith("||", i)) {
      const close = text.indexOf("||", i + 2);
      if (close !== -1) {
        const inner = text.slice(i + 2, close);
        const options = inner
          .split("|")
          .map((opt) => renderSegment(opt, config, weight));
        out.push('<span class="prompt-random">||</span>');
        out.push(options.join('<span class="prompt-random-sep">|</span>'));
        out.push('<span class="prompt-random">||</span>');
        i = close + 2;
        continue;
      }
    }

    const numericMatch = text.slice(i).match(numericWeightPattern);
    if (numericMatch) {
      const prefixLen = numericMatch[0].length + 2;
      const contentStart = i + prefixLen;
      const close = text.indexOf("::", contentStart);
      if (close !== -1) {
        const numericValue = parseFloat(numericMatch[0]);
        out.push(
          wrap(escapeHtml(text.slice(i, contentStart)), numericValue, config)
        );
        const inner = text.slice(contentStart, close);
        out.push(renderSegment(inner, config, weight * numericValue));
        out.push('<span class="prompt-weight-close">::</span>');
        i = close + 2;
        continue;
      }
    }

    const rest = text.slice(i);
    const explicitWeightedGroup = rest.match(explicitGroupPattern);
    if (explicitWeightedGroup) {
      const [, open, inner, weightStr, close] = explicitWeightedGroup;
      const explicitWeight = parseFloat(weightStr);
      out.push(`<span class="prompt-bracket">${escapeHtml(open)}</span>`);
      out.push(renderSegment(inner, config, weight * explicitWeight));
      out.push(
        wrap(escapeHtml(`:${weightStr}`), weight * explicitWeight, config)
      );
      out.push(`<span class="prompt-bracket">${escapeHtml(close)}</span>`);
      i += explicitWeightedGroup[0].length;
      continue;
    }

    const braceBlock =
      text[i] === "{" ? consumeBalancedBlock(text, i, "{", "}") : null;
    if (braceBlock) {
      const factor = Math.pow(config.parenthesisBoost, braceBlock.depth);
      out.push(
        `<span class="prompt-bracket">${"{".repeat(braceBlock.depth)}</span>`
      );
      out.push(renderSegment(braceBlock.inner, config, weight * factor));
      out.push(
        `<span class="prompt-bracket">${"}".repeat(braceBlock.depth)}</span>`
      );
      i = braceBlock.end;
      continue;
    }

    const bracketBlock =
      text[i] === "[" ? consumeBalancedBlock(text, i, "[", "]") : null;
    if (bracketBlock) {
      const factor = Math.pow(config.parenthesisBoost, bracketBlock.depth * -1);
      out.push(
        `<span class="prompt-bracket">${"[".repeat(bracketBlock.depth)}</span>`
      );
      out.push(renderSegment(bracketBlock.inner, config, weight * factor));
      out.push(
        `<span class="prompt-bracket">${"]".repeat(bracketBlock.depth)}</span>`
      );
      i = bracketBlock.end;
      continue;
    }

    const ch = text[i];
    if (ch === "\n") {
      out.push("<br/>");
      i += 1;
      continue;
    }

    if (ch === "|") {
      out.push('<span class="prompt-mix-sep">|</span>');
      i += 1;
      continue;
    }

    out.push(applyWeight(escapeHtml(ch), 1));
    i += 1;
  }

  return out.join("");
}

export function renderHighlighted(
  text: string,
  config: WeightHighlightConfig
): string {
  if (!text) return "";
  return renderSegment(text, config, 1);
}
