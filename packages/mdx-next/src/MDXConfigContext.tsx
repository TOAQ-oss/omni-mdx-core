"use client";
import React, { createContext, useContext } from 'react';
import type { MDXConfig } from './types/MDXConfig';

/**
 * Configuration options for the MDX rendering pipeline.
 * Allows injecting custom React components and toggling specific parser features.
 */
const DefaultConfig: MDXConfig = {
  features: { math: true },
  components: {},
};

const MdxConfigContext = createContext<MDXConfig>(DefaultConfig);

/**
 * Context Provider that supplies the MDX configuration to the component tree.
 * * It safely merges the user-provided configuration with the default settings
 * to ensure no baseline features (like math parsing) are accidentally disabled.
 */
export const MdxConfigProvider = ({ 
  children, 
  config 
}: { 
  children: React.ReactNode; 
  config?: MDXConfig; 
}) => {
  const mergedConfig: MDXConfig = {
    ...DefaultConfig,
    ...config,
    features: {
      ...DefaultConfig.features,
      ...config?.features,
    },
    components: {
      ...DefaultConfig.components,
      ...config?.components,
    },
  };

  return (
    <MdxConfigContext.Provider value={mergedConfig}>
      {children}
    </MdxConfigContext.Provider>
  );
};

/**
 * Hook to access the current MDX configuration from within custom components.
 * * @returns The safely merged MDXConfig object.
 */
export const useMdxConfig = () => useContext(MdxConfigContext);