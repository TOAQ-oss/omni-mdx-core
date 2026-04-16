import React from "react";

// Styles defined to be usable in fallback condition.
export const BASIC_STYLES: Record<string, React.FC<any>> = {
  h1: (props) => <h1 className="text-3xl font-bold tracking-tight text-white mt-8 mb-4" {...props} />,
  h2: (props) => <h2 className="text-2xl font-semibold tracking-tight text-neutral-100 mt-6 mb-3 border-b border-white/5 pb-2" {...props} />,
  h3: (props) => <h3 className="text-xl font-medium text-neutral-200 mt-4 mb-2" {...props} />,
  h4: (props) => <h4 className="text-lg font-medium text-neutral-300 mt-4 mb-2" {...props} />,
  p: (props) => <p className="text-base leading-7 text-neutral-400 mb-4" {...props} />,
  ul: (props) => <ul className="list-disc list-inside mb-4 space-y-1 text-neutral-400" {...props} />,
  ol: (props) => <ol className="list-decimal list-inside mb-4 space-y-1 text-neutral-400" {...props} />,
  li: (props) => <li className="ml-4" {...props} />,
  code: (props) => {
    return <code className="inline-code bg-white/10 px-1.5 py-0.5 rounded text-pink-400 font-mono text-sm" {...props} />;
  },
  blockquote: (props) => <blockquote className="border-l-4 border-blue-500/50 pl-4 italic text-neutral-500 my-6" {...props} />,
  hr: () => <hr className="border-white/5 my-8" />,
  table: (props) => <div className="overflow-x-auto mb-6"><table className="w-full text-sm text-left border-collapse" {...props} /></div>,
  th: (props) => <th className="border-b border-white/10 p-2 font-semibold text-neutral-200" {...props} />,
  td: (props) => <td className="border-b border-white/5 p-2 text-neutral-400" {...props} />,
  img: ({ children, ...props }) => <img className="rounded-xl border border-white/10 my-8 mx-auto max-w-full h-auto" {...props} />,
  a: (props) => <a className="text-blue-400 hover:text-blue-300 underline underline-offset-4 decoration-blue-500/30 transition-colors" {...props} />,
};