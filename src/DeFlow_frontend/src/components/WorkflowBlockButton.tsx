import React from 'react'
import { LiquidGlassButton } from './ui/liquid-glass-button'
import { cn } from '@/lib/utils'

export interface WorkflowBlockButtonProps {
  type: 'defi' | 'social' | 'trigger' | 'action' | 'condition' | 'custom'
  title: string
  description?: string
  icon?: React.ReactNode
  isActive?: boolean
  isDragging?: boolean
  onDragStart?: (event: React.DragEvent) => void
  onDragEnd?: () => void
  onClick?: () => void
  className?: string
  style?: React.CSSProperties
}

const blockTypeStyles = {
  defi: {
    variant: 'primary' as const,
    glowColor: '#3b82f6',
    gradient: 'from-blue-500/20 to-cyan-500/20',
    borderColor: 'border-blue-400/30'
  },
  social: {
    variant: 'workflow' as const,
    glowColor: '#8b5cf6',
    gradient: 'from-purple-500/20 to-pink-500/20',
    borderColor: 'border-purple-400/30'
  },
  trigger: {
    variant: 'secondary' as const,
    glowColor: '#10b981',
    gradient: 'from-green-500/20 to-emerald-500/20',
    borderColor: 'border-green-400/30'
  },
  action: {
    variant: 'default' as const,
    glowColor: '#f59e0b',
    gradient: 'from-amber-500/20 to-orange-500/20',
    borderColor: 'border-amber-400/30'
  },
  condition: {
    variant: 'secondary' as const,
    glowColor: '#ef4444',
    gradient: 'from-red-500/20 to-rose-500/20',
    borderColor: 'border-red-400/30'
  },
  custom: {
    variant: 'default' as const,
    glowColor: '#6b7280',
    gradient: 'from-gray-500/20 to-slate-500/20',
    borderColor: 'border-gray-400/30'
  }
}

export const WorkflowBlockButton: React.FC<WorkflowBlockButtonProps> = ({
  type,
  title,
  description,
  icon,
  isActive = false,
  isDragging = false,
  onDragStart,
  onDragEnd,
  onClick,
  className,
  style
}) => {
  const blockStyle = blockTypeStyles[type]

  return (
    <div
      className={cn(
        'group relative w-full transition-all duration-300',
        isDragging && 'rotate-2 scale-105 z-50',
        className
      )}
      style={style}
      draggable
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
    >
      <LiquidGlassButton
        variant={blockStyle.variant}
        glowColor={isActive ? blockStyle.glowColor : undefined}
        className={cn(
          'w-full h-auto p-4 text-left',
          'bg-gradient-to-br',
          blockStyle.gradient,
          blockStyle.borderColor,
          isActive && 'ring-2 ring-current',
          'hover:scale-[1.02] active:scale-[0.98]',
          'group-hover:shadow-lg'
        )}
        onClick={onClick}
      >
        <div className="flex items-start gap-3">
          {icon && (
            <div className={cn(
              'flex-shrink-0 w-8 h-8 rounded-lg',
              'bg-white/10 backdrop-blur-sm',
              'flex items-center justify-center',
              'text-current'
            )}>
              {icon}
            </div>
          )}
          
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-sm mb-1 truncate">
              {title}
            </h3>
            {description && (
              <p className="text-xs opacity-80 line-clamp-2">
                {description}
              </p>
            )}
          </div>

          {/* Drag indicator */}
          <div className="flex-shrink-0 opacity-40 group-hover:opacity-60 transition-opacity">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
            </svg>
          </div>
        </div>
      </LiquidGlassButton>

      {/* Active indicator */}
      {isActive && (
        <div className="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-current animate-pulse" />
      )}
    </div>
  )
}