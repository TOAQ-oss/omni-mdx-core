"use client";

import { Info, AlertTriangle, AlertCircle, CheckCircle2 } from 'lucide-react';
import SmartText from './SmartText';
import { NoteProps } from '../interface/Note';
import styles from '../styles/Notes.module.css';

const iconMap = {
  info: Info,
  warning: AlertTriangle,
  danger: AlertCircle,
  success: CheckCircle2,
};

const Note = ({ type = 'info', title, children }: NoteProps) => {
  const Icon = iconMap[type] || iconMap.info;
  const variantClass = styles[`type_${type}`] || styles.type_info;

  return (
    <div className={`${styles.container} ${variantClass}`}>
      <div className={styles.header}>
        <Icon 
          className={styles.icon} 
          strokeWidth={2.5} 
        />
        {title && (
          <h5 className={styles.title}>
            <SmartText>{title}</SmartText>
          </h5>
        )}
      </div>
      <div className={styles.content}>
         <SmartText>{children}</SmartText>
      </div>
    </div>
  );
}

export { Note };