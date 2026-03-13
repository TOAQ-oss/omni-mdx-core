import type { AstNode } from "./MDXServerRenderer";
import { createRequire } from "module";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";

declare const __non_webpack_require__: NodeRequire | undefined;

const _require = createRequire(import.meta.url);
const nativeRequire: NodeRequire =
  typeof __non_webpack_require__ !== "undefined"
    ? __non_webpack_require__
    : _require;

let _parse: ((mdx: string) => string) | null = null;

async function getParser(): Promise<(mdx: string) => string> {
  if (_parse) return _parse;

  const platformPackages = [
    "@toaq-oss/mdx-engine-darwin-arm64",
    "@toaq-oss/mdx-engine-darwin-x64",
    "@toaq-oss/mdx-engine-linux-x64-gnu",
    "@toaq-oss/mdx-engine-win32-x64-msvc",
  ];

  for (const pkg of platformPackages) {
    try {
      const native = nativeRequire(pkg);
      if (typeof native.parse === "function") {
        _parse = native.parse;
        return _parse!;
      }
    } catch { /* not installed on this platform */ }
  }

  // Fallback local
  try {
    const __dir    = dirname(fileURLToPath(import.meta.url));
    const nativeDir = resolve(__dir, "../native");
    const fs = nativeRequire("fs") as typeof import("fs");

    if (fs.existsSync(nativeDir)) {
      const files = fs.readdirSync(nativeDir).filter((f: string) => f.endsWith(".node"));
      const platform = process.platform ?? "";
      const arch     = process.arch     ?? "";
      const match    = files.find((f: string) => f.includes(platform) && f.includes(arch))
                    ?? files[0];

      if (match) {
        const native = nativeRequire(resolve(nativeDir, match));
        if (typeof native.parse === "function") {
          _parse = native.parse;
          return _parse!;
        }
      }
    }
  } catch { /* native/ absent */ }

  throw new Error(
    `[toaq-oss/mdx-engine] Native parser not found for this platform.\n` +
    `  Tried npm packages: ${platformPackages.join(", ")}\n` +
    `  ➜ Install the prebuilt package (once published):\n` +
    `      npm install @toaq-oss/mdx-engine-darwin-arm64   # Mac M1/M2\n` +
    `      npm install @toaq-oss/mdx-engine-linux-x64-gnu  # Linux/Vercel\n` +
    `      npm install @toaq-oss/mdx-engine-win32-x64-msvc # Windows\n`
  );
}

export async function parseMdx(mdx: string): Promise<AstNode[]> {
  const parse = await getParser();
  let json: string;
  try {
    json = parse(mdx);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }
  try {
    return JSON.parse(json) as AstNode[];
  } catch {
    throw new Error("[toaq-oss/mdx-engine] Parser returned invalid JSON.");
  }
}

export function parseMdxSync(mdx: string): AstNode[] {
  if (!_parse) throw new Error("[toaq-oss/mdx-engine] parseMdxSync() called before init.");
  try {
    return JSON.parse(_parse(mdx)) as AstNode[];
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }
}

export class MDXParseError extends Error {
  readonly source: string;
  constructor(message: string, source: string) {
    super(`MDX parse error: ${message}`);
    this.name   = "MDXParseError";
    this.source = source;
  }
}