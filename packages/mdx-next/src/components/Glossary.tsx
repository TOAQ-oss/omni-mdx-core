"use client";

import { useState, useEffect } from 'react';
import { useParams } from 'next/navigation';
import { ChevronDown } from 'lucide-react';
import { useGlossary } from '../context/GlossaryContent';
import { GlossaryProps } from '../interface/Glossary';
import styles from '../styles/Glossary.module.css';

function Glossary({ items }: GlossaryProps) {
  const [isOpen, setIsOpen] = useState(false);
  const { registerTerms } = useGlossary();

  useEffect(() => {
    registerTerms(items);
  }, [items, registerTerms]);

  const params = useParams();
  const lang = params?.lang as string;
  
  const titles: Record<string, string> = {
    fr: "Glossaire",
    en: "Glossary",
    de: "Glossar"
  };

  const currentTitle = titles[lang?.split('-')[0]] || titles.en;

  return (
    <div className={styles.container}>
      
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className={styles.header}
        type="button"
      >
        <span>
          <h4 className={styles.title}>{currentTitle || "Glossary"}</h4>
        </span>

        <div className={`${styles.iconWrapper} ${isOpen ? styles.rotate : ''}`}>
           <ChevronDown className={styles.icon} />
        </div>
      </button>

      {isOpen && (
        <div className={styles.contentWrapper}>
          <div className={styles.list}>
            {items.map((item, index) => (
              <div key={index} className={styles.item}>
                <div className={styles.termCol}>
                  <span className={styles.term} id={`term-${item.term}`}>
                    {item.term}
                  </span>
                </div>
                <div className={styles.definitionCol}>
                  {item.def}
                </div>
              </div>

            ))}
          </div>
          <div className={styles.spacer}></div>
        </div>
      )}
    </div>
  );
}

export {
  Glossary
}