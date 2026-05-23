import { describe, it, expect, beforeEach } from 'vitest';
import { cn, createClassBuilder, builder, resetBuilder } from './cn';

beforeEach(() => {
  resetBuilder();
});

describe('cn', () => {
  it('joins single class', () => {
    expect(cn('text-red-500')).toBe('text-red-500');
  });

  it('joins multiple classes', () => {
    expect(cn('px-4', 'py-2', 'font-bold')).toBe('px-4 py-2 font-bold');
  });

  it('handles conditional classes (truthy)', () => {
    expect(cn('base', true && 'extra')).toBe('base extra');
  });

  it('handles conditional classes (falsy)', () => {
    expect(cn('base', false && 'extra')).toBe('base');
  });

  it('merges conflicting Tailwind classes (last wins)', () => {
    expect(cn('px-4', 'px-8')).toBe('px-8');
  });
});

describe('createClassBuilder', () => {
  it('builds from base class', () => {
    const b = createClassBuilder('btn');
    expect(b.build()).toBe('btn');
  });

  it('adds classes', () => {
    const b = createClassBuilder();
    b.add('a', 'b');
    expect(b.build()).toBe('a b');
  });

  it('add filters falsy values', () => {
    const b = createClassBuilder();
    b.add('a', undefined, null, false, 'b');
    expect(b.build()).toBe('a b');
  });

  it('conditionally adds classes', () => {
    const b = createClassBuilder('base');
    b.conditional(true, 'visible');
    b.conditional(false, 'hidden');
    expect(b.build()).toBe('base visible');
  });

  it('supports chaining', () => {
    const result = createClassBuilder('btn')
      .add('rounded')
      .conditional(true, 'primary')
      .conditional(false, 'disabled')
      .build();
    expect(result).toBe('btn rounded primary');
  });

  it('merges Tailwind classes in build', () => {
    const result = createClassBuilder('p-4').add('p-8').build();
    expect(result).toBe('p-8');
  });
});

describe('builder singleton', () => {
  it('is a createClassBuilder instance', () => {
    expect(builder.build()).toBe('');
  });

  it('can be chained', () => {
    builder.add('shared');
    expect(builder.build()).toBe('shared');
  });
});
