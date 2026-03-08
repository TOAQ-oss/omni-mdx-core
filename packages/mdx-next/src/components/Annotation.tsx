import SmartText from './SmartText';
import { AnnotationProps } from '../interface/Annotation';
import styles from '../styles/Annotation.module.css';

const AnnotationRef = ({ index }: { index: number }) => {
  return (
    <a 
      href={`#annotation-${index}`}
      id={`annotationref-${index}`}
      className={styles.ref_link}
      aria-label={`Voir la note ${index}`}
    >
      [{index}]
    </a>
  );
};

const Annotation = ({ index, children, className = '' }: AnnotationProps) => {
  return (
    <div 
      id={`annotation-${index}`}
      className={`${styles.annotation_container} ${className}`}
    >
      <a 
        href={`#annotationref-${index}`} 
        className={styles.note_index}
        title="Remonter au texte"
      >
        {index}.
      </a>
      <div>
        <SmartText>{children}</SmartText>
      </div>
    </div>
  );
};

export {
  AnnotationRef,
  Annotation
}