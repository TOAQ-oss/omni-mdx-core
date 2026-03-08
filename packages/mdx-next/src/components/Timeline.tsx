import SmartText from './SmartText';
import { TimelineItem } from '../interface/Timeline';
import styles from '../styles/Timeline.module.css';

function Timeline({ items }: { items: TimelineItem[] }) {
  return (
    <div className={styles.container}>
      <div className={styles.scrollContainer}>
        <div className={styles.innerContainer}>
          <div className={styles.backgroundLine} aria-hidden="true" />

          <div className={styles.itemsWrapper}>
            {items.map((item, index) => (
              <div key={index} className={styles.item}>
                <div 
                  className={`
                    ${styles.marker} 
                    ${item.active ? styles.markerActive : ""}
                  `}
                >
                  <div className={styles.innerDot} />
                </div>
                <div className={styles.content}>
                  {item.time && (
                    <div className={styles.timeWrapper}>
                       <span className={styles.timeBadge}>
                        {item.time}
                      </span>
                    </div>
                  )}
                  
                  <h4 className={styles.title}>
                    <SmartText>{item.title}</SmartText>
                  </h4>
                </div>

              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export {
  Timeline
}