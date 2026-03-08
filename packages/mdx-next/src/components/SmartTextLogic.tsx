"use client";

import React from 'react';
import * as Tooltip from '@radix-ui/react-tooltip';
import { useGlossary } from '../context/GlossaryContent';
import styles from '../styles/SmartText.module.css';

const GlossaryTooltip = ({ text, def }: { text: string, def: string }) => {
  return (
    <Tooltip.Provider delayDuration={300}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <span className={styles.glossaryTrigger}>
            {text}
          </span>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content 
            className={styles.tooltipContent}
            sideOffset={5}
            side="top"
          >
            {def}
            <Tooltip.Arrow className={styles.tooltipArrow} />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
};

export default function SmartText ({ children }: { children: React.ReactNode }) {
  const { terms } = useGlossary();

  const scanForGlossary = (text: string): (string | React.ReactNode)[] => {
    if (terms.length === 0 || !text.trim()) return [text];

    const sortedTerms = [...terms].sort((a, b) => b.term.length - a.term.length);
    const regexTerms = sortedTerms.map(t => t.term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|');
    
    if (!regexTerms) return [text];

    const masterRegex = new RegExp(`(${regexTerms})`, 'gi');
    const elements: (string | React.ReactNode)[] = [];
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = masterRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
          elements.push(text.slice(lastIndex, match.index));
      }
      const foundTerm = terms.find(t => t.term.toLowerCase() === match![0].toLowerCase());
      elements.push(
          <GlossaryTooltip key={`${match.index}-${match[0]}`} text={match[0]} def={foundTerm?.def || ""} />
      );
      lastIndex = masterRegex.lastIndex;
    }
    if (lastIndex < text.length) elements.push(text.slice(lastIndex));
    return elements;
  };

  const processNode = (node: React.ReactNode): React.ReactNode => {
    if (!node || typeof node === 'boolean') return node;

    // 1. Si c'est du texte pur, on cherche uniquement les mots du glossaire
    if (typeof node === 'string') {
      return scanForGlossary(node);
    }

    // 2. Si c'est un tableau d'éléments, on boucle dessus
    if (Array.isArray(node)) {
      return React.Children.map(node, child => processNode(child));
    }

    // 3. Si c'est un composant React (comme nos équations venues de MDXRenderer)
    if (React.isValidElement(node)) {
      const nodeType = node.type as any;
      const props = node.props as any;
      const className = typeof props.className === 'string' ? props.className : '';

      // On protège les équations et les composants système pour ne pas y injecter de glossaire
      const isKatexClass = typeof className === 'string' && (
          className.includes('katex') || 
          className.includes('math') ||
          className.includes('mjx')
      );
      
      const isMathTag = typeof nodeType === 'string' && (nodeType === 'math' || nodeType === 'svg');
      
      // On détecte nos nouveaux composants mathématiques
      const isReactKatex = nodeType.name === 'InlineMath' || nodeType.name === 'BlockMath';
      
      const isExclusion = 
          nodeType.isNoGlossary === true || 
          nodeType.displayName === 'NoGlossary' || 
          nodeType.name === 'NoGlossary';

      // Si c'est une zone interdite, on la renvoie intacte
      if (isKatexClass || isMathTag || isExclusion || isReactKatex) {
          return node;
      }

      // Sinon, on plonge à l'intérieur du composant pour scanner ses enfants
      const { children } = props;
      if (!children) return node;

      return React.cloneElement(node as React.ReactElement<{ children: React.ReactNode }>, {
        children: processNode(children)
      });
    }

    return node;
  };

  return <>{processNode(children)}</>;
};