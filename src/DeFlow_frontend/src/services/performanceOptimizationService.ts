/**
 * 🚀 Performance Optimization Service for DeFlow Frontend
 * Handles memory management, caching, and performance monitoring
 * Prevents memory leaks and optimizes React component performance
 */

interface PerformanceMetrics {
  memoryUsage: number
  componentRenderCount: number
  apiCallCount: number
  cacheHitRate: number
  lastCleanup: number
}

interface CacheEntry<T> {
  data: T
  timestamp: number
  accessCount: number
  ttl: number
}

class PerformanceOptimizationService {
  private cache = new Map<string, CacheEntry<any>>()
  private renderCounters = new Map<string, number>()
  private metrics: PerformanceMetrics = {
    memoryUsage: 0,
    componentRenderCount: 0,
    apiCallCount: 0,
    cacheHitRate: 0,
    lastCleanup: 0
  }
  private cleanupInterval: NodeJS.Timeout | null = null
  private observers = new Set<MutationObserver>()
  private eventListeners = new Set<{ element: EventTarget; event: string; handler: EventListener }>()

  constructor() {
    this.startPerformanceMonitoring()
    this.scheduleCleanup()
  }

  // PERFORMANCE: Memory-efficient caching with TTL
  setCache<T>(key: string, data: T, ttlMs: number = 300000): void { // 5 min default TTL
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      accessCount: 0,
      ttl: ttlMs
    })
  }

  getCache<T>(key: string): T | null {
    const entry = this.cache.get(key)
    if (!entry) return null

    const now = Date.now()
    if (now - entry.timestamp > entry.ttl) {
      this.cache.delete(key)
      return null
    }

    entry.accessCount++
    this.updateCacheHitRate()
    return entry.data
  }

  // PERFORMANCE: Component render tracking for debugging
  trackComponentRender(componentName: string): void {
    const count = this.renderCounters.get(componentName) || 0
    this.renderCounters.set(componentName, count + 1)
    this.metrics.componentRenderCount++
  }

  // PERFORMANCE: API call tracking for rate limiting
  trackApiCall(): void {
    this.metrics.apiCallCount++
  }

  // PERFORMANCE: Memory leak prevention
  addEventListenerSafely(
    element: EventTarget,
    event: string,
    handler: EventListener,
    options?: AddEventListenerOptions
  ): () => void {
    element.addEventListener(event, handler, options)
    const listenerInfo = { element, event, handler }
    this.eventListeners.add(listenerInfo)

    // Return cleanup function
    return () => {
      element.removeEventListener(event, handler)
      this.eventListeners.delete(listenerInfo)
    }
  }

  // PERFORMANCE: Safe observer management
  addObserverSafely(observer: MutationObserver): () => void {
    this.observers.add(observer)
    
    return () => {
      observer.disconnect()
      this.observers.delete(observer)
    }
  }

  // PERFORMANCE: Debounced function creator
  createDebouncedFunction<T extends (...args: any[]) => any>(
    func: T,
    delayMs: number
  ): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout

    return (...args: Parameters<T>) => {
      clearTimeout(timeoutId)
      timeoutId = setTimeout(() => func(...args), delayMs)
    }
  }

  // PERFORMANCE: Throttled function creator
  createThrottledFunction<T extends (...args: any[]) => any>(
    func: T,
    intervalMs: number
  ): (...args: Parameters<T>) => void {
    let lastCallTime = 0

    return (...args: Parameters<T>) => {
      const now = Date.now()
      if (now - lastCallTime >= intervalMs) {
        lastCallTime = now
        func(...args)
      }
    }
  }

  // PERFORMANCE: Image lazy loading helper
  createLazyImageObserver(
    onIntersect: (entry: IntersectionObserverEntry) => void
  ): IntersectionObserver {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          onIntersect(entry)
          observer.unobserve(entry.target)
        }
      })
    }, { threshold: 0.1 })

    return observer
  }

  // PERFORMANCE: Component unmount cleanup helper
  createCleanupManager(): {
    addCleanup: (cleanup: () => void) => void
    cleanup: () => void
  } {
    const cleanupFunctions: Array<() => void> = []

    return {
      addCleanup: (cleanup: () => void) => {
        cleanupFunctions.push(cleanup)
      },
      cleanup: () => {
        cleanupFunctions.forEach(fn => {
          try {
            fn()
          } catch (error) {
            console.error('Cleanup function failed:', error)
          }
        })
        cleanupFunctions.length = 0
      }
    }
  }

  // PERFORMANCE: Monitor performance metrics
  private startPerformanceMonitoring(): void {
    if (typeof window !== 'undefined' && 'performance' in window) {
      // Monitor memory usage if available
      if ('memory' in performance) {
        setInterval(() => {
          const memInfo = (performance as any).memory
          this.metrics.memoryUsage = memInfo.usedJSHeapSize / 1024 / 1024 // MB
        }, 5000)
      }
    }
  }

  // PERFORMANCE: Cache hit rate calculation
  private updateCacheHitRate(): void {
    let totalAccess = 0
    let totalHits = 0

    this.cache.forEach(entry => {
      totalAccess += entry.accessCount
      totalHits += Math.min(entry.accessCount, 1)
    })

    this.metrics.cacheHitRate = totalAccess > 0 ? totalHits / totalAccess : 0
  }

  // PERFORMANCE: Scheduled cleanup to prevent memory leaks
  private scheduleCleanup(): void {
    this.cleanupInterval = setInterval(() => {
      this.performCleanup()
    }, 300000) // Cleanup every 5 minutes
  }

  // PERFORMANCE: Comprehensive cleanup
  performCleanup(): void {
    const now = Date.now()
    
    // Clear expired cache entries
    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.delete(key)
      }
    }

    // Reset render counters if they get too high
    for (const [component, count] of this.renderCounters.entries()) {
      if (count > 10000) {
        this.renderCounters.set(component, Math.floor(count / 10))
      }
    }

    this.metrics.lastCleanup = now
    console.log('🧹 Performance cleanup completed:', {
      cacheSize: this.cache.size,
      memoryUsage: `${this.metrics.memoryUsage.toFixed(2)} MB`,
      renderCount: this.metrics.componentRenderCount,
      cacheHitRate: `${(this.metrics.cacheHitRate * 100).toFixed(1)}%`
    })
  }

  // PERFORMANCE: Get current metrics
  getMetrics(): PerformanceMetrics {
    return { ...this.metrics }
  }

  // PERFORMANCE: Get render statistics
  getRenderStats(): Record<string, number> {
    return Object.fromEntries(this.renderCounters)
  }

  // PERFORMANCE: Force cleanup of all resources
  cleanup(): void {
    // Clear cache
    this.cache.clear()
    
    // Remove all event listeners
    this.eventListeners.forEach(({ element, event, handler }) => {
      element.removeEventListener(event, handler)
    })
    this.eventListeners.clear()

    // Disconnect all observers
    this.observers.forEach(observer => {
      observer.disconnect()
    })
    this.observers.clear()

    // Clear intervals
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval)
      this.cleanupInterval = null
    }

    console.log('🧹 Performance service fully cleaned up')
  }

  // PERFORMANCE: React Hook for component performance tracking
  usePerformanceTracking(componentName: string): {
    trackRender: () => void
    getComponentRenderCount: () => number
  } {
    return {
      trackRender: () => this.trackComponentRender(componentName),
      getComponentRenderCount: () => this.renderCounters.get(componentName) || 0
    }
  }

  // PERFORMANCE: Batch DOM operations
  batchDomOperations(operations: Array<() => void>): void {
    requestAnimationFrame(() => {
      operations.forEach(operation => {
        try {
          operation()
        } catch (error) {
          console.error('DOM operation failed:', error)
        }
      })
    })
  }

  // PERFORMANCE: Virtual scrolling helper
  calculateVisibleItems(
    containerHeight: number,
    itemHeight: number,
    scrollTop: number,
    totalItems: number,
    overscan: number = 3
  ): { startIndex: number; endIndex: number; visibleItems: number } {
    const visibleItems = Math.ceil(containerHeight / itemHeight)
    const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan)
    const endIndex = Math.min(totalItems - 1, startIndex + visibleItems + overscan * 2)

    return {
      startIndex,
      endIndex,
      visibleItems: endIndex - startIndex + 1
    }
  }
}

// Export singleton instance
export const performanceService = new PerformanceOptimizationService()
export default performanceService

// Export types
export type { PerformanceMetrics }