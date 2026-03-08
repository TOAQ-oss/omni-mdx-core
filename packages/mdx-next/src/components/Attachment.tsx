"use client";

import styles from '../styles/Attachment.module.css';
import SmartText from './SmartText';
import { useMdxConfig } from '../MDXConfigContext';

const FullImage = ({ src, alt, caption }: { src: string, alt: string, caption?: string }) => {
  const { Image } = useMdxConfig();

  return (
    <figure className={`${styles.figure} counter-figure`}>
      <div className={styles.singleWrapper}>
        <Image
          src={src}
          alt={alt}
          width={0}
          height={0}
          sizes="100vw"
          className={styles.image}
          style={{ width: '100%', height: 'auto' }} 
        />
      </div>
      
      {caption && (
        <figcaption className={styles.caption}>
            <SmartText>{caption}</SmartText>
        </figcaption>
      )}
    </figure>
  )
};

const DualImages = ({ src1, src2, caption }: { src1: string, src2: string, caption?: string }) => {
  const { Image } = useMdxConfig();

  return (
    <figure className={`${styles.figure} counter-figure`}>
      <div className={styles.dualWrapper}>
        <div className={styles.dualItem}>
          <Image
            src={src1}
            alt="Figure A"
            width={0}
            height={0}
            sizes="(max-width: 768px) 100vw, 50vw"
            className={styles.image}
            style={{ width: '100%', height: 'auto' }}
          />
        </div>
        <div className={styles.dualItem}>
          <Image
            src={src2}
            alt="Figure B"
            width={0}
            height={0}
            sizes="(max-width: 768px) 100vw, 50vw"
            className={styles.image}
            style={{ width: '100%', height: 'auto' }}
          />
        </div>
      </div>
      {caption && (
        <figcaption className={styles.caption}>
            <SmartText>{caption}</SmartText>
        </figcaption>
      )}
    </figure>
  )
};

export {
  FullImage,
  DualImages
}