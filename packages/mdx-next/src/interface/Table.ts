interface TableProps {
  title?: string;
  caption?: string;
  align?: string;
  headers: string[];
  data: (string | number | React.ReactNode)[][];
}

export type {
  TableProps
}