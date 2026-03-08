"use client";

import styles from '../styles/ChartSkeleton.module.css';

const ChartSkeleton = () => {
  return (
    <div className={styles.wrapper} role="status" aria-label="Loading chart data">
      <div className={styles.shimmerWrapper} />
      <div className={styles.header}>
        <div className={styles.bone + ' ' + styles.titleBone} />
        <div className={styles.bone + ' ' + styles.legendBone} />
      </div>
      <div className={styles.chartArea}>
        <div className={styles.yAxis}>
          <div className={styles.bone + ' ' + styles.tickBone} />
          <div className={styles.bone + ' ' + styles.tickBone} />
          <div className={styles.bone + ' ' + styles.tickBone} />
          <div className={styles.bone + ' ' + styles.tickBone} />
          <div className={styles.bone + ' ' + styles.tickBone} />
        </div>
        <div className={styles.gridArea} />
      </div>
      <div className={styles.xAxis}>
        <div className={styles.bone + ' ' + styles.xTickBone} />
        <div className={styles.bone + ' ' + styles.xTickBone} />
        <div className={styles.bone + ' ' + styles.xTickBone} />
        <div className={styles.bone + ' ' + styles.xTickBone} />
        <div className={styles.bone + ' ' + styles.xTickBone} />
      </div>
    </div>
  );
};

export {
  ChartSkeleton
}