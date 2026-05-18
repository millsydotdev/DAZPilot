import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * cn - Conditional classnames utility
 * Merges Tailwind classes with clsx for conditional classes
 * and tailwind-merge to handle conflicting Tailwind classes
 *
 * Usage:
 * cn('text-red-500', isActive && 'font-bold')
 * cn('px-4 py-2', size === 'sm' && 'text-sm')
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/**
 * Composable class builder for more complex scenarios
 */
export function createClassBuilder(baseClass?: string) {
  const classes: string[] = baseClass ? [baseClass] : [];

  return {
    add: (...newClasses: (string | undefined | null | false)[]) => {
      classes.push(...(newClasses.filter(Boolean) as string[]));
      return builder;
    },
    conditional: (condition: boolean, ...newClasses: string[]) => {
      if (condition) {
        classes.push(...newClasses);
      }
      return builder;
    },
    build: () => cn(...classes),
  };
}

const builder = createClassBuilder();
export { builder };
