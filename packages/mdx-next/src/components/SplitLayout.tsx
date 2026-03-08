import SmartText from './SmartText';
import { SplitLayoutProps, SplitRatio, VerticalAlign } from '../interface/SplitLayout';
import styles from '../styles/SplitLayout.module.css';

function SplitLayout({
  left,
  right,
  ratio = '50-50',
  align = 'top',
  reverse = false,
  stickyRight = false,
}: SplitLayoutProps) {
  
  const getRatioClasses = (r: SplitRatio) => {
    switch (r) {
      case '60-40': return { left: styles.ratio_60_40_left, right: styles.ratio_60_40_right };
      case '40-60': return { left: styles.ratio_40_60_left, right: styles.ratio_40_60_right };
      case '70-30': return { left: styles.ratio_70_30_left, right: styles.ratio_70_30_right };
      case '30-70': return { left: styles.ratio_30_70_left, right: styles.ratio_30_70_right };
      case '50-50':
      default:      return { left: styles.ratio_50_50_left, right: styles.ratio_50_50_right };
    }
  };

  const ratioStyle = getRatioClasses(ratio);

  const alignClasses: Record<VerticalAlign, string> = {
    top: styles.alignTop,
    center: styles.alignCenter,
    bottom: styles.alignBottom,
  };

  const alignmentClass = stickyRight ? styles.alignStretch : alignClasses[align];

  return (
    <div className={styles.wrapper}>
      <div 
        className={`
          ${styles.layout}
          ${alignmentClass}
          ${reverse ? styles.layoutReverse : ''}
        `}
      >
        <div className={`${styles.column} ${ratioStyle.left}`}>
          <SmartText>{left}</SmartText>
        </div>
        <div className={`${styles.column} ${ratioStyle.right}`}>
          <div className={`
            ${styles.stickyContent} 
            ${stickyRight ? styles.stickyActive : ''}
          `}>
            <SmartText>{right}</SmartText>
          </div>
        </div>

      </div>
    </div>
  );
}

export {
  SplitLayout
}