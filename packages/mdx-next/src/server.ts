/**
 * Server-only entry point.
 * Import this in Server Components, API routes, and generateStaticParams.
 *
 * ✅ Next.js App Router RSC
 * ✅ getStaticProps / getServerSideProps (Pages Router)
 * ✅ Route Handlers (app/api/...)
 * ❌ Never import in "use client" files
 */

export const SERVER_READY = true;

export {
    parseMdx,
    parseMdxSync,
    compileToJsx,
    compileToJsxSync,
    MDXParseError
} from "./parse.server";
export { MDXServerRenderer } from "./MDXServerRenderer";
export type { AstNode, AttrValueKind, MDXComponents } from "./types/MdxAST";