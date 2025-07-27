import type { Directive, DirectiveBinding } from 'vue'
import { useWindowRegistry } from '../composables/useWindowRegistry'

// Type for directive binding value
interface WindowRegistryDirectiveOptions {
  id: string
  closeOnClickOutside?: boolean
  isModal?: boolean
  priority?: number
  closeHandler?: () => void
  zIndex?: number
}

// Global registry instance for directive
const registry = useWindowRegistry({ debugMode: false })

// Map to track elements registered by directive
const directiveRegistrations = new WeakMap<HTMLElement, string>()

export const windowRegistryDirective: Directive<HTMLElement, WindowRegistryDirectiveOptions> = {
  mounted(el: HTMLElement, binding: DirectiveBinding<WindowRegistryDirectiveOptions>) {
    const options = binding.value
    
    if (!options?.id) {
      console.warn('[WindowRegistry Directive] Missing required "id" option')
      return
    }

    // Register the element
    registry.registerWindow(options.id, el, {
      closeOnClickOutside: options.closeOnClickOutside,
      isModal: options.isModal,
      priority: options.priority,
      closeHandler: options.closeHandler,
      zIndex: options.zIndex
    })

    // Track this registration
    directiveRegistrations.set(el, options.id)
  },

  updated(el: HTMLElement, binding: DirectiveBinding<WindowRegistryDirectiveOptions>) {
    const options = binding.value
    const oldOptions = binding.oldValue
    
    if (!options?.id) {
      console.warn('[WindowRegistry Directive] Missing required "id" option')
      return
    }

    // If ID changed, unregister old and register new
    if (oldOptions?.id && oldOptions.id !== options.id) {
      registry.unregisterWindow(oldOptions.id)
      registry.registerWindow(options.id, el, {
        closeOnClickOutside: options.closeOnClickOutside,
        isModal: options.isModal,
        priority: options.priority,
        closeHandler: options.closeHandler,
        zIndex: options.zIndex
      })
      directiveRegistrations.set(el, options.id)
    } else {
      // Update existing registration
      registry.updateWindow(options.id, {
        closeOnClickOutside: options.closeOnClickOutside,
        isModal: options.isModal,
        priority: options.priority,
        closeHandler: options.closeHandler,
        zIndex: options.zIndex
      })
    }
  },

  beforeUnmount(el: HTMLElement) {
    const windowId = directiveRegistrations.get(el)
    if (windowId) {
      registry.unregisterWindow(windowId)
      directiveRegistrations.delete(el)
    }
  }
}

// Export for use in main.ts or individual components
export default windowRegistryDirective