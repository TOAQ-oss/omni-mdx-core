"use client";

import React from 'react';
import dynamic from 'next/dynamic';

const SmartTextLogic = dynamic(() => import('./SmartTextLogic'), {
  ssr: true,
  loading: () => <React.Fragment />
});

const SmartText = (props: any) => {
  return <SmartTextLogic {...props} />;
};

export default SmartText;