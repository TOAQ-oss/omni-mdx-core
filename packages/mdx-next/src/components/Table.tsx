import SmartText from './SmartText';
import { TableProps } from '../interface/Table';
import styles from '../styles/Table.module.css';

const alignClasses = {
  left: styles['toaq-align-left'],
  center: styles['toaq-align-center'],
  right: styles['toaq-align-right']
};

function Table({
  caption,
  headers,
  data,
  align = 'left',
}: TableProps) {

  const alignmentClass = alignClasses[align as keyof typeof alignClasses] || alignClasses.left;

  return (
    <div className={`${styles['toaq-table-wrapper']} counter-table`}>
      <div className={`${styles['toaq-table-scroll']}`}>
        <table className={`${styles['toaq-table']}`}>
          <thead>
            <tr>
              {headers.map((header, index) => (
                <th key={index} className={alignmentClass}>
                  <SmartText>{header}</SmartText>
                </th>
              ))}
            </tr>
          </thead>

          <tbody>
            {data.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {row.map((cell, cellIndex) => (
                  <td key={cellIndex} className={alignmentClass}>
                    <SmartText>{cell}</SmartText>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {caption && (
        <div className={`${styles['toaq-caption-container']}`}>
          <p className={`${styles['toaq-caption-text']} caption-text`}>
            <SmartText>{caption}</SmartText>
          </p>
        </div>
      )}
    </div>
  );
}

export {
  Table
}