import React from 'react';

import { FullImage, DualImages } from './components/Attachment';
import { SplitLayout } from './components/SplitLayout';
import { Note } from './components/Note';
import { Table } from './components/Table';
import { Annotation, AnnotationRef } from './components/Annotation';
import { NoGlossary } from './components/NoGlossary';
import { Chart } from './MDXClientImports';
import SmartText from './components/SmartText'; 
import { MDXErrorBoundary } from './MDXErrorBoundary';
import { Details } from './components/Details';
import { Timeline } from './components/Timeline';
import { Glossary } from './components/Glossary';

// import './styles/MDXComponents.module.css';

const extractText = (node: React.ReactNode): string => {
  if (!node) return '';
  if (typeof node === 'string' || typeof node === 'number') return String(node);
  if (Array.isArray(node)) return node.map(extractText).join('');
  if (React.isValidElement(node)) {
    const children = (node.props as { children?: React.ReactNode }).children;
    return extractText(children);
  }
  return '';
};

const slugify = (node: React.ReactNode) => {
  const text = extractText(node);

  return text.toLowerCase().trim().replace(/\s+/g, '-').replace(/\-\-+/g, '-');
};

const withErrorBoundary = (Component: React.ComponentType<any>, name: string) => {
  return (props: any) => (
    <MDXErrorBoundary componentName={name}>
      <Component {...props} />
    </MDXErrorBoundary>
  );
};

export const MDX_COMPONENTS = {
  Note: withErrorBoundary(Note, 'Note'),
  Chart: withErrorBoundary(Chart, 'Chart'),
  Details: withErrorBoundary(Details, 'Details'),
  Timeline: withErrorBoundary(Timeline, 'Timeline'),
  Glossary: withErrorBoundary(Glossary, 'Glossary'),

  SplitLayout: withErrorBoundary(SplitLayout, 'SplitLayout'),
  FullImage: withErrorBoundary(FullImage, 'FullImage'),
  DualImages: withErrorBoundary(DualImages, 'DualImages'),
  Table: withErrorBoundary(Table, 'Table'),
  Annotation: withErrorBoundary(Annotation, 'Annotation'),
  AnnotationRef: withErrorBoundary(AnnotationRef, 'AnnotationRef'),
  NoGlossary: withErrorBoundary(NoGlossary, 'NoGlossary'),

  pre: (props: React.ComponentPropsWithoutRef<'pre'>) => <pre {...props} className="pre" />,
  code: (props: React.ComponentPropsWithoutRef<'code'>) => <code {...props} className="code" />,
  
  h1: (props: React.ComponentPropsWithoutRef<'h1'>) => {
    const id = slugify(props.children);
    return <h1 className="h1" id={id} {...props}><SmartText>{props.children}</SmartText></h1> 
  },
  
  h2: (props: React.ComponentPropsWithoutRef<'h2'>) => {
    const id = slugify(props.children);
    return <h2 id={id} className="h2" {...props}><SmartText>{props.children}</SmartText></h2>
  },
  
  p: (props: React.ComponentPropsWithoutRef<'p'>) => <p className="p" {...props}><SmartText>{props.children}</SmartText></p>,
  
  ul: (props: React.ComponentPropsWithoutRef<'ul'>) => (
    <ul className="ul" {...props} />
  ),
  
  ol: (props: React.ComponentPropsWithoutRef<'ol'>) => (
    <ol className="ol" {...props} />
  ),

  li: (props: React.ComponentPropsWithoutRef<'li'>) => (
    <li className="li">
      <SmartText>{props.children}</SmartText>
    </li>
  ),

  blockquote: (props: React.ComponentPropsWithoutRef<'blockquote'>) => <blockquote className="blockquote" {...props}><SmartText>{props.children}</SmartText></blockquote>,
};