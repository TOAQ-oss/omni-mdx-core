interface ImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  src: string;
  alt: string;
  width?: number;
  height?: number;
  className?: string;
  priority?: boolean;
}

type ImageComponentType = React.ComponentType<ImageProps>;

interface MdxConfigState {
  Image: ImageComponentType;
}

export interface MDXErrorDetailsType {
    message: string;
    line?: number;
    column?: number;
    index?: number;
}

interface MDXViewerProps {
  content: string;
  imageComponent?: ImageComponentType;
  onErrorChange?: (error: MDXErrorDetailsType | null) => void;
}

export type {
  ImageComponentType,
  MdxConfigState,
  MDXViewerProps
}
