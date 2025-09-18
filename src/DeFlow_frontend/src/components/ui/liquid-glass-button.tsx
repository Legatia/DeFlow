import React from 'react'
import { cn } from '@/lib/utils'

export interface LiquidGlassButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'default' | 'primary' | 'secondary' | 'destructive' | 'workflow'
  size?: 'default' | 'sm' | 'lg' | 'icon'
  glowColor?: string
  children: React.ReactNode
}

const LiquidGlassButton = React.forwardRef<
  HTMLButtonElement,
  LiquidGlassButtonProps
>(({ className, variant = 'default', size = 'default', glowColor, children, ...props }, ref) => {
  const variants = {
    default: 'bg-white/10 border-white/20 text-white hover:bg-white/20',
    primary: 'bg-blue-500/20 border-blue-400/30 text-blue-100 hover:bg-blue-400/30 hover:shadow-blue-500/25',
    secondary: 'bg-gray-500/20 border-gray-400/30 text-gray-100 hover:bg-gray-400/30',
    destructive: 'bg-red-500/20 border-red-400/30 text-red-100 hover:bg-red-400/30 hover:shadow-red-500/25',
    workflow: 'bg-gradient-to-r from-purple-500/20 to-pink-500/20 border-purple-400/30 text-purple-100 hover:from-purple-400/30 hover:to-pink-400/30'
  }

  const sizes = {
    default: 'h-10 px-4 py-2',
    sm: 'h-9 rounded-md px-3',
    lg: 'h-11 rounded-md px-8',
    icon: 'h-10 w-10'
  }

  const glowStyle = glowColor 
    ? { boxShadow: `0 0 20px ${glowColor}33, 0 0 40px ${glowColor}22` }
    : {}

  return (
    <button
      className={cn(
        // Base glass morphism styles
        'relative inline-flex items-center justify-center',
        'rounded-lg border backdrop-blur-md',
        'font-medium transition-all duration-300 ease-out',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        'disabled:pointer-events-none disabled:opacity-50',
        // Clean glass style without reflection overlays
        // Variants and sizes
        variants[variant],
        sizes[size],
        className
      )}
      style={glowStyle}
      ref={ref}
      {...props}
    >
      <span className="relative z-10 flex items-center gap-2">
        {children}
      </span>
    </button>
  )
})

LiquidGlassButton.displayName = 'LiquidGlassButton'

export { LiquidGlassButton }