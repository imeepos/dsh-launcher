// Masking: blank out comments/strings while keeping offsets and newlines.
const blankAt = (chars, j) => {
  if (chars[j] !== "\n") chars[j] = " ";
};

export function maskRust(text) {
  const chars = text.split("");
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    const raw = /^r(#*)"/.exec(text.slice(i));
    if (c === "/" && text[i + 1] === "/") {
      while (i < text.length && text[i] !== "\n") { blankAt(chars, i); i++; }
    } else if (c === "/" && text[i + 1] === "*") {
      let depth = 0;
      while (i < text.length) {
        if (text.startsWith("/*", i)) { depth++; blankAt(chars, i); blankAt(chars, i + 1); i += 2; }
        else if (text.startsWith("*/", i)) { depth--; blankAt(chars, i); blankAt(chars, i + 1); i += 2; if (depth === 0) break; }
        else { blankAt(chars, i); i++; }
      }
    } else if (raw) {
      const close = '"' + raw[1];
      const end = text.indexOf(close, i + raw[0].length);
      const stop = end < 0 ? text.length : end + close.length;
      for (let j = i; j < stop; j++) blankAt(chars, j);
      i = stop;
    } else if (c === '"') {
      blankAt(chars, i);
      let j = i + 1;
      while (j < text.length && text[j] !== '"' && text[j] !== "\n") {
        if (text[j] === "\\") { blankAt(chars, j); j++; }
        blankAt(chars, j);
        j++;
      }
      if (j < text.length && text[j] === '"') blankAt(chars, j);
      i = j + 1;
    } else if (c === "'") {
      const m = /^'(\\.|[^'\\])'/.exec(text.slice(i, i + 5));
      if (m) { for (let j = 0; j < m[0].length; j++) blankAt(chars, i + j); i += m[0].length; }
      else i++; // lifetime tick, not a char literal
    } else i++;
  }
  return chars.join("");
}

const PREQUOTE = "=([{,:;&|?+-*%~^<>!";
const KW_BEFORE = /(?:^|[^\w$])(?:return|throw|case|typeof|instanceof|new|delete|void|in|of|await|yield|import|export|default)$/;

function quoteStartsString(text, i) {
  let j = i - 1;
  while (j >= 0 && /\s/.test(text[j])) j--;
  if (j < 0) return true;
  if (PREQUOTE.includes(text[j])) return true;
  return KW_BEFORE.test(text.slice(Math.max(0, j - 12), j + 1));
}

export function maskTs(text) {
  const chars = text.split("");
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    if (c === "/" && text[i + 1] === "/") {
      while (i < text.length && text[i] !== "\n") { blankAt(chars, i); i++; }
    } else if (c === "/" && text[i + 1] === "*") {
      while (i < text.length && !text.startsWith("*/", i)) { blankAt(chars, i); i++; }
      blankAt(chars, i);
      blankAt(chars, i + 1);
      i += 2;
    } else if (c === '"' || c === "'" || c === "`") {
      const q = c;
      if (q !== "`" && !quoteStartsString(text, i)) { i++; continue; } // JSX/text apostrophe
      blankAt(chars, i);
      let j = i + 1;
      while (j < text.length) {
        if (text[j] === "\\") { blankAt(chars, j); blankAt(chars, j + 1); j += 2; continue; }
        if (text[j] === q || (text[j] === "\n" && q !== "`")) break;
        blankAt(chars, j);
        j++;
      }
      if (j < text.length && text[j] === q) blankAt(chars, j);
      i = j + 1;
    } else i++;
  }
  return chars.join("");
}
