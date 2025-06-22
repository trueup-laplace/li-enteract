<script setup lang="ts">
import { computed } from 'vue'
import { useTransparency } from '../../composables/useTransparency'

const transparency = useTransparency()

// Calculate border intensity based on transparency level
const borderIntensity = computed(() => {
  if (!transparency.isTransparent.value) return 0
  
  // More transparent = more prominent border
  const transparencyAmount = 1 - transparency.transparencyLevel.value
  return Math.min(transparencyAmount * 1.5, 1) // Cap at 1.0
})

// Border style based on transparency state
const borderStyle = computed(() => {
  const intensity = borderIntensity.value
  
  if (intensity === 0) return { display: 'none' }
  
  return {
    '--border-intensity': intensity,
    '--glow-intensity': intensity * 0.8,
    '--shimmer-speed': `${2 - intensity}s`, // Faster shimmer when more transparent
    opacity: intensity
  }
})

// Dynamic border classes
const borderClasses = computed(() => {
  const classes = ['refraction-border']
  
  if (transparency.isClickThrough.value) {
    classes.push('click-through-mode')
  } else if (transparency.transparencyLevel.value < 0.3) {
    classes.push('high-transparency')
  } else if (transparency.transparencyLevel.value < 0.7) {
    classes.push('medium-transparency')
  }
  
  return classes.join(' ')
})
</script>

<template>
  <!-- Refraction Border Container -->
  <div 
    v-if="transparency.isTransparent.value"
    :class="borderClasses"
    :style="borderStyle"
  >
    <!-- Main Border Effect -->
    <div class="border-main"></div>
    
    <!-- Animated Shimmer Effect -->
    <div class="border-shimmer"></div>
    
    <!-- Corner Accents -->
    <div class="corner-accent top-left"></div>
    <div class="corner-accent top-right"></div>
    <div class="corner-accent bottom-left"></div>
    <div class="corner-accent bottom-right"></div>
    
    <!-- Edge Highlights -->
    <div class="edge-highlight top"></div>
    <div class="edge-highlight right"></div>
    <div class="edge-highlight bottom"></div>
    <div class="edge-highlight left"></div>
    
    <!-- Pulsing Glow for Click-Through Mode -->
    <div v-if="transparency.isClickThrough.value" class="click-through-pulse"></div>
  </div>
</template>

<style scoped>
.refraction-border {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 99999;
  border-radius: 12px;
}

/* Main Border Effect */
.border-main {
  position: absolute;
  inset: 0;
  border-radius: 12px;
  border: 2px solid transparent;
  background: linear-gradient(45deg, 
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.3)),
    rgba(0, 255, 255, calc(var(--border-intensity) * 0.2)),
    rgba(255, 0, 255, calc(var(--border-intensity) * 0.15)),
    rgba(255, 255, 0, calc(var(--border-intensity) * 0.2)),
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.3))
  ) border-box;
  mask: linear-gradient(white 0 0) padding-box, linear-gradient(white 0 0);
  mask-composite: subtract;
  -webkit-mask: linear-gradient(white 0 0) padding-box, linear-gradient(white 0 0);
  -webkit-mask-composite: source-out;
  
  /* Refraction effect */
  backdrop-filter: 
    blur(1px) 
    brightness(1.1) 
    saturate(1.2) 
    hue-rotate(10deg);
  
  /* Subtle glow */
  box-shadow: 
    0 0 20px rgba(255, 255, 255, calc(var(--glow-intensity) * 0.3)),
    inset 0 0 20px rgba(255, 255, 255, calc(var(--glow-intensity) * 0.1));
}

/* Animated Shimmer Effect */
.border-shimmer {
  position: absolute;
  inset: -2px;
  border-radius: 14px;
  background: linear-gradient(
    45deg,
    transparent 30%, 
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.6)) 50%, 
    transparent 70%
  );
  background-size: 200% 200%;
  animation: shimmer var(--shimmer-speed) ease-in-out infinite;
  mask: linear-gradient(white 0 0) padding-box, linear-gradient(white 0 0);
  mask-composite: subtract;
  -webkit-mask: linear-gradient(white 0 0) padding-box, linear-gradient(white 0 0);
  -webkit-mask-composite: source-out;
}

@keyframes shimmer {
  0% { background-position: -200% -200%; }
  50% { background-position: 200% 200%; }
  100% { background-position: -200% -200%; }
}

/* Corner Accents */
.corner-accent {
  position: absolute;
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, calc(var(--border-intensity) * 0.8));
  background: radial-gradient(
    circle at center,
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.3)) 0%,
    transparent 70%
  );
  backdrop-filter: blur(2px) brightness(1.3);
}

.corner-accent.top-left {
  top: -2px;
  left: -2px;
  border-right: none;
  border-bottom: none;
  border-radius: 12px 0 0 0;
}

.corner-accent.top-right {
  top: -2px;
  right: -2px;
  border-left: none;
  border-bottom: none;
  border-radius: 0 12px 0 0;
}

.corner-accent.bottom-left {
  bottom: -2px;
  left: -2px;
  border-right: none;
  border-top: none;
  border-radius: 0 0 0 12px;
}

.corner-accent.bottom-right {
  bottom: -2px;
  right: -2px;
  border-left: none;
  border-top: none;
  border-radius: 0 0 12px 0;
}

/* Edge Highlights */
.edge-highlight {
  position: absolute;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.4)) 50%,
    transparent 100%
  );
  backdrop-filter: blur(1px) brightness(1.2);
}

.edge-highlight.top,
.edge-highlight.bottom {
  left: 20px;
  right: 20px;
  height: 1px;
}

.edge-highlight.top {
  top: 0;
}

.edge-highlight.bottom {
  bottom: 0;
}

.edge-highlight.left,
.edge-highlight.right {
  top: 20px;
  bottom: 20px;
  width: 1px;
  background: linear-gradient(
    0deg,
    transparent 0%,
    rgba(255, 255, 255, calc(var(--border-intensity) * 0.4)) 50%,
    transparent 100%
  );
}

.edge-highlight.left {
  left: 0;
}

.edge-highlight.right {
  right: 0;
}

/* Click-Through Mode Pulse */
.click-through-pulse {
  position: absolute;
  inset: -4px;
  border-radius: 16px;
  border: 2px solid rgba(255, 165, 0, 0.6);
  background: rgba(255, 165, 0, 0.1);
  animation: clickThroughPulse 2s ease-in-out infinite;
}

@keyframes clickThroughPulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.6;
  }
  50% {
    transform: scale(1.02);
    opacity: 1;
  }
}

/* Enhanced effects for different transparency levels */
.high-transparency .border-main {
  border-width: 3px;
  backdrop-filter: 
    blur(2px) 
    brightness(1.2) 
    saturate(1.4) 
    hue-rotate(20deg);
}

.medium-transparency .border-main {
  backdrop-filter: 
    blur(1.5px) 
    brightness(1.15) 
    saturate(1.3) 
    hue-rotate(15deg);
}

.click-through-mode .border-main {
  border-color: rgba(255, 165, 0, 0.8);
  animation: warningGlow 1.5s ease-in-out infinite alternate;
}

@keyframes warningGlow {
  from {
    box-shadow: 
      0 0 20px rgba(255, 165, 0, 0.3),
      inset 0 0 20px rgba(255, 165, 0, 0.1);
  }
  to {
    box-shadow: 
      0 0 40px rgba(255, 165, 0, 0.6),
      inset 0 0 30px rgba(255, 165, 0, 0.2);
  }
}

/* Responsive effects */
@media (max-width: 768px) {
  .corner-accent {
    width: 15px;
    height: 15px;
  }
  
  .edge-highlight.top,
  .edge-highlight.bottom {
    left: 15px;
    right: 15px;
  }
  
  .edge-highlight.left,
  .edge-highlight.right {
    top: 15px;
    bottom: 15px;
  }
}

/* Ensure borders don't interfere with interactions */
.refraction-border * {
  pointer-events: none !important;
}
</style> 