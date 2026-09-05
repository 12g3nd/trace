import { describe, expect, it } from 'vitest';
import component from './OrbitSidecar.svelte?raw';

describe('Orbit Sidecar active launcher treatment', () => {
  it('uses a restrained halo without a status dot or inset outline', () => {
    expect(component).not.toContain('.launcher-btn.running::after');
    expect(component).toContain('background: rgba(42, 148, 226, 0.22);');
    expect(component).toContain('box-shadow: 0 0 8px rgba(74, 191, 255, 0.34);');
    expect(component).not.toContain('inset 0 0 0 1px');
  });
});
