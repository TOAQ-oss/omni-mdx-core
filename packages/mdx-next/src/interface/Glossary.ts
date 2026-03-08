interface GlossaryItem {
  term: string;
  def: string;
}

interface GlossaryProps {
  items: GlossaryItem[];
}

export type {
    GlossaryItem,
    GlossaryProps
}