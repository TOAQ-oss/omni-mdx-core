import { describe, it, expect } from 'vitest';
import * as client from '../../src/client';
import * as server from '../../src/server';
import * as root from '../../src/index';

describe('Entry Points & Exports', () => {
  it('server entry point exports correct members and executes logic', () => {
    expect(server.parseMdx).toBeDefined();
    expect(server.MDXServerRenderer).toBeDefined();
    expect(server.SERVER_READY).toBe(true);

    const ast = server.parseMdxSync('# Trigger Coverage');
    expect(ast[0].node_type).toBe('h1');
  });

  it('client entry point exports expected components', () => {
    expect(client.MDXClientRenderer).toBeDefined();
    expect(client.parseMdx).toBeDefined();
  });

  it('root entry point resolves attributes correctly', () => {
    expect(root.MDX_COMPONENTS).toBeDefined();

    const stringifiedProps = '{ title: "Hello", active: true, count: 42 }';
    const props = root.parseProps(stringifiedProps) as any;

    expect(props.title).toBe('Hello');
    expect(props.active).toBe(true);
    expect(props.count).toBe(42);
  });
});