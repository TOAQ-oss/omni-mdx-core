import { describe, it, expect } from 'vitest';
import * as client from '../../src/client';
import * as server from '../../src/server';

describe('Server Entry Point', () => {
  it('should trigger coverage for server.ts by accessing exports', () => {
    expect(typeof server.parseMdx).toBe('function');
    expect(typeof server.MDXServerRenderer).toBe('function');
    
    // On vérifie que les exports sont bien présents
    const exportedKeys = Object.keys(server);
    expect(exportedKeys).toContain('parseMdx');
    expect(exportedKeys).toContain('MDXServerRenderer');
  });
});

describe('Server Entry Point Validation', () => {
  it('should trigger coverage by executing logic through the entry point', () => {
    // 1. Accès direct à une constante (si tu en as une)
    expect(server).toBeDefined();
    expect(server.SERVER_READY).toBe(true);
    // 2. APPEL RÉEL : C'est ce qui va "allumer" le fichier dans le rapport
    // On appelle parseMdxSync via l'objet server
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