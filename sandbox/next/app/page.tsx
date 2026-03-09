"use client";
import { useEffect, useMemo, useState } from 'react';
import initWasm, { MDXViewer, parse_mdx_to_json } from '@toaq-oss/mdx-engine';
import { MDX_COMPONENTS } from './components/MDXComponents';
import "katex/dist/katex.min.css";

const markdownContent = `
# Episode 1: The Evolution of Vocal Models

Welcome to the first analysis script for our podcast series. Today, we're looking at fundamental frequency compression.

<Note type="warning">
  Please note: The samples in this vocal dataset have not yet been normalized. 
  The entropy formula used for filtering is $H(X) = - \\sum P(x) \\log P(x)$.
</Note>

## Transcript of the exchange

<Speaker name="Dr. Laurent" time="02:15">
  We are seeing a clear improvement in latency during generation. The Rust architecture we compiled in WebAssembly is formidable.
</Speaker>

<Speaker name="Researcher Sophie" time="02:42">
  Exactly. The load test results confirm this empirically. Let's look at the metrics from our latest iteration.
</Speaker>

## Performance analysis

Here is the distribution of response times (in milliseconds) for our last 5 stress tests:

<DataChart title="Latency per iteration (ms)" data={[120, 85, 45, 30, 15]} />

Thank you for submitting your research!

Translated with DeepL.com (free version)
`;

const myConfig = {
  features: { math: true },
  components: MDX_COMPONENTS,
};

export default function TestPage() {
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then(() => setIsReady(true))
      .catch((err) => {
        console.error("Échec du chargement du moteur Rust:", err);
        setError("Impossible de charger le moteur de rendu.");
      });
  }, []);

  const { ast, parseError } = useMemo(() => {
    if (!isReady || !markdownContent) return { ast: null, parseError: null };

    try {
      const jsonAst = parse_mdx_to_json(markdownContent);
      return { 
        ast: JSON.parse(jsonAst), 
        parseError: null 
      };
    } catch (err) {
      return { 
        ast: null, 
        parseError: err instanceof Error ? err.message : String(err) 
      };
    }
  }, [isReady, markdownContent]);

  if (error || parseError) return <p className="p-8 text-red-600">Erreur : {error || parseError}</p>;
  if (!isReady || !ast) return <p className="p-8">Chargement du moteur Rust ultra-rapide...</p>;

  return (
    <main className="max-w-3xl mx-auto p-8">
      <MDXViewer ast={ast} config={{ features: { math: true }, components: MDX_COMPONENTS }} />
    </main>
  );
}