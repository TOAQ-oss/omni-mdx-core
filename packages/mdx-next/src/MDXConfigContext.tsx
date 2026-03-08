"use client";

import React, { createContext, useContext } from 'react';
import { ImageComponentType, MdxConfigState } from './interface/MDXEngine';


const DefaultImage: ImageComponentType = (props) => {
  return <img {...props} alt={props.alt || ''} />;
};

const MdxConfigContext = createContext<MdxConfigState>({
  Image: DefaultImage,
});

export const MdxConfigProvider = ({ 
  children, 
  ImageComponent 
}: { 
  children: React.ReactNode; 
  ImageComponent?: ImageComponentType; 
}) => {
  return (
    <MdxConfigContext.Provider value={{ Image: ImageComponent || DefaultImage }}>
      {children}
    </MdxConfigContext.Provider>
  );
};

export const useMdxConfig = () => useContext(MdxConfigContext);