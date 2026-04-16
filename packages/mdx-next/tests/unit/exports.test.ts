import { describe, it, expect } from 'vitest';
import * as client from '../../src/client';
import * as server from '../../src/server';

describe('Server Entry Point', () => {
  it('should trigger coverage for server.ts by accessing exports', () => {
    expect(typeof server.parseMdx).toBe('function');
    expect(typeof server.MDXServerRenderer).toBe('function');
    
    const exportedKeys = Object.keys(server);
    expect(exportedKeys).toContain('parseMdx');
    expect(exportedKeys).toContain('MDXServerRenderer');
  });

  it('should trigger coverage by executing logic through the entry point', () => {
    expect(server).toBeDefined();
    expect(server.SERVER_READY).toBe(true);

    const ast = server.parseMdxSync('# Trigger Coverage');
    
    expect(ast[0].node_type).toBe('h1');
    expect(server.MDXServerRenderer).toBeDefined();
  });
});

describe('Client Entry Points', () => {
  it('client entry point exports expected components', () => {
    expect(client.MDXClientRenderer).toBeDefined();
    expect(client.parseMdx).toBeDefined();
  });
});