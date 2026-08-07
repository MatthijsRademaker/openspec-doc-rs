import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export { default as Badge } from './Badge.vue'

const stateBase = 'border bg-transparent'

export const badgeVariants = cva(
  'min-h-5 gap-1 rounded-[var(--radius-sm)] border border-transparent px-2 py-0.5 font-mono text-xs font-medium transition-colors [transition-duration:var(--motion-duration)] [&>svg]:size-3! group/badge inline-flex w-fit shrink-0 items-center justify-center overflow-hidden whitespace-nowrap focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 aria-invalid:border-destructive aria-invalid:ring-destructive/20 [&>svg]:pointer-events-none',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground [a]:hover:bg-accent',
        instrument: 'border-border bg-transparent text-foreground [a]:hover:border-accent',
        secondary: 'bg-secondary text-secondary-foreground [a]:hover:bg-muted',
        destructive:
          'border-destructive/50 bg-transparent text-destructive [a]:hover:bg-destructive/10',
        outline: 'border-border bg-transparent text-foreground [a]:hover:border-accent',
        ghost: 'text-foreground hover:bg-muted hover:text-muted-foreground',
        link: 'text-foreground underline-offset-4 hover:text-accent hover:underline',
        open: `${stateBase} border-open/60 text-open`,
        addressed: `${stateBase} border-addressed/60 text-addressed`,
        resolved: `${stateBase} border-resolved/60 text-resolved`,
        verdict: `${stateBase} border-verdict/60 text-verdict`,
        delivery: `${stateBase} border-delivery/60 text-delivery`,
        reviewer: `${stateBase} border-reviewer/60 text-reviewer`,
        agent: `${stateBase} border-agent/60 text-agent`,
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)
export type BadgeVariants = VariantProps<typeof badgeVariants>
