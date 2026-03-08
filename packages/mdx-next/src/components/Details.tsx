"use client";

import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';
import SmartText from './SmartText';
import styles from '../styles/Details.module.css';

const Details = ({ title, children }: { title: string, children: React.ReactNode }) => {
  const [isOpen, setIsOpen] = useState(false);
  
  return (
    <div className={styles.container}>
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className={styles.header}
        type="button"
      >
        <h4 className={styles.title}>{title}</h4>
        
        <ChevronDown 
          className={`${styles.icon} ${isOpen ? styles.rotate : ''}`} 
        />
      </button>
      
      {isOpen && (
        <div className={styles.content}>
          <SmartText>{children}</SmartText>
        </div>
      )}
    </div>
  );
}

export { Details };